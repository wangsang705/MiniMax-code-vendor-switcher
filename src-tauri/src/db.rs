use rusqlite::{Connection, Result};
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VendorInstance {
    pub id: String,
    pub preset_id: Option<String>,
    pub name: String,
    pub api_base: String,
    pub model: String,
    pub keyring_key: String,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn init_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS vendors (
            id TEXT PRIMARY KEY,
            preset_id TEXT,
            name TEXT NOT NULL,
            api_base TEXT NOT NULL,
            model TEXT NOT NULL,
            keyring_key TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        "#,
    )?;
    Ok(conn)
}
