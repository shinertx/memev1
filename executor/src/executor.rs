// executor/src/executor.rs
use crate::{config::CONFIG, database::Database, jupiter::JupiterClient, portfolio_monitor, signer_client, strategies};
use anyhow::{anyhow, Result};
use shared_models::{MarketEvent, StrategyAction, StrategyAllocation, OrderDetails, EventType, Side};
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::sync::mpsc::{self, Sender, Receiver};
use tokio::task::JoinHandle;
use tracing::{error, info, instrument, warn};
use redis::AsyncCommands; // P-7: For Redis Streams
// add to top
use drift_sdk::{Client as DriftClient, Network as DriftNet, OpenPositionArgs};
use jito_searcher_client::JitoClient;

pub struct MasterExecutor {
    db: Arc<Database>,
    active_strategies: HashMap<String, (Sender<MarketEvent>, JoinHandle<()>)>, // ID -> (Sender, TaskHandle)
    event_router_senders: HashMap<EventType, Vec<Sender<MarketEvent>>>, // EventType -> List of interested strategy senders
    redis_client: redis::Client, // P-7: Client for Redis Streams
    jupiter_client: Arc<JupiterClient>,
    sol_usd_price: Arc<tokio::sync::Mutex<f64>>, // P-2: Store live SOL/USD price
    portfolio_paused: Arc<tokio::sync::Mutex<bool>>, // P-6: Flag to pause trading
    /* existing fields */ 
    jito_client: Arc<JitoClient>,               // NEW
    drift:       Arc<DriftClient>,              // NEW
}
}

impl MasterExecutor {
    pub async fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            active_strategies: HashMap::new(),
            event_router_senders: HashMap::new(),
            redis_client: redis::Client::open(CONFIG.redis_url.clone()).unwrap(),
            jupiter_client: Arc::new(JupiterClient::new()),
            sol_usd_price: Arc::new(tokio::sync::Mutex::new(100.0)), // P-2: Default to $100
            portfolio_paused: Arc::new(tokio::sync::Mutex::new(false)), // P-6: Not paused by default
            // inside new()
            jito_client: Arc::new(JitoClient::new(CONFIG.jito_rpc_url.clone()).await.unwrap()),
            drift: Arc::new(DriftClient::connect(DriftNet::Mainnet).await.unwrap()),
        }
    }

    // simple getter for monitor
    pub fn paused_flag(&self) -> Arc<tokio::sync::Mutex<bool>> {
        self.portfolio_paused.clone()
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting Master Executor run loop.");
        
        let mut allocation_listener = self.redis_client.get_async_connection().await?.into_pubsub();
        allocation_listener.subscribe("allocations_channel").await?;

        let mut price_event_listener = self.redis_client.get_async_connection().await?.into_pubsub();
        price_event_listener.subscribe("events:price").await?;

        let mut social_event_listener = self.redis_client.get_async_connection().await?.into_pubsub();
        social_event_listener.subscribe("events:social").await?;

        loop {
            tokio::select! {
                Some(msg) = allocation_listener.get_message() => {
                    if let Ok(payload) = msg.get_payload::<String>() {
                        if let Ok(allocations) = serde_json::from_str::<Vec<StrategyAllocation>>(&payload) {
                            self.reconcile_strategies(allocations).await;
                        } else {
                            error!("Failed to deserialize allocations: {}", payload);
                        }
                    } else {
                        error!("Failed to get payload from allocation_listener message.");
                    }
                }
                Some(msg) = price_event_listener.get_message() => {
                    if let Ok(payload) = msg.get_payload::<String>() {
                        if let Ok(event) = serde_json::from_str::<shared_models::PriceTick>(&payload) {
                            self.dispatch_event(shared_models::MarketEvent::Price(event)).await;
                        } else {
                            error!("Failed to deserialize PriceTick: {}", payload);
                        }
                    }
                }
                Some(msg) = social_event_listener.get_message() => {
                    if let Ok(payload) = msg.get_payload::<String>() {
                        if let Ok(event) = serde_json::from_str::<shared_models::SocialMention>(&payload) {
                            self.dispatch_event(shared_models::MarketEvent::Social(event)).await;
                        } else {
                            error!("Failed to deserialize SocialMention: {}", payload);
                        }
                    }
                }
            }
        }
    }

    async fn reconcile_strategies(&mut self, allocations: Vec<StrategyAllocation>) {
        let new_ids: HashMap<String, StrategyAllocation> = allocations.into_iter().map(|a| (a.id.clone(), a)).collect();
        let current_ids: Vec<String> = self.active_strategies.keys().cloned().collect();

        // 1. Stop strategies that are no longer allocated
        for id in current_ids.iter().filter(|id| !new_ids.contains_key(*id)) {
            if let Some((_, handle)) = self.active_strategies.remove(id) {
                handle.abort();
                info!(strategy = id, "Stopped strategy due to deallocation.");
            }
            // Remove from event router senders as well
            for (_, senders) in self.event_router_senders.iter_mut() {
                senders.retain(|s| !s.is_closed()); // Remove closed channels
            }
        }

        // 2. Start new strategies and update existing weights
        for (id, alloc) in new_ids {
            if !self.active_strategies.contains_key(&id) {
                info!(strategy = id, weight = alloc.weight, "Starting new strategy.");
                if let Some(mut strategy_instance) = self.build_strategy(&id) {
                    if let Err(e) = strategy_instance.init(&alloc.params).await { // Pass actual params from alloc
                        error!(strategy = id, error = %e, "Failed to initialize strategy, skipping.");
                        continue;
                    }

                    let (tx, rx) = mpsc::channel(100); // Bounded channel for backpressure
                    let strategy_id_clone = id.clone();
                    let db_clone = self.db.clone();
                    let jupiter_client_clone = self.jupiter_client.clone();

                    // Register subscriptions
                    for sub_type in strategy_instance.subscriptions() {
                        self.event_router_senders.entry(sub_type).or_default().push(tx.clone());
                    }

                    let handle = tokio::spawn(async move {
                        strategy_task(strategy_instance, rx, db_clone, jupiter_client_clone, 
                                     self.drift.clone(), self.jito_client.clone(), 
                                     self.sol_usd_price.clone(), strategy_id_clone).await;
                    });
                    self.active_strategies.insert(id, (tx, handle));
                } else {
                    warn!(strategy = id, "Strategy constructor not found. Skipping allocation.");
                }
            } else {
                // Strategy already running, potentially update its internal weight/config if needed
                // (Current strategy trait doesn't have an `update_params` method, but could be added)
                 info!(strategy = id, weight = alloc.weight, "Strategy already active, weight updated.");
            }
        }
    }

    async fn dispatch_event(&self, event: MarketEvent) {
        let event_type = event.get_type();
        if let Some(senders) = self.event_router_senders.get(&event_type) {
            for sender in senders {
                if let Err(e) = sender.send(event.clone()).await {
                    error!(event_type = ?event_type, error = %e, "Failed to dispatch event to strategy channel.");
                }
            }
        }
    }

    fn build_strategy(&self, id: &str) -> Option<Box<dyn strategies::Strategy>> {
        for constructor in inventory::iter::<strategies::StrategyConstructor> {
            if constructor.0 == id {
                return Some((constructor.1)());
            }
        }
        None
    }
}

