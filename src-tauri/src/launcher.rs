use std::path::PathBuf;
use std::process::Command;

/// 查找 MiniMax Code 桌面版可执行文件
pub fn find_minimax_desktop() -> Option<PathBuf> {
    let candidates = desktop_candidate_paths();
    for p in &candidates {
        if p.is_file() {
            return Some(p.clone());
        }
    }
    None
}

pub fn find_claude_desktop() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        candidates.push(PathBuf::from(&local_app_data).join("AnthropicClaude").join("Claude.exe"));
        candidates.push(PathBuf::from(local_app_data).join("Programs").join("Claude").join("Claude.exe"));
    }
    candidates.into_iter().find(|path| path.is_file())
}

pub fn find_codex_desktop() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        let base = PathBuf::from(local_app_data);
        candidates.push(base.join("Programs").join("Codex").join("Codex.exe"));
        candidates.push(base.join("OpenAI").join("Codex").join("codex.exe"));
    }
    // Scoop install path
    if let Some(user_profile) = std::env::var_os("USERPROFILE") {
        let base = PathBuf::from(user_profile);
        candidates.push(base.join("scoop").join("apps").join("codex").join("current").join("Codex.exe"));
        candidates.push(base.join("scoop").join("apps").join("codex-cli").join("current").join("Codex.exe"));
    }
    candidates.into_iter().find(|path| path.is_file())
}

pub fn find_gemini_desktop() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|base| base.join("Programs").join("Gemini").join("Gemini.exe"))
        .filter(|path| path.is_file())
}

pub fn find_cli_command(cmd: &str) -> Option<PathBuf> {
    which(cmd).ok().or_else(|| {
        let home = dirs_home()?;
        let local_app_data = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
        let app_data = std::env::var_os("APPDATA").map(PathBuf::from);

        let candidates = match cmd {
            "qwen" => vec![
                app_data.as_ref().map(|p| p.join("npm").join("qwen.cmd")),
                Some(home.join(".local").join("bin").join("qwen.exe")),
                Some(home.join(".bun").join("bin").join("qwen.exe")),
                Some(home.join("scoop").join("shims").join("qwen.exe")),
                local_app_data.as_ref().map(|p| p.join("pnpm").join("qwen.exe")),
            ],
            "kimi" => vec![
                Some(home.join(".kimi").join("bin").join("kimi.exe")),
                app_data.as_ref().map(|p| p.join("npm").join("kimi.cmd")),
                Some(home.join(".local").join("bin").join("kimi.exe")),
                Some(home.join(".bun").join("bin").join("kimi.exe")),
            ],
            "aider" => vec![
                Some(home.join(".local").join("bin").join("aider.exe")),
                app_data.as_ref().map(|p| p.join("Python").join("Scripts").join("aider.exe")),
                local_app_data.as_ref().map(|p| p.join("Programs").join("Python").join("Python312").join("Scripts").join("aider.exe")),
                local_app_data.as_ref().map(|p| p.join("Programs").join("Python").join("Python311").join("Scripts").join("aider.exe")),
                local_app_data.as_ref().map(|p| p.join("Programs").join("Python").join("Python310").join("Scripts").join("aider.exe")),
            ],
            "opencode" => vec![
                app_data.as_ref().map(|p| p.join("npm").join("opencode.cmd")),
                local_app_data.as_ref().map(|p| p.join("Programs").join("opencode").join("opencode.exe")),
                Some(home.join("scoop").join("shims").join("opencode.exe")),
                Some(PathBuf::from(r"C:\Program Files").join("opencode").join("opencode.exe")),
                Some(home.join(".bun").join("bin").join("opencode.exe")),
                Some(home.join(".opencode").join("bin").join("opencode.exe")),
                local_app_data.as_ref().map(|p| p.join("pnpm").join("opencode.exe")),
            ],
            "grok" => vec![
                Some(home.join(".grok").join("bin").join("grok.exe")),
                Some(home.join(".local").join("bin").join("grok.exe")),
                local_app_data.as_ref().map(|p| p.join("Programs").join("grok").join("grok.exe")),
                Some(home.join(".bun").join("bin").join("grok.exe")),
                local_app_data.as_ref().map(|p| p.join("pnpm").join("grok.exe")),
            ],
            "coffee-cli" => vec![
                local_app_data.as_ref().map(|p| p.join("Coffee CLI").join("coffee-cli.exe")),
                app_data.as_ref().map(|p| p.join("npm").join("coffee-cli.cmd")),
            ],
            _ => Vec::new(),
        };

        candidates.into_iter().flatten().find(|path| path.is_file())
    })
}

fn desktop_candidate_paths() -> Vec<PathBuf> {
    let mut list = Vec::new();

    // 1) 桌面快捷方式所在路径（用户自定义安装）
    if let Some(home) = dirs_home() {
        list.push(
            home.join("Desktop")
                .join("ai编程")
                .join("MiniMax Code")
                .join("MiniMax Code.exe"),
        );
        list.push(
            home.join("Desktop")
                .join("MiniMax Code")
                .join("MiniMax Code.exe"),
        );
    }

    // 2) 标准 Electron 安装路径（%LOCALAPPDATA%\Programs\）
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        let base = PathBuf::from(local_app_data).join("Programs").join("MiniMax Code");
        list.push(base.join("MiniMax Code.exe"));
    }

    list
}

