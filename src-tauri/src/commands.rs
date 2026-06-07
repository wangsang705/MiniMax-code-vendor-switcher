use crate::db::{self, VendorInstance};
use crate::claude_config;
use crate::detector;
use crate::keyring_store;
use crate::launcher;
use crate::minimax_config;
use crate::vendor;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

const KEYRING_SERVICE: &str = "MiniMax-vendor-switcher";

fn provider_account(provider_id: &str) -> String {
    format!("provider:{}", provider_id)
}

fn binding_account(binding_id: &str) -> String {
    format!("binding:{}", binding_id)
}

fn provider_has_key(state: &State<AppState>, provider_id: &str) -> Result<bool, String> {
    let account = provider_account(provider_id);
    if let Ok(key) = keyring_store::get_key(KEYRING_SERVICE, &account) {
        if !key.trim().is_empty() {
            return Ok(true);
        }
    }

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    Ok(db::get_provider_legacy_api_key(&conn, provider_id)
        .map_err(|e| e.to_string())?
        .is_some_and(|key| !key.trim().is_empty()))
}

fn hydrate_provider(state: &State<AppState>, mut provider: db::Provider) -> Result<db::Provider, String> {
    provider.has_api_key = provider_has_key(state, &provider.id)?;
    Ok(provider)
}

pub struct AppState {
    pub db: Mutex<Connection>,
    pub config_path: Mutex<PathBuf>,
}

fn now_ts() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

// ===== 旧命令（向后兼容） =====

#[tauri::command]
pub fn list_vendors(state: State<AppState>) -> Result<Vec<VendorInstance>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::list_vendors(&conn).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct VendorPresetDto { pub id: &'static str, pub name: &'static str, pub api_base: &'static str, pub default_model: &'static str }

#[tauri::command]
pub fn list_presets() -> Vec<VendorPresetDto> {
    vendor::presets().into_iter().map(|p| VendorPresetDto { id: p.id, name: p.name, api_base: p.api_base, default_model: p.default_model }).collect()
}

#[derive(serde::Deserialize)]
pub struct CreateVendorInput { pub preset_id: Option<String>, pub name: String, pub api_base: String, pub model: String, pub api_key: String }

#[tauri::command]
pub fn create_vendor(state: State<AppState>, input: CreateVendorInput) -> Result<VendorInstance, String> {
    let CreateVendorInput { preset_id, name, api_base, model, api_key } = input;
    let id = uuid::Uuid::new_v4().to_string();
    let keyring_key = format!("vendor:{}", id);
    let v = VendorInstance { id: id.clone(), preset_id, name, api_base, model, keyring_key: keyring_key.clone(), created_at: now_ts(), updated_at: now_ts() };
    { let conn = state.db.lock().map_err(|e| e.to_string())?; db::insert_vendor(&conn, &v).map_err(|e| e.to_string())?; }
    if let Err(e) = keyring_store::set_key(KEYRING_SERVICE, &keyring_key, &api_key) {
        if let Ok(conn) = state.db.lock() { let _ = db::delete_vendor(&conn, &id); }
        return Err(format!("Keyring 写入失败: {}", e));
    }
    Ok(v)
}

#[derive(serde::Deserialize)]
pub struct UpdateVendorInput { pub id: String, pub name: String, pub api_base: String, pub model: String, pub api_key: Option<String> }

