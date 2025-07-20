use crate::{register_strategy, strategies::Strategy};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashSet, HashMap};
use tracing::info;
use shared_models::{EventType, MarketEvent, StrategyAction, OrderDetails, Side};

#[derive(Default, Deserialize)]
struct PerpBasisArb {
    basis_threshold_pct: f64,
    #[serde(skip)] spot_prices: HashMap<String, f64>,
    #[serde(skip)] perp_funding_rates: HashMap<String, f64>, // Simplified to direct funding rate for token
}

#[async_trait]
impl Strategy for PerpBasisArb {
    fn id(&self) -> &'static str { "perp_basis_arb" }
    // This strategy needs both Price (spot) and a new 'PerpData' EventType (for funding/perp price)
    // For simulation, we will use Price events for both spot and a simulated perp component.
    fn subscriptions(&self) -> HashSet<EventType> { [EventType::Price].iter().cloned().collect() }

    async fn init(&mut self, params: &Value) -> Result<()> {
        #[derive(Deserialize)] struct P { basis_threshold_pct: f64 }
        let p: P = serde_json::from_value(params.clone())?;
        self.basis_threshold_pct = p.basis_threshold_pct;
        info!(strategy = self.id(), "Initialized with basis_threshold_pct: {}", self.basis_threshold_pct);
        Ok(())
    }

    async fn on_event(&mut self, event: &MarketEvent) -> Result<StrategyAction> {
        if let MarketEvent::Price(tick) = event {
            self.spot_prices.insert(tick.token_address.clone(), tick.price_usd);
            // Simulate funding rate: higher volume means more volatile funding
            let simulated_funding_rate = tick.volume_usd_1m / 1_000_000.0 * 0.001; // Tiny percentage
            self.perp_funding_rates.insert(tick.token_address.clone(), simulated_funding_rate);

            if let (Some(&spot_price), Some(&funding_rate)) = (self.spot_prices.get(&tick.token_address), self.perp_funding_rates.get(&tick.token_address)) {
                // Simplified: Basis is just the simulated funding rate
                // A real basis would be (perp_price - spot_price) / spot_price
                let basis = funding_rate; 

                if basis.abs() > self.basis_threshold_pct / 100.0 {
                    if basis > 0.0 { // Positive basis: perp is more expensive, short perp & long spot
                        info!(id = self.id(), token = %tick.token_address, "SHORT PERP/LONG SPOT signal: Basis {:.4}% is above threshold. (Simulated)", basis * 100.0);
                        return Ok(StrategyAction::Execute(OrderDetails { // Short leg
                            token_address: tick.token_address.clone(),
                            suggested_size_usd: 800.0,
                            confidence: 0.9,
                            side: Side::Short,
                        }));
                        // A real strategy would also execute the long spot leg here
                    } else { // Negative basis: perp is cheaper, long perp & short spot
                         info!(id = self.id(), token = %tick.token_address, "LONG PERP/SHORT SPOT signal: Basis {:.4}% is below threshold. (Simulated)", basis * 100.0);
                         return Ok(StrategyAction::Execute(OrderDetails { // Long leg
                             token_address: tick.token_address.clone(),
                             suggested_size_usd: 800.0,
                             confidence: 0.9,
                             side: Side::Long,
                         }));
                         // A real strategy would also execute the short spot leg here
                    }
                }
            }
        }
        Ok(StrategyAction::Hold)
    }
}
register_strategy!(PerpBasisArb, "perp_basis_arb");
