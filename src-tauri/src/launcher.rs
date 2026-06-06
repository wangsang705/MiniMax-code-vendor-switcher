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

fn desktop_candidate_paths() -> Vec<PathBuf> {
    let mut list = Vec::new();

    // 1) 桌面快捷方式所在路径（用户自定义安装）
    if let Some(home) = dirs_home() {
        // 常见用户自定义安装路径
        list.push(
            home.join("Desktop")
                .join("ai编程")
                .join("MiniMax Code")
                .join("MiniMax Code.exe"),
        );
        // 另一种可能的桌面安装路径
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
    let mut cmd = if lower.ends_with(".cmd") || lower.ends_with(".bat") {
        let mut c = Command::new("cmd");
        c.arg("/c").arg(&path);
        c
    } else if lower.ends_with(".sh") {
        let mut c = Command::new("bash");
        c.arg(&path);
        c
    } else if lower.ends_with(".exe") {
        Command::new(&path)
    } else {
        // 无后缀脚本
        let mut c = Command::new("bash");
        c.arg(&path);
        c
    };

    let child = cmd.spawn()?;
    Ok(child.id())
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