#[tauri::command]
pub fn update_vendor(state: State<AppState>, input: UpdateVendorInput) -> Result<VendorInstance, String> {
    let UpdateVendorInput { id, name, api_base, model, api_key } = input;
    let (updated, is_active) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let mut existing = db::get_vendor(&conn, &id).map_err(|e| e.to_string())?.ok_or_else(|| "vendor not found".to_string())?;
        existing.name = name; existing.api_base = api_base; existing.model = model; existing.updated_at = now_ts();
        if let Some(ref key) = api_key { if !key.is_empty() { keyring_store::set_key(KEYRING_SERVICE, &existing.keyring_key, key).map_err(|e| format!("Keyring 写入失败: {}", e))?; } }
        db::update_vendor(&conn, &existing).map_err(|e| e.to_string())?;
        let active = conn.query_row("SELECT value FROM settings WHERE key='active_vendor'", [], |r| r.get::<_, String>(0)).ok();
        (existing, active.as_deref() == Some(&id))
    };
    if is_active {
        let api_key = keyring_store::get_key(KEYRING_SERVICE, &updated.keyring_key).map_err(|e| format!("Keyring 读取失败: {}", e))?;
        let path = state.config_path.lock().map_err(|e| e.to_string())?.clone();
        let provider_id = updated.preset_id.clone().unwrap_or_else(|| updated.name.clone()).to_lowercase().replace(' ', "-");
        minimax_config::apply_provider(&path, &provider_id, &updated.name, &updated.api_base, &updated.model, &api_key).map_err(|e| format!("配置文件写入失败: {}", e))?;
    }
    Ok(updated)
}

#[tauri::command]
pub fn delete_vendor(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let v = db::get_vendor(&conn, &id).map_err(|e| e.to_string())?.ok_or_else(|| "vendor not found".to_string())?;
    db::delete_vendor(&conn, &id).map_err(|e| e.to_string())?;
    if let Err(e) = keyring_store::delete_key(KEYRING_SERVICE, &v.keyring_key) { return Err(format!("Keyring 清理失败: {}", e)); }
    let _ = conn.execute("DELETE FROM settings WHERE key='active_vendor' AND value=?1", [&id]);
    Ok(())
}

#[tauri::command]
pub fn apply_vendor(state: State<AppState>, id: String) -> Result<(), String> {
    let (vendor, api_key) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let v = db::get_vendor(&conn, &id).map_err(|e| e.to_string())?.ok_or_else(|| "vendor not found".to_string())?;
        let key = keyring_store::get_key(KEYRING_SERVICE, &v.keyring_key).map_err(|e| format!("Keyring 读取失败: {}", e))?;
        (v, key)
    };
    let path = state.config_path.lock().map_err(|e| e.to_string())?.clone();
    let provider_id = vendor.preset_id.clone().unwrap_or_else(|| vendor.name.clone()).to_lowercase().replace(' ', "-");
    minimax_config::apply_provider(&path, &provider_id, &vendor.name, &vendor.api_base, &vendor.model, &api_key).map_err(|e| format!("MiniMax 配置写入失败: {}", e))?;
    { let conn = state.db.lock().map_err(|e| e.to_string())?; conn.execute("INSERT OR REPLACE INTO settings (key,value) VALUES ('active_vendor',?1)", [&id]).map_err(|e| e.to_string())?; }
    Ok(())
}

fn get_active_vendor_inner(conn: &Connection) -> Result<Option<String>, String> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key='active_vendor'").map_err(|e| e.to_string())?;
    let mut iter = stmt.query_map([], |row| row.get::<_, String>(0)).map_err(|e| e.to_string())?;
    Ok(iter.next().transpose().map_err(|e| e.to_string())?)
}

#[tauri::command]
pub fn get_active_vendor(state: State<AppState>) -> Result<Option<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?; get_active_vendor_inner(&conn)
}

#[tauri::command]
pub fn launch_claude_cmd() -> Result<u32, String> { launcher::launch_claude().map_err(|e| format!("启动失败: {}", e)) }

#[tauri::command]
pub fn is_claude_installed() -> bool { launcher::find_claude().is_some() }

// ===== 新命令 =====

#[tauri::command]
pub fn detect_installed_tools(state: State<AppState>) -> Result<Vec<detector::DetectionResult>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?; let tools = db::list_tools(&conn).map_err(|e| e.to_string())?; Ok(detector::detect_all_tools(&tools))
}

#[tauri::command]
pub fn list_tools(state: State<AppState>) -> Result<Vec<db::Tool>, String> { let conn = state.db.lock().map_err(|e| e.to_string())?; db::list_tools(&conn).map_err(|e| e.to_string()) }

