// executor/src/database.rs
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use shared_models::OrderDetails;
use std::path::Path;
use tracing::info;

// --- Trade Record Struct ---
#[derive(Debug)]
pub struct TradeRecord {
    pub id: i64,
    pub strategy_id: String,
    pub token_address: String,
    pub symbol: String, // Stored for dashboard convenience
    pub amount_usd: f64,
    pub status: String,
    pub signature: Option<String>,
    pub entry_time: i64,
    pub entry_price_usd: f64,
    pub close_time: Option<i64>,
    pub close_price_usd: Option<f64>,
    pub pnl_usd: Option<f64>,
    pub confidence: f64,
}

// --- Database Manager ---
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let path = Path::new(db_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path).with_context(|| format!("Failed to open database at {}", db_path))?;
        info!("Database opened at {}", db_path);
        Self::init_db(&conn)?;
        Ok(Self { conn })
    }

    fn init_db(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS trades (
                id INTEGER PRIMARY KEY,
                strategy_id TEXT NOT NULL,
                token_address TEXT NOT NULL,
                symbol TEXT NOT NULL,
                amount_usd REAL NOT NULL,
                status TEXT NOT NULL, -- PENDING, OPEN, CLOSED_PROFIT, CLOSED_LOSS, CANCELED
                signature TEXT,
                entry_time INTEGER NOT NULL,
                entry_price_usd REAL NOT NULL,
                close_time INTEGER,
                close_price_usd REAL,
                pnl_usd REAL,
                confidence REAL NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn log_trade_attempt(&self, details: &OrderDetails, strategy_id: &str, entry_price_usd: f64) -> Result<i64> {
        let now: DateTime<Utc> = Utc::now();
        self.conn.execute(
            "INSERT INTO trades (strategy_id, token_address, symbol, amount_usd, status, entry_time, entry_price_usd, confidence)
             VALUES (?1, ?2, ?3, ?4, 'PENDING', ?5, ?6, ?7)",
            params![
                strategy_id,
                details.token_address,
                details.token_address, // Use address as symbol for now, can be updated later
                details.suggested_size_usd,
                now.timestamp(),
                entry_price_usd,
                details.confidence,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn open_trade(&self, trade_id: i64, signature: &str) -> Result<()> {
        self.conn.execute("UPDATE trades SET status = 'OPEN', signature = ?1 WHERE id = ?2", params![signature, trade_id])?;
        Ok(())
    }
    
    pub fn get_all_trades(&self) -> Result<Vec<TradeRecord>> {
        let mut stmt = self.conn.prepare("SELECT * FROM trades ORDER BY entry_time DESC")?;
        let trades_iter = stmt.query_map([], |row| {
            Ok(TradeRecord {
                id: row.get(0)?,
                strategy_id: row.get(1)?,
                token_address: row.get(2)?,
                symbol: row.get(3)?,
                amount_usd: row.get(4)?,
                status: row.get(5)?,
                signature: row.get(6)?,
                entry_time: row.get(7)?,
                entry_price_usd: row.get(8)?,
                close_time: row.get(9)?,
                close_price_usd: row.get(10)?,
                pnl_usd: row.get(11)?,
                confidence: row.get(12)?,
            })
        })?;

        trades_iter.collect::<Result<Vec<TradeRecord>, rusqlite::Error>>().map_err(anyhow::Error::from)
    }

    pub fn update_trade_pnl(&self, trade_id: i64, status: &str, close_price_usd: f64, pnl_usd: f64) -> Result<()> {
        let now: DateTime<Utc> = Utc::now();
        self.conn.execute(
            "UPDATE trades SET status = ?1, close_time = ?2, close_price_usd = ?3, pnl_usd = ?4 WHERE id = ?5",
            params![status, now.timestamp(), close_price_usd, pnl_usd, trade_id],
        )?;
        Ok(())
    }

    // P-6: New function to get total PnL for portfolio monitor
    pub fn get_total_pnl(&self) -> Result<f64> {
        let total: f64 = self.conn.query_row(
            "SELECT SUM(pnl_usd) FROM trades WHERE status LIKE 'CLOSED_%'",
            [],
            |row| row.get(0),
        ).unwrap_or(0.0);
        Ok(total)
    }
}