#[instrument(skip(strategy_instance, rx, db, jupiter_client))]
async fn strategy_task(
    mut strategy_instance: Box<dyn strategies::Strategy>,
    mut rx: Receiver<MarketEvent>,
    db: Arc<Database>,
    jupiter_client: Arc<JupiterClient>,
    drift: Arc<DriftClient>,
    jito_client: Arc<JitoClient>,
    sol_price: Arc<tokio::sync::Mutex<f64>>,
    strategy_id: String,
) {
    info!(strategy = strategy_id.as_str(), "Strategy task started.");
    while let Some(event) = rx.recv().await {
        // ─────────────────── strategy_task ───────────────────
        match strategy_instance.on_event(&event).await {
            Ok(StrategyAction::Execute(details)) => {
                if let Err(e) = execute_trade(
                        db.clone(),
                        jupiter_client.clone(),
                        drift.clone(),
                        jito_client.clone(),
                        sol_price.clone(),
                        details,
                        &strategy_id,
                ).await { error!(strategy=%strategy_id, %e, "trade failed"); }
            }
            Ok(StrategyAction::Hold) => {}
            Err(e) => error!(strategy=%strategy_id, %e, "strategy error"),
        }
    }
    info!(strategy = strategy_id.as_str(), "Strategy task finished.");
}

#[instrument(skip(db, jupiter_client))]
// ─────────────────── execute_trade ───────────────────
#[instrument(skip_all)]
async fn execute_trade(
    db: Arc<Database>,
    jupiter: Arc<JupiterClient>,
    drift: Arc<DriftClient>,
    jito:  Arc<JitoClient>,
    sol_price: Arc<tokio::sync::Mutex<f64>>,
    details: OrderDetails,
    strategy_id: &str,
) -> Result<()> {
    let is_live = !CONFIG.paper_trading_mode;
    let is_short = matches!(details.side, Side::Short);

    // ----------- sizing ----------
    let size_usd = details.suggested_size_usd.min(CONFIG.global_max_position_usd);

    // log attempt
    let quote = jupiter.get_quote(1.0, &details.token_address).await?;   // 1 USD probe
    let trade_id = db.log_trade_attempt(&details, strategy_id, quote.price_per_token)?;

    if !is_live {
        simulate_fill(&db, trade_id, size_usd, is_short)?;
        return Ok(())
    }

    // ------------- live -------------
    if  is_short {
        // Perp short on Drift
        let margin_acct = drift.get_or_create_user().await?;
        let args = OpenPositionArgs {
            market_index: 0,   // SOL-PERP
            direction: drift_sdk::Direction::Short,
            base_asset_amount: (size_usd / *sol_price.lock().await * 1e9) as u64,
            limit_price: None,
            reduce_only: false,
        };
        let sig = drift.open_position(&margin_acct, &args).await?;
        db.open_trade(trade_id, &sig.to_string())?;
    } else {
        // Spot buy via Jupiter
        let user_pk = Pubkey::from_str(&signer_client::get_pubkey().await?)?;
        let swap_b64 = jupiter.get_swap_transaction(&user_pk, &details.token_address, size_usd).await?;
        let signed_b64 = signer_client::sign_transaction(&swap_b64).await?;
        let mut tx     = jupiter::deserialize_transaction(&signed_b64)?;

        // ------- Jito tip injection --------
        let bh = jito.get_recent_blockhash().await?;
        tx.message.set_recent_blockhash(bh);
        jito.attach_tip(&mut tx, CONFIG.jito_tip_lamports).await?;
        jito.send_transaction(&tx).await?;

        db.open_trade(trade_id, &tx.signatures[0].to_string())?;
    }
    Ok(())
}

// helper
fn simulate_fill(db:&Database, id:i64, size:f64, short:bool) -> Result<()> {
    let pnl = size * (rand::random::<f64>()*0.1 - 0.05) * if short { -1.0 } else { 1.0 };
    let status = if pnl>0.0 { "CLOSED_PROFIT" } else { "CLOSED_LOSS" };
    db.open_trade(id,"paper")?;
    db.update_trade_pnl(id,status,0.0,pnl)?;
    Ok(())
}