#[tauri::command]
pub fn list_providers(state: State<AppState>) -> Result<Vec<db::Provider>, String> {
    let providers = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::list_providers(&conn).map_err(|e| e.to_string())?
    };
    providers
        .into_iter()
        .map(|provider| hydrate_provider(&state, provider))
        .collect()
}

#[tauri::command]
pub fn list_models(state: State<AppState>) -> Result<Vec<db::Model>, String> { let conn = state.db.lock().map_err(|e| e.to_string())?; db::list_models(&conn).map_err(|e| e.to_string()) }

#[derive(serde::Deserialize)]
pub struct CreateProviderInput { pub id: String, pub name: String, pub api_base: String, pub anthropic_mode: bool, pub api_key: Option<String> }

#[tauri::command]
pub fn create_provider(state: State<AppState>, input: CreateProviderInput) -> Result<db::Provider, String> {
    let CreateProviderInput { id, name, api_base, anthropic_mode, api_key } = input;
    let p = db::Provider { id: id.clone(), name, api_base, anthropic_mode, has_api_key: false, created_at: now_ts(), updated_at: now_ts() };
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::insert_provider(&conn, &p).map_err(|e| e.to_string())?;
    drop(conn);
    if let Some(key) = api_key.as_deref().filter(|k| !k.is_empty()) {
        let account = provider_account(&p.id);
        if let Err(e) = keyring_store::set_key(KEYRING_SERVICE, &account, key) {
            let conn = state.db.lock().map_err(|err| err.to_string())?;
            let _ = db::delete_provider(&conn, &id);
            return Err(format!("Keyring 写入失败: {}", e));
        }
    }
    hydrate_provider(&state, p)
}

#[tauri::command]
pub fn delete_provider(state: State<AppState>, id: String) -> Result<(), String> {
    let (provider_key, binding_accounts) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let provider_key = provider_account(&id);
        let bindings = db::list_bindings_by_provider(&conn, &id).map_err(|e| e.to_string())?;
        let binding_accounts = bindings
            .into_iter()
            .filter_map(|b| b.keyring_key)
            .collect::<Vec<_>>();
        (provider_key, binding_accounts)
    };

    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::delete_provider_cascade(&conn, &id).map_err(|e| e.to_string())?;
    }

    let mut keyring_errors = Vec::new();
    if let Err(e) = keyring_store::delete_key(KEYRING_SERVICE, &provider_key) {
        keyring_errors.push(format!("provider key: {}", e));
    }
    for account in binding_accounts {
        if let Err(e) = keyring_store::delete_key(KEYRING_SERVICE, &account) {
            keyring_errors.push(format!("binding {}: {}", account, e));
        }
    }

    if keyring_errors.is_empty() {
        Ok(())
    } else {
        Err(format!("厂商已删除，但部分 Keyring 清理失败: {}", keyring_errors.join("; ")))
    }
}

#[tauri::command]
pub fn apply_binding(state: State<AppState>, tool_id: String, provider_id: String, model_id: String) -> Result<(), String> {
    let model_name = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let models = db::list_models_by_provider(&conn, &provider_id).map_err(|e| e.to_string())?;
        models.into_iter().find(|m| m.id == model_id).map(|m| m.model_id).ok_or_else(|| "模型未找到".to_string())?
    };
    let api_key = fetch_provider_key(&state, &provider_id)?;
    let previous_bindings = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::list_bindings_by_tool(&conn, &tool_id).map_err(|e| e.to_string())?
    };
    let binding_id = uuid::Uuid::new_v4().to_string();
    let keyring_key = binding_account(&binding_id);
    keyring_store::set_key(KEYRING_SERVICE, &keyring_key, &api_key).map_err(|e| format!("Keyring 写入失败: {}", e))?;
    for binding in previous_bindings {
        if let Some(account) = binding.keyring_key {
            let _ = keyring_store::delete_key(KEYRING_SERVICE, &account);
        }
    }
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::delete_bindings_by_tool(&conn, &tool_id).map_err(|e| e.to_string())?;
    }
    let now = now_ts();
    let binding = db::ToolBinding { id: binding_id.clone(), tool_id: tool_id.clone(), provider_id: provider_id.clone(), model_id: model_id.clone(), keyring_key: Some(keyring_key), is_active: true, created_at: now, updated_at: now };
    { let conn = state.db.lock().map_err(|e| e.to_string())?; db::set_active_binding(&conn, &binding_id, &tool_id).map_err(|e| e.to_string())?; db::upsert_binding(&conn, &binding).map_err(|e| e.to_string())?; }
    apply_config_for_tool(&state, &tool_id, &provider_id, &model_name, &api_key)?;
    Ok(())
}

