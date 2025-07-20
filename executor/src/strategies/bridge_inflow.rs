use crate::{register_strategy, strategies::Strategy};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use tracing::info;
use shared_models::{EventType, MarketEvent, StrategyAction, OrderDetails, Side};

#[derive(Default, Deserialize)]
struct BridgeInflow {
    min_bridge_volume_usd: f64,
    #[serde(skip)] tokens_with_recent_inflow: HashSet<String>,
}

#[async_trait]
impl Strategy for BridgeInflow {
    fn id(&self) -> &'static str { "bridge_inflow" }
    // This strategy needs a new 'BridgeEvent' type.
    // For simulation, we'll use high volume price ticks as a proxy.
    fn subscriptions(&self) -> HashSet<EventType> { [EventType::Price].iter().cloned().collect() }

    async fn init(&mut self, params: &Value) -> Result<()> {
        #[derive(Deserialize)] struct P { min_bridge_volume_usd: f64 }
        let p: P = serde_json::from_value(params.clone())?;
        self.min_bridge_volume_usd = p.min_bridge_volume_usd;
        info!(strategy = self.id(), "Initialized with min_bridge_volume_usd: {}", self.min_bridge_volume_usd);
        Ok(())
    }

    async fn on_event(&mut self, event: &MarketEvent) -> Result<StrategyAction> {
        if let MarketEvent::Price(tick) = event {
            // Simulate: A very large volume might indicate significant inflow, potentially from a bridge.
            // A real strategy would monitor Wormhole or other bridge contracts directly.
            if tick.volume_usd_1m > self.min_bridge_volume_usd * 2.0 && !self.tokens_with_recent_inflow.contains(&tick.token_address) {
                info!(id = self.id(), token = %tick.token_address, "BUY signal: Detected simulated bridge inflow (V: {:.0} USD).", tick.volume_usd_1m);
                self.tokens_with_recent_inflow.insert(tick.token_address.clone());
                return Ok(StrategyAction::Execute(OrderDetails {
                    token_address: tick.token_address.clone(),
                    suggested_size_usd: 800.0,
                    confidence: 0.75,
                    side: Side::Long,
                }));
            }
        }
        Ok(StrategyAction::Hold)
    }
}
register_strategy!(BridgeInflow, "bridge_inflow");