pub fn launch_minimax_desktop() -> std::io::Result<u32> {
    let path = find_minimax_desktop().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "MiniMax Code desktop app not found")
    })?;
    spawn_or_elevate(path)
}

fn launch_specific(path: Option<PathBuf>, label: &str) -> std::io::Result<u32> {
    let binary = path.ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, format!("{label} not found"))
    })?;
    spawn_or_elevate(binary)
}

/// 尝试启动进程，如果遇到 UAC 权限问题则自动请求提权
fn spawn_or_elevate(path: PathBuf) -> std::io::Result<u32> {
    // 先直接尝试 spawn（普通情况）
    match Command::new(&path).spawn() {
        Ok(child) => return Ok(child.id()),
        Err(e) => {
            // Windows 上错误码 740 = ERROR_ELEVATION_REQUIRED
            if cfg!(windows) && e.raw_os_error() == Some(740) {
                // 忽略该错误，继续走提权路径
            } else {
                return Err(e);
            }
        }
    }

    // 需要提权：通过 PowerShell Start-Process -Verb RunAs 触发 UAC 弹窗
    let path_str = path.to_string_lossy();
    let ps_cmd = format!("Start-Process '{}' -Verb RunAs", path_str.replace('\'', "''"));
    let ps_child = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_cmd])
        .spawn()
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("请求管理员权限失败: {}。可尝试右键以管理员身份运行此工具。", e),
            )
        })?;
    Ok(ps_child.id())
}

pub fn launch_claude_desktop() -> std::io::Result<u32> {
    launch_specific(find_claude_desktop(), "Claude desktop app")
}

pub fn launch_codex_desktop() -> std::io::Result<u32> {
    launch_specific(find_codex_desktop(), "Codex desktop app")
}

pub fn launch_gemini_desktop() -> std::io::Result<u32> {
    launch_specific(find_gemini_desktop(), "Gemini desktop app")
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

pub fn find_claude() -> Option<PathBuf> {
    // 优先找 MiniMax Code 桌面版
    if let Some(p) = find_minimax_desktop() {
        return Some(p);
    }
    // 回退到 minimax CLI
    if let Ok(p) = which("minimax") {
        return Some(p);
    }
    // 再回退到 claude CLI
    which("claude").ok()
}

pub fn claude_binary_path() -> PathBuf {
    // 优先 MiniMax Code 桌面版
    if let Some(p) = find_minimax_desktop() {
        return p;
    }
    // 回退到 minimax CLI
    if let Ok(p) = which("minimax") {
        return p;
    }
    // 再回退到 claude CLI
    if let Ok(p) = which("claude") {
        return p;
    }
    PathBuf::from("MiniMax Code.exe")
}

pub fn which(cmd: &str) -> std::io::Result<PathBuf> {
    let path = std::env::var_os("PATH")
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "PATH not set"))?;

    // Windows 上优先 .exe > .cmd > .bat > 无后缀。
    #[cfg(windows)]
    let candidates_per_dir: Vec<String> = {
        let mut v = Vec::with_capacity(4);
        v.push(format!("{}.exe", cmd));
        v.push(format!("{}.cmd", cmd));
        v.push(format!("{}.bat", cmd));
        v.push(cmd.to_string());
        v
    };

    #[cfg(not(windows))]
    let candidates_per_dir: Vec<String> = vec![cmd.to_string()];

    for dir in std::env::split_paths(&path) {
        for candidate_name in &candidates_per_dir {
            let candidate = dir.join(candidate_name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "not found"))
}

// 仅测试可见的别名
#[doc(hidden)]
pub use which as which_for_test;

pub fn launch_claude() -> std::io::Result<u32> {
    let path = claude_binary_path();
    let path_str = path.to_string_lossy();
    let lower = path_str.to_lowercase();

    // 根据文件扩展名选择解释器
    let cmd = if lower.ends_with(".cmd") || lower.ends_with(".bat") {
        let mut c = Command::new("cmd");
        c.arg("/c").arg(&path);
        // cmd /c 启动的 .bat 由 cmd 负责，不需要额外提权处理
        c.spawn().map(|child| child.id())
    } else if lower.ends_with(".sh") {
        let mut c = Command::new("bash");
        c.arg(&path);
        c.spawn().map(|child| child.id())
    } else if lower.ends_with(".exe") {
        spawn_or_elevate(path)
    } else {
        // 无后缀脚本
        let mut c = Command::new("bash");
        c.arg(&path);
        c.spawn().map(|child| child.id())
    };

    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_which_finds_self() {
        // 应该能找到 cmd.exe（Windows）或 sh（Unix）
        let result = if cfg!(windows) {
            which("cmd")
        } else {
            which("sh")
        };
        assert!(result.is_ok(), "应该能找到系统命令: {:?}", result.err());
    }
}