fn fetch_provider_key(state: &State<AppState>, provider_id: &str) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let account = provider_account(provider_id);
    match keyring_store::get_key(KEYRING_SERVICE, &account) {
        Ok(key) if !key.trim().is_empty() => Ok(key),
        _ => {
            let legacy = db::get_provider_legacy_api_key(&conn, provider_id).map_err(|e| e.to_string())?;
            let key = legacy.filter(|k| !k.trim().is_empty()).ok_or_else(|| {
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

fn apply_config_for_tool(state: &State<AppState>, tool_id: &str, provider_id: &str, model_name: &str, api_key: &str) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let provider = db::list_providers(&conn).map_err(|e| e.to_string())?.into_iter().find(|p| p.id == provider_id).ok_or_else(|| format!("厂商未找到: {}", provider_id))?;
    drop(conn);
    match tool_id {
        "minimax-code-cli" | "minimax-code-desktop" => { let path = state.config_path.lock().map_err(|e| e.to_string())?.clone(); minimax_config::apply_provider(&path, provider_id, &provider.name, &provider.api_base, model_name, api_key).map_err(|e| format!("MiniMax 配置写入失败: {}", e))?; }
        "claude-code-cli" => {
            let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from).ok_or_else(|| "无法找到用户目录".to_string())?;
            let path = home.join(".claude").join("settings.json");
            let mut updates = std::collections::HashMap::new();
            updates.insert("ANTHROPIC_BASE_URL".to_string(), provider.api_base.clone());
            updates.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
            updates.insert("ANTHROPIC_MODEL".to_string(), model_name.to_string());
            claude_config::merge_and_write_env(&path, &updates)
                .map_err(|e| format!("Claude 配置写入失败: {}", e))?;
        }
        "claude-desktop" => {
            let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from).ok_or_else(|| "无法找到用户目录".to_string())?;
            let path = home.join(".claude").join("settings.json");
            let mut updates = std::collections::HashMap::new();
            updates.insert("ANTHROPIC_BASE_URL".to_string(), provider.api_base.clone());
            updates.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
            updates.insert("ANTHROPIC_MODEL".to_string(), model_name.to_string());
            claude_config::merge_and_write_env(&path, &updates)
                .map_err(|e| format!("Claude 桌面端配置写入失败: {}", e))?;
        }
        "codex-cli" => crate::tool_configs::apply_codex(
            provider_id,
            &provider.name,
            &provider.api_base,
            model_name,
            api_key,
        ).map_err(|e| format!("Codex 配置写入失败: {}", e))?,
        "codex-desktop" => crate::tool_configs::apply_codex(
            provider_id,
            &provider.name,
            &provider.api_base,
            model_name,
            api_key,
        ).map_err(|e| format!("Codex 桌面端配置写入失败: {}", e))?,
        "opencode-cli" => crate::tool_configs::apply_opencode(
            provider_id,
            &provider.name,
            &provider.api_base,
            model_name,
            api_key,
            provider.anthropic_mode,
        ).map_err(|e| format!("OpenCode 配置写入失败: {}", e))?,
        "qwen-code-cli" => crate::tool_configs::apply_qwen(
            provider_id,
            &provider.name,
            &provider.api_base,
            model_name,
            api_key,
            provider.anthropic_mode,
        ).map_err(|e| format!("Qwen 配置写入失败: {}", e))?,
        "aider-cli" => crate::tool_configs::apply_aider(
            &provider.api_base,
            model_name,
            api_key,
            provider.anthropic_mode,
        ).map_err(|e| format!("Aider 配置写入失败: {}", e))?,
        "grok-build" => crate::tool_configs::apply_grok(
            provider_id,
            &provider.name,
            &provider.api_base,
            model_name,
            api_key,
            provider.anthropic_mode,
        ).map_err(|e| format!("Grok 配置写入失败: {}", e))?,
        "kimi-cli" => crate::tool_configs::apply_kimi(
            provider_id,
            &provider.name,
            &provider.api_base,
            model_name,
            api_key,
            provider.anthropic_mode,
        ).map_err(|e| format!("Kimi 配置写入失败: {}", e))?,
        "openclaw" => crate::agent_adapters::apply_openclaw(&provider.name, &provider.api_base, model_name, api_key).map_err(|e| format!("OpenClaw 配置写入失败: {}", e))?,
        "hermes-agent" => crate::agent_adapters::apply_hermes(provider_id, &provider.api_base, model_name, api_key).map_err(|e| format!("Hermes Agent 配置写入失败: {}", e))?,
        "nanobot" => crate::agent_adapters::apply_nanobot(provider_id, &provider.api_base, model_name, api_key).map_err(|e| format!("Nanobot 配置写入失败: {}", e))?,
        _ => return Err(format!("暂不支持的工具: {}", tool_id)),
    }
    Ok(())
}

#[tauri::command]
pub fn launch_tool(state: State<AppState>, tool_id: String) -> Result<u32, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let tool = db::get_tool(&conn, &tool_id).map_err(|e| e.to_string())?.ok_or_else(|| format!("工具未找到: {}", tool_id))?;
    drop(conn);
    if tool.id == "minimax-code-desktop" {
        return launcher::launch_minimax_desktop().map_err(|e| format!("启动失败: {}", e));
    }
    if tool.id == "claude-desktop" {
        return launcher::launch_claude_desktop().map_err(|e| format!("启动失败: {}", e));
    }
    if tool.id == "codex-desktop" {
        return launcher::launch_codex_desktop().map_err(|e| format!("启动失败: {}", e));
    }
    if tool.id == "gemini-desktop" {
        return launcher::launch_gemini_desktop().map_err(|e| format!("启动失败: {}", e));
    }
    if let Some(ref cmd) = tool.launch_command {
        let binary = launcher::find_cli_command(cmd).unwrap_or_else(|| PathBuf::from(cmd));
        return Ok(std::process::Command::new(binary).spawn().map_err(|e| format!("启动失败: {}", e))?.id());
    }
    if let Some(ref path) = tool.launch_path { return Ok(std::process::Command::new(path).spawn().map_err(|e| format!("启动失败: {}", e))?.id()); }
    Err(format!("{} 没有配置启动方式", tool.name))
}

