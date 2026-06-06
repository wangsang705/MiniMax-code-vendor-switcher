use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 安装方式
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InstallMethod {
    Npm { package: String },
    Curl { url: String },
    Pip { package: String },
    Download { url: String, filename: String },
    Manual { guide: String },
}

/// 安装信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallInfo {
    pub tool_id: String,
    pub methods: Vec<InstallMethod>,
    pub description: String,
}

/// 获取工具的安装信息
pub fn get_install_info(tool_id: &str) -> Option<InstallInfo> {
    let make = |desc: &str, methods: Vec<InstallMethod>| -> InstallInfo {
        InstallInfo {
            tool_id: tool_id.to_string(),
            description: desc.to_string(),
            methods,
        }
    };

    Some(match tool_id {
        "claude-code-cli" => make("Anthropic 官方 AI 编码 CLI", vec![
            InstallMethod::Npm { package: "@anthropic-ai/claude-code".to_string() },
            InstallMethod::Manual { guide: "详见 https://docs.anthropic.com/en/docs/claude-code".to_string() },
        ]),
        "minimax-code-cli" => make("MiniMax 官方 AI 编码 CLI", vec![
            InstallMethod::Manual { guide: "通过 MiniMax Code 桌面版安装包安装后，CLI 会自动可用".to_string() },
        ]),
        "minimax-code-desktop" => make("MiniMax Code 桌面应用", vec![
            InstallMethod::Download {
                url: "https://www.minimaxi.com/download".to_string(),
                filename: "MiniMax Code Setup.exe".to_string(),
            },
        ]),
        "codex-cli" => make("OpenAI Codex CLI", vec![
            InstallMethod::Npm { package: "@openai/codex".to_string() },
            InstallMethod::Manual { guide: "详见 https://github.com/openai/codex".to_string() },
        ]),
        "qwen-code-cli" => make("通义千问 Code CLI", vec![
            InstallMethod::Npm { package: "@qwen/code".to_string() },
            InstallMethod::Manual { guide: "详见 https://github.com/Qwen/Qwen-code".to_string() },
        ]),
        "opencode-cli" => make("OpenCode CLI", vec![
            InstallMethod::Npm { package: "@opencode-ai/cli".to_string() },
            InstallMethod::Manual { guide: "详见 https://github.com/opencode-ai".to_string() },
        ]),
        "kimi-cli" => make("月之暗面 Kimi CLI", vec![
            InstallMethod::Manual { guide: "详见 Kimi 官方文档".to_string() },
        ]),
        "openclaw" => make("OpenClaw AI Agent 框架", vec![
            InstallMethod::Curl { url: "https://raw.githubusercontent.com/openclaw/openclaw/main/scripts/install.sh".to_string() },
            InstallMethod::Manual { guide: "详见 https://docs.openclaw.ai".to_string() },
        ]),
        "hermes-agent" => make("Hermes AI Agent（Nous Research）", vec![
            InstallMethod::Curl { url: "https://raw.githubusercontent.com/NousResearch/hermes-agent/main/scripts/install.sh".to_string() },
            InstallMethod::Manual { guide: "详见 https://github.com/NousResearch/hermes-agent".to_string() },
        ]),
        "nanobot" => make("Nanobot AI Agent", vec![
            InstallMethod::Npm { package: "nanobot".to_string() },
            InstallMethod::Manual { guide: "详见 https://github.com/nanobot-ai/nanobot".to_string() },
        ]),
        _ => return None,
    })
}

/// 执行安装命令
pub fn run_install(tool_id: &str) -> Result<String, String> {
    let info = get_install_info(tool_id)
        .ok_or_else(|| format!("未知工具: {}", tool_id))?;

    for method in &info.methods {
        match method {
            InstallMethod::Npm { package } => {
                let output = std::process::Command::new("npm")
                    .args(["install", "-g", package])
                    .output()
                    .map_err(|e| format!("执行 npm install 失败: {}", e))?;
                if output.status.success() {
                    let log = String::from_utf8_lossy(&output.stdout);
                    return Ok(format!("✅ npm install {} 成功\n{}", package, log));
                } else {
                    let err = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("npm install 失败: {}", err));
                }
            }
            InstallMethod::Curl { url } => {
                let ps_script = format!(
                    "[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; \
                     Invoke-WebRequest -Uri '{}' -UseBasicParsing | Select-Object -ExpandProperty Content | Invoke-Expression",
                    url
                );
                let output = std::process::Command::new("powershell")
                    .args(["-NoProfile", "-Command", &ps_script])
                    .output()
                    .map_err(|e| format!("执行安装脚本失败: {}", e))?;
                if output.status.success() {
                    return Ok("✅ 安装脚本执行成功".to_string());
                } else {
                    let err = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("安装失败: {}", err));
                }
            }
            InstallMethod::Pip { package } => {
                let output = std::process::Command::new("pip")
                    .args(["install", package])
                    .output()
                    .map_err(|e| format!("执行 pip install 失败: {}", e))?;
                if output.status.success() {
                    return Ok(format!("✅ pip install {} 成功", package));
                } else {
                    let err = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("pip install 失败: {}", err));
                }
            }
            InstallMethod::Download { url, filename } => {
                let desktop = dirs_home().map(|h| h.join("Desktop").join(filename));
                if let Some(dest) = desktop {
                    let ps_download = format!(
                        "[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; \
                         Invoke-WebRequest -Uri '{}' -OutFile '{}'",
                        url, dest.display()
                    );
                    let output = std::process::Command::new("powershell")
                        .args(["-NoProfile", "-Command", &ps_download])
                        .output()
                        .map_err(|e| format!("下载失败: {}", e))?;
                    if output.status.success() {
                        return Ok(format!("✅ 已下载到桌面: {}\n请手动运行安装程序", filename));
                    } else {
                        let err = String::from_utf8_lossy(&output.stderr);
                        return Err(format!("下载失败: {}", err));
                    }
                }
                return Err("无法找到桌面目录".to_string());
            }
            InstallMethod::Manual { guide } => {
                return Err(format!("需手动安装: {}", guide));
            }
        }
    }
    Err("没有可用的自动安装方式".to_string())
}

fn dirs_home() -> Option<PathBuf> {
    #[cfg(windows)] { std::env::var_os("USERPROFILE").map(PathBuf::from) }
    #[cfg(not(windows))] { std::env::var_os("HOME").map(PathBuf::from) }
}
