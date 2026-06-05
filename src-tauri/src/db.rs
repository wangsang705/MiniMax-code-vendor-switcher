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

pub fn insert_vendor(conn: &Connection, v: &VendorInstance) -> Result<()> {
    conn.execute(
        "INSERT INTO vendors (id, preset_id, name, api_base, model, keyring_key, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            v.id, v.preset_id, v.name, v.api_base, v.model, v.keyring_key, v.created_at, v.updated_at
        ],
    )?;
    Ok(())
}

pub fn list_vendors(conn: &Connection) -> Result<Vec<VendorInstance>> {
    let mut stmt = conn.prepare(
        "SELECT id, preset_id, name, api_base, model, keyring_key, created_at, updated_at
         FROM vendors ORDER BY created_at ASC",
    )?;
    let iter = stmt.query_map([], |row| {
        Ok(VendorInstance {
            id: row.get(0)?,
            preset_id: row.get(1)?,
            name: row.get(2)?,
            api_base: row.get(3)?,
            model: row.get(4)?,
            keyring_key: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    iter.collect()
}

pub fn get_vendor(conn: &Connection, id: &str) -> Result<Option<VendorInstance>> {
    let mut stmt = conn.prepare(
        "SELECT id, preset_id, name, api_base, model, keyring_key, created_at, updated_at
         FROM vendors WHERE id = ?1",
    )?;
    let mut iter = stmt.query_map([id], |row| {
        Ok(VendorInstance {
            id: row.get(0)?,
            preset_id: row.get(1)?,
            name: row.get(2)?,
            api_base: row.get(3)?,
            model: row.get(4)?,
            keyring_key: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    Ok(iter.next().transpose()?)
}

pub fn update_vendor(conn: &Connection, v: &VendorInstance) -> Result<()> {
    conn.execute(
        "UPDATE vendors SET preset_id=?2, name=?3, api_base=?4, model=?5,
         keyring_key=?6, updated_at=?7 WHERE id=?1",
        rusqlite::params![
            v.id, v.preset_id, v.name, v.api_base, v.model, v.keyring_key, v.updated_at
        ],
    )?;
    Ok(())
}

pub fn delete_vendor(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM vendors WHERE id = ?1", [id])?;
    Ok(())
}
