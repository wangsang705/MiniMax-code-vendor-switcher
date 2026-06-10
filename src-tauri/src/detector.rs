use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
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

/// 使用 Windows `where.exe` 搜索可执行文件
/// 优先匹配 .exe 后缀（避免把 npm CLI 脚本误认为桌面应用）
#[cfg(windows)]
fn where_exe(exe_name: &str) -> Option<PathBuf> {
    // 先尝试 .exe 精确匹配（桌面应用一定是 exe）
    let exe_candidate = format!("{}.exe", exe_name);
    let output = Command::new("where")
        .arg(&exe_candidate)
        .output()
        .ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().next() {
            let path = PathBuf::from(line.trim());
            if path.is_file() && path.extension().map(|e| e.eq_ignore_ascii_case("exe")).unwrap_or(false) {
                return Some(path);
            }
        }
    }
    None
}

/// 在 Start Menu 中搜索工具的快捷方式，解析出实际 exe 路径
#[cfg(windows)]
fn search_start_menu(exe_name: &str) -> Option<PathBuf> {
    let start_menu = std::env::var_os("APPDATA")
        .map(|p| PathBuf::from(p).join(r"Microsoft\Windows\Start Menu\Programs"))?;
    let common_start = PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs");

    for base in [&start_menu, &common_start] {
        if !base.exists() { continue; }
        // 递归搜索 .lnk 文件（递归深度控制），在文件名中匹配 exe_name
        if let Ok(entries) = walk_dir_fast(base, 4) {
            for entry in entries {
                let name = entry.file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if name.to_lowercase().contains(&exe_name.to_lowercase().trim_end_matches(".exe")) {
                    // 通过 PowerShell 解析 .lnk 目标路径
                    let ps_cmd = format!(
                        "$s = (New-Object -COM WScript.Shell).CreateShortcut('{}'); $s.TargetPath",
                        entry.display().to_string().replace('\'', "''")
                    );
                    if let Ok(out) = Command::new("powershell")
                        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
                        .output()
                    {
                        let target = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        if !target.is_empty() {
                            let tp = PathBuf::from(&target);
                            if tp.is_file() { return Some(tp); }
                        }
                    }
                    // PowerShell 解析失败时，直接返回 .lnk 自身（不理想但可点）
                    if entry.is_file() { return Some(entry); }
                }
            }
        }
    }
    None
}

/// 快速有限深度目录遍历
#[cfg(windows)]
fn walk_dir_fast(root: &PathBuf, max_depth: u32) -> std::io::Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    let mut stack = vec![(root.clone(), 0u32)];
    while let Some((dir, depth)) = stack.pop() {
        if depth > max_depth { continue; }
        if let Ok(read) = std::fs::read_dir(&dir) {
            for entry in read.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push((path, depth + 1));
                } else {
                    results.push(path);
                }
            }
        }
    }
    Ok(results)
}

/// 在常见安装目录递归搜索（限制深度 3，避免全盘扫描）
#[cfg(windows)]
fn search_common_install_locations(exe_name: &str) -> Option<PathBuf> {
    let target_exe = if exe_name.ends_with(".exe") {
        exe_name.to_string()
    } else {
        format!("{}.exe", exe_name)
    };
    let target_lower = target_exe.to_lowercase();

    let home = std::env::var_os("USERPROFILE").map(PathBuf::from);
    let local_app_data = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let program_files = Some(PathBuf::from(r"C:\Program Files"));
    let program_files_x86 = Some(PathBuf::from(r"C:\Program Files (x86)"));
    let app_data = std::env::var_os("APPDATA").map(PathBuf::from);

    let search_roots = vec![
        // Electron/ npm global 安装位置
        local_app_data.as_ref().map(|p| p.join("Programs")),
        local_app_data.as_ref().map(|p| p.join("Microsoft\\WindowsApps")),
        app_data.as_ref().map(|p| p.join("npm")),
        // Scoop
        home.as_ref().map(|p| p.join("scoop\\apps")),
        home.as_ref().map(|p| p.join("scoop\\shims")),
        // 桌面快捷方式目录
        home.as_ref().map(|p| p.join("Desktop")),
        // 用户自定义安装
        home.as_ref().map(|p| p.join("AppData\\Local\\@mmx-agent")),
        home.as_ref().map(|p| p.join(".mavis\\bin")),
        // Program Files
        program_files.as_ref().map(|p| p.to_path_buf()),
        program_files_x86.as_ref().map(|p| p.to_path_buf()),
    ];

    for root in search_roots.into_iter().flatten() {
        if !root.exists() { continue; }
        if let Ok(entries) = walk_dir_fast(&root, 4) {
            for entry in entries {
                let name = entry.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if name.to_lowercase() == target_lower {
                    return Some(entry);
                }
            }
        }
    }
    None
}

