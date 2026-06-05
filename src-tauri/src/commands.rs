use crate::claude_config::{read_settings, write_env_atomic, ClaudeSettings};
use crate::db::{self, VendorInstance};
use crate::keyring_store;
use crate::launcher;
use crate::vendor;
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

const KEYRING_SERVICE: &str = "MiniMax-vendor-switcher";

pub struct AppState {
    pub db: Mutex<Connection>,
    pub settings_path: Mutex<PathBuf>,
}

fn now_ts() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[tauri::command]
pub fn list_vendors(state: State<AppState>) -> Result<Vec<VendorInstance>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::list_vendors(&conn).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct VendorPresetDto {
    pub id: &'static str,
    pub name: &'static str,
    pub api_base: &'static str,
    pub default_model: &'static str,
}

#[tauri::command]
pub fn list_presets() -> Vec<VendorPresetDto> {
    vendor::presets()
        .into_iter()
        .map(|p| VendorPresetDto {
            id: p.id,
            name: p.name,
            api_base: p.api_base,
            default_model: p.default_model,
        })
        .collect()
}

#[derive(serde::Deserialize)]
pub struct CreateVendorInput {
    pub preset_id: Option<String>,
    pub name: String,
    pub api_base: String,
    pub model: String,
    pub api_key: String,
}

#[tauri::command]
pub fn create_vendor(
    state: State<AppState>,
    input: CreateVendorInput,
) -> Result<VendorInstance, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let keyring_key = format!("vendor:{}", id);
    keyring_store::set_key(KEYRING_SERVICE, &keyring_key, &input.api_key)
        .map_err(|e| format!("Keyring 写入失败: {}", e))?;

    let v = VendorInstance {
        id: id.clone(),
        preset_id: input.preset_id,
        name: input.name,
        api_base: input.api_base,
        model: input.model,
        keyring_key,
        created_at: now_ts(),
        updated_at: now_ts(),
    };
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::insert_vendor(&conn, &v).map_err(|e| e.to_string())?;
    Ok(v)
}

#[derive(serde::Deserialize)]
pub struct UpdateVendorInput {
    pub id: String,
    pub name: String,
    pub api_base: String,
    pub model: String,
    pub api_key: Option<String>,
}

#[tauri::command]
pub fn update_vendor(
    state: State<AppState>,
    input: UpdateVendorInput,
) -> Result<VendorInstance, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut existing = db::get_vendor(&conn, &input.id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "vendor not found".to_string())?;

    existing.name = input.name;
    existing.api_base = input.api_base;
    existing.model = input.model;
    existing.updated_at = now_ts();

    if let Some(key) = input.api_key {
        if !key.is_empty() {
            keyring_store::set_key(KEYRING_SERVICE, &existing.keyring_key, &key)
                .map_err(|e| format!("Keyring 写入失败: {}", e))?;
        }
    }
    db::update_vendor(&conn, &existing).map_err(|e| e.to_string())?;
    Ok(existing)
}

#[tauri::command]
pub fn delete_vendor(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let v = db::get_vendor(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "vendor not found".to_string())?;
    let _ = keyring_store::delete_key(KEYRING_SERVICE, &v.keyring_key);
    db::delete_vendor(&conn, &id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn apply_vendor(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let v = db::get_vendor(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "vendor not found".to_string())?;
    let api_key = keyring_store::get_key(KEYRING_SERVICE, &v.keyring_key)
        .map_err(|e| format!("Keyring 读取失败: {}", e))?;

    let path = state.settings_path.lock().map_err(|e| e.to_string())?.clone();
    let mut settings: ClaudeSettings = read_settings(&path).map_err(|e| e.to_string())?;
    let mut env: HashMap<String, String> = settings.env.clone().unwrap_or_default();
    env.insert("ANTHROPIC_BASE_URL".into(), v.api_base.clone());
    env.insert("ANTHROPIC_AUTH_TOKEN".into(), api_key);
    env.insert("ANTHROPIC_MODEL".into(), v.model.clone());
    settings.env = Some(env);
    write_env_atomic(&path, &settings).map_err(|e| e.to_string())?;

    // 记录当前激活
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('active_vendor', ?1)",
        [&id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_active_vendor(state: State<AppState>) -> Result<Option<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = 'active_vendor'")
        .map_err(|e| e.to_string())?;
    let mut iter = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    Ok(iter.next().transpose().map_err(|e| e.to_string())?)
}

#[tauri::command]
pub fn launch_claude_cmd() -> Result<u32, String> {
    launcher::launch_claude().map_err(|e| format!("启动失败: {}", e))
}

#[tauri::command]
pub fn is_claude_installed() -> bool {
    launcher::find_claude().is_some()
}
