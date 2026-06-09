use super::AppState;
use crate::db;
use crate::detector;
use crate::keyring_store;
use crate::launcher;
use crate::llm_chat;
use crate::installer;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use tauri::State;

#[cfg(windows)]
const CREATE_NEW_CONSOLE: u32 = 0x00000010;

/// 尝试启动进程，如果遇到 740 (需要提权) 则自动用 PowerShell 提权
fn spawn_or_elevate(
    cmd: &mut std::process::Command,
) -> Result<std::process::Child, String> {
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NEW_CONSOLE);
    }
    match cmd.spawn() {
        Ok(child) => Ok(child),
        Err(e) => {
            #[cfg(windows)]
            if e.raw_os_error() == Some(740) {
                let prog = cmd.get_program().to_string_lossy().to_string();
                // 安全：用 -FilePath 作为独立参数传递，避免字符串拼接注入
                let ps_child = std::process::Command::new("powershell")
                    .args([
                        "-NoProfile",
                        "-WindowStyle", "Hidden",
                        "-Command",
                        "Start-Process",
                        "-FilePath", &prog,
                        "-Verb", "RunAs",
                    ])
                    .spawn()
                    .map_err(|e| {
                        format!(
                            "请求管理员权限失败: {}。可尝试右键以管理员身份运行观景。",
                            e
                        )
                    })?;
                return Ok(ps_child);
            }
            Err(format!("启动失败: {}", e))
        }
    }
}

// ===== Tool detection & listing =====

#[tauri::command]
pub fn detect_installed_tools(
    state: State<AppState>,
) -> Result<Vec<detector::DetectionResult>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let tools = db::list_tools(&conn).map_err(|e| e.to_string())?;
    Ok(detector::detect_all_tools(&tools))
}

#[tauri::command]
pub fn list_tools(state: State<AppState>) -> Result<Vec<db::Tool>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::list_tools(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn launch_tool(state: State<AppState>, tool_id: String) -> Result<u32, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let tool = db::get_tool(&conn, &tool_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("工具未找到: {}", tool_id))?;

    let binding = db::get_active_binding(&conn, &tool_id).map_err(|e| e.to_string())?;
    let provider_info: Option<(String, String, bool, String)> = binding.as_ref().and_then(|b| {
        let provider = db::get_provider(&conn, &b.provider_id).ok()??;
        let key = keyring_store::get_key(
            super::KEYRING_SERVICE,
            b.keyring_key.as_ref()?,
        )
        .ok()?;
        let model = db::list_models_by_provider(&conn, &b.provider_id)
            .ok()?
            .into_iter()
            .find(|m| m.id == b.model_id)?;
        Some((
            provider.api_base,
            key,
            provider.anthropic_mode,
            model.model_id,
        ))
    });
    drop(conn);

    // 桌面版——优先注入环境变量
    if tool.id == "codex-desktop" {
        if let Some((base_url, api_key, anthropic_mode, _model)) = provider_info {
            let exe = launcher::find_codex_desktop().ok_or("未找到 Codex Desktop")?;
            let mut cmd = std::process::Command::new(&exe);
            if anthropic_mode {
                cmd.env("ANTHROPIC_BASE_URL", &base_url);
                cmd.env("ANTHROPIC_AUTH_TOKEN", &api_key);
            } else {
                cmd.env("OPENAI_API_KEY", &api_key);
                cmd.env("OPENAI_BASE_URL", &base_url);
            }
            let child = spawn_or_elevate(&mut cmd)?;
            return Ok(child.id());
        }
        return launcher::launch_codex_desktop().map_err(|e| format!("启动失败: {}", e));
    }
    if tool.id == "minimax-code-desktop" {
        if let Some((base_url, api_key, anthropic_mode, _model)) = provider_info {
            let exe = launcher::find_minimax_desktop().ok_or("未找到 MiniMax")?;
            let mut cmd = std::process::Command::new(&exe);
            if anthropic_mode {
                cmd.env("ANTHROPIC_BASE_URL", &base_url);
                cmd.env("ANTHROPIC_AUTH_TOKEN", &api_key);
            }
            let child = spawn_or_elevate(&mut cmd)?;
            return Ok(child.id());
        }
        return launcher::launch_minimax_desktop().map_err(|e| format!("启动失败: {}", e));
    }
    if tool.id == "claude-desktop" {
        if let Some((base_url, api_key, anthropic_mode, _model)) = provider_info {
            let exe = launcher::find_claude_desktop().ok_or("未找到 Claude")?;
            let mut cmd = std::process::Command::new(&exe);
            if anthropic_mode {
                cmd.env("ANTHROPIC_BASE_URL", &base_url);
                cmd.env("ANTHROPIC_AUTH_TOKEN", &api_key);
            }
            let child = spawn_or_elevate(&mut cmd)?;
            return Ok(child.id());
        }
        return launcher::launch_claude_desktop().map_err(|e| format!("启动失败: {}", e));
    }
    if tool.id == "gemini-desktop" {
        return launcher::launch_gemini_desktop().map_err(|e| format!("启动失败: {}", e));
    }

    // CLI 工具：注入环境变量启动
    if let Some(ref cmd_name) = tool.launch_command {
        let binary =
            launcher::find_cli_command(cmd_name).unwrap_or_else(|| PathBuf::from(cmd_name));

        if let Some((base_url, api_key, anthropic_mode, model)) = provider_info {
            let mut cmd = std::process::Command::new(&binary);
            if anthropic_mode {
                cmd.env("ANTHROPIC_BASE_URL", &base_url);
                cmd.env("ANTHROPIC_AUTH_TOKEN", &api_key);
                cmd.env("ANTHROPIC_MODEL", &model);
            } else {
                cmd.env("OPENAI_API_KEY", &api_key);
                cmd.env("OPENAI_BASE_URL", &base_url);
                cmd.env("OPENAI_MODEL", &model);
            }
            #[cfg(windows)]
            {
                cmd.creation_flags(CREATE_NEW_CONSOLE);
            }
            let child = spawn_or_elevate(&mut cmd)?;
            return Ok(child.id());
        }

        let mut cmd = std::process::Command::new(&binary);
        return Ok(spawn_or_elevate(&mut cmd)?.id());
    }

    if let Some(ref path) = tool.launch_path {
        let mut cmd = std::process::Command::new(path);
        return Ok(spawn_or_elevate(&mut cmd)?.id());
    }

    Err(format!("{} 没有配置启动方式", tool.name))
}

// ===== Chat =====

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
pub async fn chat_send(
    input: ChatInput,
) -> Result<llm_chat::ChatResponse, String> {
    llm_chat::chat_complete(llm_chat::ChatRequest {
        messages: input
            .messages
            .into_iter()
            .map(|m| llm_chat::ChatMessage {
                role: m.role,
                content: m.content,
            })
            .collect(),
        api_base: input.api_base,
        api_key: input.api_key,
        model: input.model,
        anthropic_mode: input.anthropic_mode,
        max_tokens: Some(4096),
    })
    .await
}

// ===== Install =====

#[tauri::command]
pub fn get_install_info(
    tool_id: String,
) -> Result<Option<installer::InstallInfo>, String> {
    Ok(installer::get_install_info(&tool_id))
}

#[tauri::command]
pub fn install_tool(tool_id: String) -> Result<String, String> {
    installer::run_install(&tool_id)
}
