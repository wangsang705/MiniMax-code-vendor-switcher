use crate::db::{self, VendorInstance};
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

// ===================================================================
// 第二阶段：多工具支持命令
// ===================================================================

/// 检测所有已注册工具的安装状态
#[tauri::command]
pub fn detect_installed_tools(state: State<AppState>) -> Result<Vec<detector::DetectionResult>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let tools = db::list_tools(&conn).map_err(|e| e.to_string())?;
    Ok(detector::detect_all_tools(&tools))
}

/// 列出所有已注册工具
#[tauri::command]
pub fn list_tools(state: State<AppState>) -> Result<Vec<db::Tool>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::list_tools(&conn).map_err(|e| e.to_string())
}

/// 列出所有厂商
#[tauri::command]
pub fn list_providers(state: State<AppState>) -> Result<Vec<db::Provider>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::list_providers(&conn).map_err(|e| e.to_string())
}

/// 列出所有模型
#[tauri::command]
pub fn list_models(state: State<AppState>) -> Result<Vec<db::Model>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::list_models(&conn).map_err(|e| e.to_string())
}

#[derive(serde::Deserialize)]
pub struct CreateProviderInput {
    pub id: String,
    pub name: String,
    pub api_base: String,
    pub anthropic_mode: bool,
}

/// 创建新厂商
#[tauri::command]
pub fn create_provider(
    state: State<AppState>,
    input: CreateProviderInput,
) -> Result<db::Provider, String> {
    let p = db::Provider {
        id: input.id,
        name: input.name,
        api_base: input.api_base,
        anthropic_mode: input.anthropic_mode,
        created_at: now_ts(),
        updated_at: now_ts(),
    };
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::insert_provider(&conn, &p).map_err(|e| e.to_string())?;
    Ok(p)
}

/// 删除厂商
#[tauri::command]
pub fn delete_provider(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_provider(&conn, &id).map_err(|e| e.to_string())
}

/// 应用绑定（工具 + 厂商 + 模型）
#[tauri::command]
pub fn apply_binding(
    state: State<AppState>,
    tool_id: String,
    provider_id: String,
    model_id: String,
    api_key: String,
) -> Result<(), String> {
    // 保存 API Key 到 keyring
    let binding_id = uuid::Uuid::new_v4().to_string();
    let keyring_key = format!("binding:{}", binding_id);

    keyring_store::set_key(KEYRING_SERVICE, &keyring_key, &api_key)
        .map_err(|e| format!("Keyring 写入失败: {}", e))?;

    let now = now_ts();
    let binding = db::ToolBinding {
        id: binding_id.clone(),
        tool_id: tool_id.clone(),
        provider_id: provider_id.clone(),
        model_id,
        keyring_key: Some(keyring_key),
        is_active: true,
        created_at: now,
        updated_at: now,
    };

    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::set_active_binding(&conn, &binding_id, &tool_id)
            .map_err(|e| e.to_string())?;
        db::upsert_binding(&conn, &binding).map_err(|e| e.to_string())?;
    }

    // 写入工具配置文件
    apply_config_for_tool(&state, &tool_id, &provider_id, &api_key)?;

    Ok(())
}

