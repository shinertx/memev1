//! Common structs used by every service.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/* ---------- enums ---------- */

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum EventType {
    Price,
    Social,
    Depth,
    Bridge,
    Funding,
    SolPrice,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Side { Long, Short }

/* ---------- strategy plumbing ---------- */

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StrategySpec {
    pub id: String,
    pub family: String,
    pub params: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StrategyAllocation {
    pub id: String,
    pub weight: f64,
    pub sharpe_ratio: f64,
}

/* ---------- market events ---------- */

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceTick       { pub token_address: String, pub price_usd: f64, pub volume_usd_1m: f64 }
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SocialMention   { pub token_address: String, pub source: String, pub sentiment: f64 }
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DepthEvent      { pub token_address: String, pub bid_price: f64, pub ask_price: f64, pub bid_size_usd: f64, pub ask_size_usd: f64 }
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BridgeEvent     { pub token_address: String, pub source_chain: String, pub destination_chain: String, pub volume_usd: f64 }
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FundingEvent    { pub token_address: String, pub funding_rate_pct: f64, pub next_funding_time_sec: u64 }
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SolPriceEvent   { pub price_usd: f64 }
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum MarketEvent {
    Price(PriceTick),
    Social(SocialMention),
    Depth(DepthEvent),
    Bridge(BridgeEvent),
    Funding(FundingEvent),
    SolPrice(SolPriceEvent),
}

impl MarketEvent {
    pub fn get_type(&self) -> EventType {
        use MarketEvent::*;
        match self {
            Price(_)      => EventType::Price,
            Social(_)     => EventType::Social,
            Depth(_)      => EventType::Depth,
            Bridge(_)     => EventType::Bridge,
            Funding(_)    => EventType::Funding,
            SolPrice(_)   => EventType::SolPrice,
        }
    }
    /// helper for strategies that need the token symbol quickly
    pub fn token(&self) -> &str {
        use MarketEvent::*;
        match self {
            Price(e)   | Social(e) | Depth(e) | Bridge(e) | Funding(e) => e.token_address.as_str(),
            SolPrice(_) => "So11111111111111111111111111111111111111112",
        }
    }
}

/* ---------- execution ---------- */

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OrderDetails {
    pub token_address: String,
    pub suggested_size_usd: f64,
    pub confidence: f64,
    pub side: Side,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum StrategyAction {
    Execute(OrderDetails),   // single unified action
    Hold,
}

/* ---------- signer ---------- */

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignRequest  { pub transaction_b64: String }
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignResponse { pub signed_transaction_b64: String }
