//! 观景 VISTA Tauri Command 入口
//!
//! 按领域拆分为子模块：
//! - vendor: 旧版兼容命令（Vendor CRUD + Claude 启动）
//! - provider_models: Provider/Model/Binding CRUD
//! - service: 工具检测/启动、AI 对话、一键安装

pub mod provider_models;
pub mod service;
pub mod vendor;

use crate::config_writer;
use crate::db;
use crate::keyring_store;
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::State;

// ===== 共享常量 =====

const KEYRING_SERVICE: &str = "MiniMax-vendor-switcher";

// ===== 共享状态 =====

pub struct AppState {
    pub db: Mutex<Connection>,
}

// ===== 共享辅助函数 =====

fn now_ts() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn provider_account(provider_id: &str) -> String {
    format!("provider:{}", provider_id)
}

fn binding_account(binding_id: &str) -> String {
    format!("binding:{}", binding_id)
}

fn fetch_provider_key(state: &State<AppState>, provider_id: &str) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let account = provider_account(provider_id);
    match keyring_store::get_key(KEYRING_SERVICE, &account) {
        Ok(key) if !key.trim().is_empty() => Ok(key),
        _ => {
            let legacy = db::get_provider_legacy_api_key(&conn, provider_id)
                .map_err(|e| e.to_string())?;
            let key = legacy
                .filter(|k| !k.trim().is_empty())
                .ok_or_else(|| {
                    "该厂商未保存 API Key，请在模型中心编辑厂商并填写 API Key".to_string()
                })?;
            drop(conn);
            keyring_store::set_key(KEYRING_SERVICE, &account, &key)
                .map_err(|e| format!("Keyring 写入失败: {}", e))?;
            let conn = state.db.lock().map_err(|e| e.to_string())?;
            let _ = db::clear_provider_legacy_api_key(&conn, provider_id);
            Ok(key)
        }
    }
}

fn apply_config_for_tool(
    state: &State<AppState>,
    tool_id: &str,
    provider_id: &str,
    model_name: &str,
    api_key: &str,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let provider = db::get_provider(&conn, provider_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("厂商未找到: {}", provider_id))?;
    drop(conn);

    let ctx = config_writer::WriteContext {
        provider_id,
        provider_name: &provider.name,
        base_url: &provider.api_base,
        model_name,
        api_key,
        anthropic_mode: provider.anthropic_mode,
    };

    config_writer::get_registry().write_config(tool_id, &ctx)
}

// ===== 工具级细粒度锁 =====

fn tool_locks() -> &'static Mutex<HashMap<String, Arc<Mutex<()>>>> {
    static LOCKS: OnceLock<Mutex<HashMap<String, Arc<Mutex<()>>>>> = OnceLock::new();
    LOCKS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_tool_lock(tool_id: &str) -> Arc<Mutex<()>> {
    let mut locks = tool_locks().lock().unwrap();
    locks
        .entry(tool_id.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}