/// 写入特定工具的配置文件
fn apply_config_for_tool(
    state: &State<AppState>,
    tool_id: &str,
    provider_id: &str,
    api_key: &str,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let tool = db::get_tool(&conn, tool_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("工具未找到: {}", tool_id))?;
    let provider = db::list_providers(&conn).map_err(|e| e.to_string())?
        .into_iter().find(|p| p.id == provider_id)
        .ok_or_else(|| format!("厂商未找到: {}", provider_id))?;
    drop(conn);

    match tool_id {
        "minimax-code-cli" | "minimax-code-desktop" => {
            let path = state.config_path.lock().map_err(|e| e.to_string())?.clone();
            let model = &provider_id; // FIXME: use actual model name
            minimax_config::apply_provider(
                &path, provider_id, &provider.name,
                &provider.api_base, model, api_key,
            ).map_err(|e| format!("MiniMax 配置写入失败: {}", e))?;
        }
        "claude-code-cli" => {
            // Claude Code: 写 ~/.claude/settings.json
            let home = std::env::var_os("USERPROFILE")
                .or_else(|| std::env::var_os("HOME"))
                .map(PathBuf::from)
                .ok_or_else(|| "无法找到用户目录".to_string())?;
            let path = home.join(".claude").join("settings.json");
            let settings = if path.exists() {
                let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
            } else {
                serde_json::json!({})
            };
            let mut settings = match settings {
                serde_json::Value::Object(map) => map,
                _ => return Err("settings.json 格式错误".to_string()),
            };
            settings.insert("env".to_string(), serde_json::json!({
                "ANTHROPIC_BASE_URL": provider.api_base,
                "ANTHROPIC_AUTH_TOKEN": api_key,
                "ANTHROPIC_MODEL": provider_id,
            }));
            let content = serde_json::to_string_pretty(&serde_json::Value::Object(settings))
                .map_err(|e| e.to_string())?;
            std::fs::write(&path, content).map_err(|e| e.to_string())?;
        }
        // -- Agent 适配器 --
        "openclaw" => {
            crate::agent_adapters::apply_openclaw(&provider.name, &provider.api_base, provider_id, api_key)
                .map_err(|e| format!("OpenClaw 配置写入失败: {}", e))?;
        }
        "hermes-agent" => {
            crate::agent_adapters::apply_hermes(provider_id, &provider.api_base, provider_id, api_key)
                .map_err(|e| format!("Hermes Agent 配置写入失败: {}", e))?;
        }
        "nanobot" => {
            crate::agent_adapters::apply_nanobot(provider_id, &provider.api_base, provider_id, api_key)
                .map_err(|e| format!("Nanobot 配置写入失败: {}", e))?;
        }
        _ => return Err(format!("暂不支持的工具: {}", tool_id)),
    }
    Ok(())
}

/// 启动指定工具
#[tauri::command]
pub fn launch_tool(state: State<AppState>, tool_id: String) -> Result<u32, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let tool = db::get_tool(&conn, &tool_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("工具未找到: {}", tool_id))?;
    drop(conn);

    // 优先用 launch_command（CLI）
    if let Some(ref cmd) = tool.launch_command {
        let child = std::process::Command::new(cmd)
            .spawn()
            .map_err(|e| format!("启动 {} 失败: {}", tool.name, e))?;
        return Ok(child.id());
    }

    // 其次用 launch_path（桌面端）
    if let Some(ref path) = tool.launch_path {
        let child = std::process::Command::new(path)
            .spawn()
            .map_err(|e| format!("启动 {} 失败: {}", tool.name, e))?;
        return Ok(child.id());
    }

    Err(format!("{} 没有配置启动方式", tool.name))
}

// ===================================================================
// AI 对话命令
// ===================================================================

#[derive(serde::Deserialize)]
pub struct ChatInput {
    pub messages: Vec<ChatMsgInput>,
    pub api_base: String,
    pub api_key: String,
    pub model: String,
    pub anthropic_mode: bool,
}

#[derive(serde::Deserialize)]
pub struct ChatMsgInput {
    pub role: String,
    pub content: String,
}

#[tauri::command]
pub async fn chat_send(input: ChatInput) -> Result<crate::llm_chat::ChatResponse, String> {
    let req = crate::llm_chat::ChatRequest {
        messages: input.messages.into_iter().map(|m| crate::llm_chat::ChatMessage {
            role: m.role,
            content: m.content,
        }).collect(),
        api_base: input.api_base,
        api_key: input.api_key,
        model: input.model,
        anthropic_mode: input.anthropic_mode,
        max_tokens: Some(4096),
    };
    crate::llm_chat::chat_complete(req).await
}

// ===================================================================
// 一键安装命令
// ===================================================================

#[tauri::command]
pub fn get_install_info(tool_id: String) -> Result<Option<crate::installer::InstallInfo>, String> {
    Ok(crate::installer::get_install_info(&tool_id))
}

#[tauri::command]
pub fn install_tool(tool_id: String) -> Result<String, String> {
    crate::installer::run_install(&tool_id)
}
