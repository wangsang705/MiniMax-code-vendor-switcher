use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::launcher;

/// 检测结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DetectionResult {
    pub tool_id: String,
    pub tool_name: String,
    pub installed: bool,
    pub install_type: String,  // 'cli', 'desktop', 'both'
    pub versions: Vec<String>,
}

/// 检测 PATH 中是否包含指定命令
pub fn detect_cli(command: &str) -> bool {
    which(command).is_some()
}

/// 检测文件是否存在
pub fn detect_file(path: &str) -> bool {
    let expanded = expand_path(path);
    expanded.is_file()
}

/// 检测桌面端应用（注册表优先，文件系统路径兜底）
pub fn detect_desktop(exe_name: &str) -> Option<PathBuf> {
    // 先尝试注册表检测
    #[cfg(windows)]
    {
        if let Some(path) = crate::registry::detect_desktop_via_registry(exe_name, exe_name) {
            return Some(path);
        }
        // 去掉 .exe 后缀再试一次
        let name_no_ext = exe_name.trim_end_matches(".exe");
        if let Some(path) = crate::registry::detect_desktop_via_registry(name_no_ext, exe_name) {
            return Some(path);
        }
    }

    // 再尝试专用查找函数
    if exe_name == "MiniMax Code" || exe_name == "MiniMax Code.exe" {
        return launcher::find_minimax_desktop();
    }
    if exe_name == "Claude" || exe_name == "Claude.exe" {
        return launcher::find_claude_desktop();
    }
    if exe_name == "Codex" || exe_name == "Codex.exe" {
        return launcher::find_codex_desktop();
    }
    if exe_name == "Gemini" || exe_name == "Gemini.exe" {
        return launcher::find_gemini_desktop();
    }

    // 常见安装目录
    let candidates = vec![
        // 桌面快捷方式目录
        dirs_home().map(|h| h.join("Desktop").join(exe_name).join(format!("{}.exe", exe_name))),
        // 标准 Program Files
        Some(PathBuf::from(r"C:\Program Files").join(exe_name).join(format!("{}.exe", exe_name))),
        Some(PathBuf::from(r"C:\Program Files (x86)").join(exe_name).join(format!("{}.exe", exe_name))),
        // 用户 AppData
        dirs_home().map(|h| h.join("AppData").join("Local").join("Programs").join(exe_name).join(format!("{}.exe", exe_name))),
        dirs_home().map(|h| h.join("AppData").join("Local").join(exe_name).join(format!("{}.exe", exe_name))),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// 在 PATH 中查找命令
fn which(cmd: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    #[cfg(windows)]
    let extensions = [".exe", ".cmd", ".bat", ""];
    #[cfg(not(windows))]
    let extensions = [""];

    for dir in std::env::split_paths(&path) {
        for ext in &extensions {
            let candidate = dir.join(format!("{}{}", cmd, ext));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn launch_path_exists(path: &str) -> bool {
    let expanded = expand_path(path);
    expanded.is_file()
}

/// 展开 ~ 为用户目录
fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") || path == "~" {
        if let Some(home) = dirs_home() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

fn dirs_home() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

/// 检测所有已注册工具
pub fn detect_all_tools(tools: &[crate::db::Tool]) -> Vec<DetectionResult> {
    tools.iter().map(|tool| {
        let path_cmds: Vec<String> = serde_json::from_str(&tool.detection_path_cmds).unwrap_or_default();
        let files: Vec<String> = serde_json::from_str(&tool.detection_files).unwrap_or_default();

        let mut installed = false;
        let mut versions = Vec::new();
        let mut has_cli = false;
        let mut has_desktop = false;

        // 检测 CLI（先用 PATH 找，再用 launcher 的 fallback 路径）
        for cmd in &path_cmds {
            if !cmd.is_empty() {
                let found = detect_cli(cmd)
                    || crate::launcher::find_cli_command(cmd).is_some();
                if found {
                    has_cli = true;
                    installed = true;
                    versions.push(format!("cli:{}", cmd));
                }
            }
        }

        // 检测桌面端：launch_path
        if let Some(ref launch_path) = tool.launch_path {
            if launch_path_exists(launch_path) {
                has_desktop = true;
                versions.push(format!("desktop:{}", expand_path(launch_path).display()));
            }
        }

        // 检测桌面端：detection_files
        for file in &files {
            if !file.is_empty() {
                if let Some(path) = detect_desktop(file.trim_end_matches(".exe")) {
                    has_desktop = true;
                    installed = true;
                    if !versions.iter().any(|v| v.starts_with("desktop:")) {
                        versions.push(format!("desktop:{}", path.display()));
                    }
                }
            }
        }

        // 也检测 config_path（仅记录，不影响安装状态）
        if let Some(ref config_path) = tool.config_path {
            let expanded = expand_path(config_path);
            if expanded.exists() {
                // 不改变 installed 状态
            }
        }

        let install_type = if has_cli && has_desktop {
            "both"
        } else if has_cli {
            "cli"
        } else if has_desktop {
            "desktop"
        } else {
            "none"
        }.to_string();

        DetectionResult {
            tool_id: tool.id.clone(),
            tool_name: tool.name.clone(),
            installed,
            install_type,
            versions,
        }
    }).collect()
}
