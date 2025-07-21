use crate::{register_strategy, strategies::{Strategy, MarketEvent, StrategyAction, OrderDetails, EventType}};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use tracing::info;
use shared_models::Side; // P-5: Import Side

#[derive(Default, Deserialize)]
struct LiquidityMigration {
    min_volume_migrate_usd: f64,
    #[serde(skip)] migrated_tokens: HashSet<String>, // Tracks tokens we've already traded for this migration
}

#[async_trait]
impl Strategy for LiquidityMigration {
    fn id(&self) -> &'static str { "liquidity_migration" }
    // This strategy would ideally subscribe to 'OnChain' events signaling LP movements.
    // For simulation, we'll react to unusual volume spikes after a quiet period.
    fn subscriptions(&self) -> HashSet<EventType> { [EventType::Price].iter().cloned().collect() }

    async fn init(&mut self, params: &Value) -> Result<()> {
        #[derive(Deserialize)] struct P { min_volume_migrate_usd: f64 }
        let p: P = serde_json::from_value(params.clone())?;
        self.min_volume_migrate_usd = p.min_volume_migrate_usd;
        info!(strategy = self.id(), "Initialized with min_volume_migrate_usd: {}", self.min_volume_migrate_usd);
        Ok(())
    }

    async fn on_event(&mut self, event: &MarketEvent) -> Result<StrategyAction> {
        if let MarketEvent::Price(tick) = event {
            // Simplified: If a token has very high sudden volume and we haven't traded it for migration yet
            // A real strategy would involve tracking liquidity pools and detecting large fund movements
            if tick.volume_usd_1m > self.min_volume_migrate_usd && !self.migrated_tokens.contains(&tick.token_address) {
                info!(id = self.id(), token = %tick.token_address, "BUY signal: Detected large liquidity movement / volume spike (V: {:.0} USD).", tick.volume_usd_1m);
                self.migrated_tokens.insert(tick.token_address.clone()); // Prevent re-trading same event
                return Ok(StrategyAction::Execute(OrderDetails {
                    token_address: tick.token_address.clone(),
                    suggested_size_usd: 700.0,
                    confidence: 0.8,
                    side: Side::Long,
                }));
            }
        }
        Ok(StrategyAction::Hold)
    }
}
register_strategy!(LiquidityMigration, "liquidity_migration");