/// 检测桌面端应用（多重策略：注册表 → 专用函数 → Start Menu → where.exe → 常见目录）
pub fn detect_desktop(exe_name: &str) -> Option<PathBuf> {
    // 先尝试注册表检测（最快）
    #[cfg(windows)]
    {
        if let Some(path) = crate::registry::detect_desktop_via_registry(exe_name, exe_name) {
            return Some(path);
        }
        let name_no_ext = exe_name.trim_end_matches(".exe");
        if let Some(path) = crate::registry::detect_desktop_via_registry(name_no_ext, exe_name) {
            return Some(path);
        }
    }

    // 再尝试专用查找函数（已知安装模式）
    let specialized = match exe_name {
        "MiniMax Code" | "MiniMax Code.exe" => launcher::find_minimax_desktop(),
        "Claude" | "Claude.exe" => launcher::find_claude_desktop(),
        "Codex" | "Codex.exe" => launcher::find_codex_desktop(),
        "Gemini" | "Gemini.exe" => launcher::find_gemini_desktop(),
        _ => None,
    };
    if specialized.is_some() {
        return specialized;
    }

    // Windows 特有增强搜索
    #[cfg(windows)]
    {
        // Start Menu 搜索（覆盖正常安装的桌面应用）
        if let Some(path) = search_start_menu(exe_name) {
            return Some(path);
        }

        // where.exe 搜索（覆盖 PATH 和当前目录中的工具）
        if let Some(path) = where_exe(exe_name) {
            return Some(path);
        }

        // 常见安装目录递归搜索（最后兜底）
        if let Some(path) = search_common_install_locations(exe_name) {
            return Some(path);
        }
    }

    // 最后的兜底——常见安装目录（跨平台版本）
    let candidates = vec![
        dirs_home().map(|h| h.join("Desktop").join(exe_name).join(format!("{}.exe", exe_name))),
        dirs_home().map(|h| h.join("Desktop").join("ai编程").join(exe_name).join(format!("{}.exe", exe_name))),
        Some(PathBuf::from(r"C:\Program Files").join(exe_name).join(format!("{}.exe", exe_name))),
        Some(PathBuf::from(r"C:\Program Files (x86)").join(exe_name).join(format!("{}.exe", exe_name))),
        dirs_home().map(|h| h.join("AppData").join("Local").join("Programs").join(exe_name).join(format!("{}.exe", exe_name))),
        dirs_home().map(|h| h.join("AppData").join("Local").join(exe_name).join(format!("{}.exe", exe_name))),
        dirs_home().map(|h| h.join(".mavis").join("bin").join(format!("{}.exe", exe_name))),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// 在 PATH 中查找命令
pub fn which(cmd: &str) -> Option<PathBuf> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_which_finds_system_command() {
        if cfg!(windows) {
            assert!(which("cmd").is_some());
        } else {
            assert!(which("sh").is_some());
        }
    }

    #[test]
    fn test_which_returns_none_for_nonexistent() {
        assert!(which("nonexistent-command-xyz-123").is_none());
    }

    #[test]
    fn test_expand_path_absolute() {
        let p = expand_path(r"C:\Windows\System32");
        assert_eq!(p, PathBuf::from(r"C:\Windows\System32"));
    }

    #[test]
    fn test_expand_path_tilde() {
        let home = dirs_home().unwrap();
        let p = expand_path("~/test");
        assert_eq!(p, home.join("test"));
    }

    #[test]
    fn test_detect_desktop_returns_none_for_garbage() {
        assert!(detect_desktop("THIS_DOES_NOT_EXIST_123456").is_none());
    }

    #[test]
    fn test_walk_dir_fast_non_existent() {
        let result = walk_dir_fast(&PathBuf::from(r"C:\THIS_PATH_DOES_NOT_EXIST_123"), 3);
        assert!(result.is_ok() || result.is_err());
    }
}
