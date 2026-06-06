use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

/// 检测桌面端应用
pub fn detect_desktop(exe_name: &str) -> Option<PathBuf> {
    // 常见安装目录
    let candidates = vec![
        // 桌面快捷方式目录
        dirs_home().map(|h| h.join("Desktop").join("ai编程").join(exe_name).join(format!("{}.exe", exe_name))),
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
        let mut install_type = String::new();
        let mut versions = Vec::new();

        // 检测 CLI
        for cmd in &path_cmds {
            if !cmd.is_empty() && detect_cli(cmd) {
                installed = true;
                if !install_type.is_empty() { install_type.push_str("+"); }
                install_type.push_str("cli");
                versions.push(format!("cli:{}", cmd));
            }
        }

        // 检测桌面端
        for file in &files {
            if !file.is_empty() {
                if let Some(path) = detect_desktop(file.trim_end_matches(".exe")) {
                    installed = true;
                    if !install_type.is_empty() { install_type.push_str("+"); }
                    install_type.push_str("desktop");
                    versions.push(format!("desktop:{}", path.display()));
                }
            }
        }

        // 也检测 config_path
        if let Some(ref config_path) = tool.config_path {
            let expanded = expand_path(config_path);
            if expanded.exists() {
                // 配置文件存在但不一定说明工具已安装
                // 不改变 installed 状态，只是记录
            }
        }

        if !installed {
            install_type = "none".to_string();
        }

        DetectionResult {
            tool_id: tool.id.clone(),
            tool_name: tool.name.clone(),
            installed,
            install_type,
            versions,
        }
    }).collect()
}
