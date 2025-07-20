use crate::{config::CONFIG, database::Database};
use anyhow::Result;
use std::{sync::Arc, time::Duration};
use tracing::{error, info, warn};
use redis::AsyncCommands;

pub async fn run_monitor(db: Arc<Database>, paused_flag: Arc<tokio::sync::Mutex<bool>>) {
    info!("📈 Portfolio monitor online");
    let client = redis::Client::open(CONFIG.redis_url.clone()).unwrap();
    let mut hwm = 0.0_f64;

    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;

        let pnl = match db.get_total_pnl() {
            Ok(p) => p, Err(e) => { error!("DB error {e}"); continue }
        };
        hwm = hwm.max(pnl);
        let dd = if hwm>0.0 { (hwm-pnl)/hwm*100.0 } else { 0.0 };

        info!("PnL {:.2} USD | Peak {:.2} | DD {:.2}%", pnl,hwm,dd);

        let mut conn = match client.get_async_connection().await {
            Ok(c)=>c, Err(e)=>{ warn!("Redis err {e}"); continue }
        };

        if dd > CONFIG.portfolio_stop_loss_percent {
            if !*paused_flag.lock().await {
                conn.publish("kill_switch_channel","PAUSE").await.ok();
                *paused_flag.lock().await = true;
                error!("🚨 Trading paused – draw-down {:.1}% > {:.1}%", dd,CONFIG.portfolio_stop_loss_percent);
            }
        } else if *paused_flag.lock().await && dd < CONFIG.portfolio_stop_loss_percent*0.8 {
            conn.publish("kill_switch_channel","RESUME").await.ok();
            *paused_flag.lock().await = false;
            info!("✅ Trading resumed – draw-down {:.1}%", dd);
        }
    }
}
                    // P-6: Publish to kill switch channel
                    if let Err(e) = conn.publish("kill_switch_channel", "PAUSE").await {
                        error!("Failed to publish PAUSE to kill_switch_channel: {}", e);
                    }
                } else {
                    // If drawdown is recovered significantly, resume trading
                    if drawdown_from_peak < CONFIG.portfolio_stop_loss_percent * 0.8 { // Resume if recovered significantly
                        info!("✅ Portfolio recovered. Drawdown {:.2}% < Threshold {:.2}%. Resuming trading.",
                            drawdown_from_peak, CONFIG.portfolio_stop_loss_percent * 0.8);
                        if let Err(e) = conn.publish("kill_switch_channel", "RESUME").await {
                            error!("Failed to publish RESUME to kill_switch_channel: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Portfolio Monitor: Failed to get total PnL from DB: {}", e);
            }
        }
    }
}
