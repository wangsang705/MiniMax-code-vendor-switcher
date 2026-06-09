use super::{now_ts, AppState, KEYRING_SERVICE};
use crate::common::path_util;
use crate::db::{self, VendorInstance};
use crate::keyring_db;
use crate::keyring_store;
use crate::launcher;
use crate::minimax_config;
use crate::vendor;
use tauri::State;

#[derive(serde::Serialize)]
pub struct VendorPresetDto {
 pub id: &'static str,
 pub name: &'static str,
 pub api_base: &'static str,
 pub default_model: &'static str,
}

#[derive(serde::Deserialize)]
pub struct CreateVendorInput {
 pub preset_id: Option<String>,
 pub name: String,
 pub api_base: String,
 pub model: String,
 pub api_key: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateVendorInput {
 pub id: String,
 pub name: String,
 pub api_base: String,
 pub model: String,
 pub api_key: Option<String>,
}

// =====旧命令（向后兼容） =====

#[tauri::command]
pub fn list_vendors(state: State<AppState>) -> Result<Vec<VendorInstance>, String> {
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 db::list_vendors(&conn).map_err(|e| e.to_string())
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

#[tauri::command]
pub fn create_vendor(
 state: State<AppState>,
 input: CreateVendorInput,
) -> Result<VendorInstance, String> {
 let CreateVendorInput {
 preset_id,
 name,
 api_base,
 model,
 api_key,
 } = input;

 let id = uuid::Uuid::new_v4().to_string();
 let keyring_key = format!("vendor:{}", id);
 let v = VendorInstance {
 id: id.clone(),
 preset_id,
 name,
 api_base,
 model,
 keyring_key: keyring_key.clone(),
 created_at: now_ts(),
 updated_at: now_ts(),
 };

 // =====阶段1：DB 先 commit =====
 {
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 db::insert_vendor(&conn, &v).map_err(|e| e.to_string())?;
 }

 // =====阶段2：Keyring 后写（DB 已 commit，失败仅 warn）=====
 if let Err(e) = keyring_db::write_keyring_one(KEYRING_SERVICE, &keyring_key, &api_key) {
 eprintln!(
 "Warning: vendor {} Keyring写入失败,DB 已保留: {}",
 id, e
 );
 return Err(e);
 }

 Ok(v)
}

#[tauri::command]
pub fn update_vendor(
 state: State<AppState>,
 input: UpdateVendorInput,
) -> Result<VendorInstance, String> {
 let UpdateVendorInput {
 id,
 name,
 api_base,
 model,
 api_key,
 } = input;

 // =====阶段1：读 +改字段（无副作用）=====
 let (updated, is_active, old_keyring_key) = {
 let mut conn = state.db.lock().map_err(|e| e.to_string())?;
 let mut existing = db::get_vendor(&conn, &id)
 .map_err(|e| e.to_string())?
 .ok_or_else(|| "vendor not found".to_string())?;
 existing.name = name;
 existing.api_base = api_base;
 existing.model = model;
 existing.updated_at = now_ts();
 let old_key = existing.keyring_key.clone();

 keyring_db::run_tx(&mut conn, |tx| db::update_vendor(tx, &existing))
 .map_err(|e| e.to_string())?;

 let active = conn
 .query_row("SELECT value FROM settings WHERE key='active_vendor'", [], |r| {
 r.get::<_, String>(0)
 })
 .ok();
 (existing, active.as_deref() == Some(&id), old_key)
 };

 // =====阶段2：Keyring 后写（不在 DB锁内）=====
 if let Some(ref key) = api_key {
 if !key.is_empty() {
 if let Err(e) = keyring_db::write_keyring_one(KEYRING_SERVICE, &updated.keyring_key, key) {
 eprintln!(
 "Warning: vendor {} Keyring写入失败,DB 已更新: {}",
 updated.id, e
 );
 return Err(e);
 }
 }
 }

 // =====阶段3：如果当前 vendor 是 active，重新写配置文件 =====
 if is_active {
 let api_key =
 keyring_store::get_key(KEYRING_SERVICE, &updated.keyring_key)
 .map_err(|e| format!("Keyring读取失败: {}", e))?;
 let path = path_util::minimax_config_path().ok_or("无法找到用户目录")?;
 let provider_id = updated
 .preset_id
 .clone()
 .unwrap_or_else(|| updated.name.clone())
 .to_lowercase()
 .replace(' ', "-");
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

 // =====阶段4：清理旧 keyring引用（如果 keyring_key变了）=====
 if old_keyring_key != updated.keyring_key {
 keyring_db::delete_keyring_best_effort(KEYRING_SERVICE, &[old_keyring_key]);
 }

 Ok(updated)
}

#[tauri::command]
pub fn delete_vendor(state: State<AppState>, id: String) -> Result<(), String> {
 // =====阶段1：读 vendor（无副作用）=====
 let v = {
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 db::get_vendor(&conn, &id)
 .map_err(|e| e.to_string())?
 .ok_or_else(|| "vendor not found".to_string())?
 };

 // =====阶段2：DB 先 commit 删除 +清理 settings =====
 {
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 db::delete_vendor(&conn, &id).map_err(|e| e.to_string())?;
 if let Err(e) = conn.execute(
 "DELETE FROM settings WHERE key='active_vendor' AND value=?1",
 [&id],
 ) {
 eprintln!("Warning:清理 active_vendor settings失败: {}", e);
 }
 }

 // =====阶段3：Keyring best-effort清理 =====
 keyring_db::delete_keyring_best_effort(KEYRING_SERVICE, &[v.keyring_key]);

 Ok(())
}

#[tauri::command]
pub fn apply_vendor(state: State<AppState>, id: String) -> Result<(), String> {
 let (vendor, api_key) = {
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 let v = db::get_vendor(&conn, &id)
 .map_err(|e| e.to_string())?
 .ok_or_else(|| "vendor not found".to_string())?;
 let key = keyring_store::get_key(KEYRING_SERVICE, &v.keyring_key)
 .map_err(|e| format!("Keyring读取失败: {}", e))?;
 (v, key)
 };
 let path = path_util::minimax_config_path().ok_or("无法找到用户目录")?;
 let provider_id = vendor
 .preset_id
 .clone()
 .unwrap_or_else(|| vendor.name.clone())
 .to_lowercase()
 .replace(' ', "-");
 minimax_config::apply_provider(
 &path,
 &provider_id,
 &vendor.name,
 &vendor.api_base,
 &vendor.model,
 &api_key,
 )
 .map_err(|e| format!("MiniMax 配置写入失败: {}", e))?;
 {
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 conn.execute(
 "INSERT OR REPLACE INTO settings (key,value) VALUES ('active_vendor',?1)",
 [&id],
 )
 .map_err(|e| e.to_string())?;
 }
 Ok(())
}

fn get_active_vendor_inner(conn: &rusqlite::Connection) -> Result<Option<String>, String> {
 let mut stmt = conn
 .prepare("SELECT value FROM settings WHERE key='active_vendor'")
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
