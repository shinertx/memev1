use anyhow::Result;
use redis::AsyncCommands;
use shared_models::{StrategyAllocation, StrategySpec};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, warn, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;
use statrs::statistics::{Mean, StandardDeviation};
use statrs::statistics::{Mean, StandardDeviation};

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(filter).init();
    
    info!("🚀 Starting Meta-Allocator v17...");

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://redis:6379".to_string());
    let client = redis::Client::open(redis_url)?;

    loop {
        info!("Allocator loop starting...");
        let mut conn = match client.get_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to connect to Redis: {}. Retrying in 10s.", e);
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        };

        info!("Checking strategy registry for new specs...");

        let specs_json: Vec<String> = match conn.smembers("strategy_registry").await {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to read strategy_registry from Redis: {}. Retrying in 10s.", e);
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        };
        
        let specs: Vec<StrategySpec> = specs_json.into_iter()
            .filter_map(|s| serde_json::from_str(&s).ok())
            .collect();

        if specs.is_empty() {
            warn!("No valid strategy specs found in registry. Waiting...");
            tokio::time::sleep(Duration::from_secs(30)).await;
            continue;
        }

        // 1. Get performance data for each strategy
        let mut strategy_metrics = HashMap::new();
        for spec in &specs {
            let pnl_history_key = format!("perf:{}:pnl_history", spec.id);
            let pnl_history_json: Vec<String> = conn.lrange(&pnl_history_key, 0, -1).await.unwrap_or_default();
            
            let pnl_values: Vec<f64> = pnl_history_json.into_iter()
                .filter_map(|s| s.parse::<f64>().ok())
                .collect();

            if pnl_values.len() > 1 {
                let mean_pnl = pnl_values.mean();
                let std_dev_pnl = pnl_values.std_dev();
                
                // Calculate Sharpe Ratio (simplified: uses mean PnL as excess return, std dev as risk)
                // A true Sharpe would use daily returns and risk-free rate
                let sharpe_ratio = if std_dev_pnl > 0.0 { mean_pnl / std_dev_pnl } else { 0.0 };
                strategy_metrics.insert(spec.id.clone(), (mean_pnl, sharpe_ratio));
            } else {
                strategy_metrics.insert(spec.id.clone(), (0.0, 0.0)); // No data yet
            }
        }

        // 2. Calculate weights based on Sharpe Ratio (and PnL for tie-breaking)
        let mut sorted_strategies: Vec<&StrategySpec> = specs.iter().collect();
        sorted_strategies.sort_by(|a, b| {
            let (pnl_a, sharpe_a) = strategy_metrics.get(&a.id).unwrap_or(&(0.0, 0.0));
            let (pnl_b, sharpe_b) = strategy_metrics.get(&b.id).unwrap_or(&(0.0, 0.0));
            
            sharpe_b.partial_cmp(sharpe_a) // Higher Sharpe first
                .unwrap_or_else(|| pnl_b.partial_cmp(pnl_a).unwrap_or(std::cmp::Ordering::Equal)) // Then higher PnL
        });

        let mut allocations: Vec<StrategyAllocation> = Vec::new();
        let mut total_sharpe_for_weighting = 0.0;
        for (i, spec) in sorted_strategies.iter().enumerate() {
            let (_, sharpe) = strategy_metrics.get(&spec.id).unwrap_or(&(0.0, 0.0));
            // Only consider positive Sharpe ratios for weighting, or a small base weight for new strategies
            let weight_factor = sharpe.max(0.1); // Give a floor to new/low-sharpe strategies
            total_sharpe_for_weighting += weight_factor;
        }

        for spec in sorted_strategies {
            let (_, sharpe) = strategy_metrics.get(&spec.id).unwrap_or(&(0.0, 0.0));
            let weight = if total_sharpe_for_weighting > 0.0 {
                (sharpe.max(0.1)) / total_sharpe_for_weighting
            } else {
                1.0 / specs.len() as f64 // Fallback if no positive sharpe sum
            };
            
            allocations.push(StrategyAllocation { id: spec.id.clone(), weight, sharpe_ratio: *sharpe });
        }

        info!("Publishing {} allocations with dynamic Sharpe-based weights.", allocations.len());
        let payload = serde_json::to_string(&allocations)?;
        
        // Store current allocations for dashboard
        conn.set("active_allocations", &payload).await?; 
        // Publish to executor
        if let Err(e) = conn.publish("allocations_channel", payload).await {
            warn!("Failed to publish allocations: {}.", e);
        }

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