#[tauri::command]
pub fn get_tool_binding(state: State<AppState>, tool_id: String) -> Result<Option<serde_json::Value>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let binding = db::get_active_binding(&conn, &tool_id).map_err(|e| e.to_string())?;
    let Some(b) = binding else { return Ok(None) };
    let provider = db::list_providers(&conn).map_err(|e| e.to_string())?.into_iter().find(|p| p.id == b.provider_id);
    let model = db::list_models(&conn).map_err(|e| e.to_string())?.into_iter().find(|m| m.id == b.model_id);
    Ok(Some(serde_json::json!({ "id": b.id, "tool_id": b.tool_id, "provider_id": b.provider_id, "provider_name": provider.as_ref().map(|p| p.name.as_str()), "model_id": b.model_id, "model_name": model.as_ref().map(|m| m.name.as_str()), "is_active": b.is_active })))
}

#[tauri::command]
pub fn unbind_tool(state: State<AppState>, binding_id: String) -> Result<(), String> {
    let binding = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::list_bindings(&conn)
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|b| b.id == binding_id)
    };

    let Some(binding) = binding else { return Ok(()); };

    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::delete_binding(&conn, &binding_id).map_err(|e| e.to_string())?;
    }

    if let Some(account) = binding.keyring_key {
        keyring_store::delete_key(KEYRING_SERVICE, &account)
            .map_err(|e| format!("Keyring 清理失败: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn update_provider(state: State<AppState>, input: CreateProviderInput) -> Result<db::Provider, String> {
    let CreateProviderInput { id, name, api_base, anthropic_mode, api_key } = input;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut provider = db::list_providers(&conn).map_err(|e| e.to_string())?.into_iter().find(|p| p.id == id).ok_or_else(|| "厂商未找到".to_string())?;
    provider.name = name; provider.api_base = api_base; provider.anthropic_mode = anthropic_mode; provider.updated_at = now_ts();
    if let Some(key) = api_key.as_deref().filter(|k| !k.is_empty()) {
        let account = provider_account(&provider.id);
        keyring_store::set_key(KEYRING_SERVICE, &account, key).map_err(|e| format!("Keyring 写入失败: {}", e))?;
    }
    db::update_provider(&conn, &provider).map_err(|e| e.to_string())?;
    hydrate_provider(&state, provider)
}

#[derive(serde::Deserialize)]
pub struct CreateModelInput { pub provider_id: String, pub name: String, pub model_id: String, pub context_length: i64, pub max_output: i64 }

#[tauri::command]
pub fn create_model(state: State<AppState>, input: CreateModelInput) -> Result<db::Model, String> {
    let id = format!("{}/{}", input.provider_id, input.model_id); let now = now_ts();
    let m = db::Model { id, provider_id: input.provider_id, name: input.name, model_id: input.model_id, context_length: input.context_length, max_output: input.max_output, supports_attachment: false, supports_reasoning: true, supports_tool_call: true, supports_vision: false, created_at: now, updated_at: now };
    let conn = state.db.lock().map_err(|e| e.to_string())?; db::insert_model(&conn, &m).map_err(|e| e.to_string())?; Ok(m)
}

#[derive(serde::Deserialize)]
pub struct UpdateModelInput {
    pub id: String,
    pub provider_id: String,
    pub name: String,
    pub model_id: String,
    pub context_length: i64,
    pub max_output: i64,
}

#[tauri::command]
pub fn update_model(state: State<AppState>, input: UpdateModelInput) -> Result<db::Model, String> {
    let now = now_ts();
    let m = db::Model {
        id: input.id,
        provider_id: input.provider_id,
        name: input.name,
        model_id: input.model_id,
        context_length: input.context_length,
        max_output: input.max_output,
        supports_attachment: false,
        supports_reasoning: true,
        supports_tool_call: true,
        supports_vision: false,
        created_at: now,
        updated_at: now,
    };
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::update_model(&conn, &m).map_err(|e| e.to_string())?;
    Ok(m)
}

#[tauri::command]
pub fn delete_model(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_model(&conn, &id).map_err(|e| e.to_string())
}

// ===== AI 对话 =====

#[derive(serde::Deserialize)]
pub struct ChatInput { pub messages: Vec<ChatMsgInput>, pub api_base: String, pub api_key: String, pub model: String, pub anthropic_mode: bool }
#[derive(serde::Deserialize)]
pub struct ChatMsgInput { pub role: String, pub content: String }

#[tauri::command]
pub async fn chat_send(input: ChatInput) -> Result<crate::llm_chat::ChatResponse, String> {
    crate::llm_chat::chat_complete(crate::llm_chat::ChatRequest { messages: input.messages.into_iter().map(|m| crate::llm_chat::ChatMessage { role: m.role, content: m.content }).collect(), api_base: input.api_base, api_key: input.api_key, model: input.model, anthropic_mode: input.anthropic_mode, max_tokens: Some(4096) }).await
}

// ===== 一键安装 =====

#[tauri::command]
pub fn get_install_info(tool_id: String) -> Result<Option<crate::installer::InstallInfo>, String> { Ok(crate::installer::get_install_info(&tool_id)) }
#[tauri::command]
pub fn install_tool(tool_id: String) -> Result<String, String> { crate::installer::run_install(&tool_id) }
