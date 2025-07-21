// executor/src/main.rs
mod config;
mod database;
mod executor;
mod jito_client;
mod jupiter;
mod portfolio_monitor;
mod signer_client;
mod strategies;

use crate::config::CONFIG;
use anyhow::Result;
use database::Database;
use executor::MasterExecutor;
use std::sync::Arc;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(filter).init();

    info!(version = %env!("CARGO_PKG_VERSION"), "🚀 Starting MemeSnipe Executor Orchestrator v17-Pro (Patched)...");

    let db = Arc::new(Database::new(&CONFIG.database_path)?);
    let mut master_executor = MasterExecutor::new(db.clone()).await; // Pass db clone to MasterExecutor

    // P-6: Start the portfolio monitor task
    let paused = master_executor.paused_flag();
    tokio::spawn(portfolio_monitor::run_monitor(db.clone(), paused));

    master_executor.run().await?;
    Ok(())
}
