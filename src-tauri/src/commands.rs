use crate::db::{self, VendorInstance};
use crate::keyring_store;
use crate::launcher;
use crate::minimax_config;
use crate::vendor;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

const KEYRING_SERVICE: &str = "MiniMax-vendor-switcher";

pub struct AppState {
    pub db: Mutex<Connection>,
    pub config_path: Mutex<PathBuf>,
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

    let v = VendorInstance {
        id: id.clone(),
        preset_id: input.preset_id,
        name: input.name,
        api_base: input.api_base,
        model: input.model,
        keyring_key: keyring_key.clone(),
        created_at: now_ts(),
        updated_at: now_ts(),
    };

    // 1) Insert DB row first
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::insert_vendor(&conn, &v).map_err(|e| e.to_string())?;
    }

    // 2) Write keyring; on failure roll back DB row
    if let Err(e) = keyring_store::set_key(KEYRING_SERVICE, &keyring_key, &input.api_key) {
        if let Ok(conn) = state.db.lock() {
            let _ = db::delete_vendor(&conn, &id);
        }
        return Err(format!("Keyring 写入失败: {}", e));
    }

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
    // 1) 读取并更新数据库（提前释放 conn 锁）
    let (updated, is_active) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let mut existing = db::get_vendor(&conn, &input.id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "vendor not found".to_string())?;

        existing.name = input.name;
        existing.api_base = input.api_base;
        existing.model = input.model;
        existing.updated_at = now_ts();

        // 更新 API Key（如果提供了新 key）
        if let Some(ref key) = input.api_key {
            if !key.is_empty() {
                keyring_store::set_key(KEYRING_SERVICE, &existing.keyring_key, key)
                    .map_err(|e| format!("Keyring 写入失败: {}", e))?;
            }
        }

        db::update_vendor(&conn, &existing).map_err(|e| e.to_string())?;

        // 检查是否当前激活的厂商
        let active = get_active_vendor_inner(&conn).ok().flatten();
        let is_active = active.as_deref() == Some(&input.id);

        (existing, is_active)
    }; // conn 锁在此释放

    // 2) 如果该厂商是当前激活的，立即重新写入 config.yaml
    if is_active {
        let api_key = keyring_store::get_key(KEYRING_SERVICE, &updated.keyring_key)
            .map_err(|e| format!("Keyring 读取失败: {}", e))?;

        let path = state.config_path.lock().map_err(|e| e.to_string())?.clone();
        // provider_id 统一小写，确保与 config.yaml 中的 key 大小写不敏感匹配
        let provider_id = updated.preset_id.clone().unwrap_or_else(|| {
            updated.name.clone()
        }).to_lowercase().replace(' ', "-");

        minimax_config::apply_provider(
            &path,
            &provider_id,
            &updated.name,
            &updated.api_base,
            &updated.model,
            &api_key,
        )
        .map_err(|e| format!("配置文件写入失败: {}", e))?;
    }

    Ok(updated)
}

#[tauri::command]
pub fn delete_vendor(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let v = db::get_vendor(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "vendor not found".to_string())?;

    // 1) Delete DB row first
    db::delete_vendor(&conn, &id).map_err(|e| e.to_string())?;

    // 2) Best-effort keyring cleanup
    if let Err(e) = keyring_store::delete_key(KEYRING_SERVICE, &v.keyring_key) {
        return Err(format!(
            "厂商已删除，但 Keyring 清理失败（需手动移除 vendor:{}）: {}",
            v.id, e
        ));
    }

    // Clear active_vendor if it pointed to this one
    let _ = conn.execute(
        "DELETE FROM settings WHERE key = 'active_vendor' AND value = ?1",
        [&id],
    );

    Ok(())
}

/// 写入 MiniMax 桌面版 config.yaml + 记录激活状态
#[tauri::command]
pub fn apply_vendor(state: State<AppState>, id: String) -> Result<(), String> {
    // 读取厂商信息（提前释放锁）
    let (vendor, api_key) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let v = db::get_vendor(&conn, &id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "vendor not found".to_string())?;
        let key = keyring_store::get_key(KEYRING_SERVICE, &v.keyring_key)
            .map_err(|e| format!("Keyring 读取失败: {}", e))?;
        (v, key)
    };

    // 写入 MiniMax config.yaml
    let path = state.config_path.lock().map_err(|e| e.to_string())?.clone();
    let provider_id = vendor.preset_id.clone().unwrap_or_else(|| {
        vendor.name.clone()
    }).to_lowercase().replace(' ', "-");

    minimax_config::apply_provider(
        &path,
        &provider_id,
        &vendor.name,
        &vendor.api_base,
        &vendor.model,
        &api_key,
    )
    .map_err(|e| format!("MiniMax 配置写入失败: {}", e))?;

    // 记录当前激活
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('active_vendor', ?1)",
            [&id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn get_active_vendor_inner(conn: &Connection) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = 'active_vendor'")
        .map_err(|e| e.to_string())?;
    let mut iter = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    Ok(iter.next().transpose().map_err(|e| e.to_string())?)
}

#[tauri::command]
pub fn get_active_vendor(state: State<AppState>) -> Result<Option<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    get_active_vendor_inner(&conn)
}

#[tauri::command]
pub fn launch_claude_cmd() -> Result<u32, String> {
    launcher::launch_claude().map_err(|e| format!("启动失败: {}", e))
}

#[tauri::command]
pub fn is_claude_installed() -> bool {
    launcher::find_claude().is_some()
}
