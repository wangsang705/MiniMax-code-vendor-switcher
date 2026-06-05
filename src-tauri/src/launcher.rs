use std::path::PathBuf;
use std::process::Command;

pub fn claude_binary_path() -> PathBuf {
    // 优先 MiniMax-code，回退到 claude
    if let Ok(p) = which("MiniMax-code") {
        return p;
    }
    if let Ok(p) = which("claude") {
        return p;
    }
    PathBuf::from("MiniMax-code")
}

pub fn find_claude() -> Option<PathBuf> {
    which("MiniMax-code").or_else(|_| which("claude")).ok()
}

pub fn which(cmd: &str) -> std::io::Result<PathBuf> {
    let path = std::env::var_os("PATH")
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "PATH not set"))?;

    // Windows 上优先 .exe > .cmd > .bat > 无后缀。
    // 在 Windows 上，Path 上名字完全无后缀的条目（典型为 sh/bash 脚本 wrapper）
    // 不能被 Command::new 直接 spawn；必须降级到最后再考虑，
    // 否则会把 npm 的 sh 脚本当成本体执行。
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

// 仅测试可见的别名，方便 tests/launcher_test.rs 直接验证搜索优先级。
#[doc(hidden)]
pub use which as which_for_test;

pub fn launch_claude() -> std::io::Result<u32> {
    let path = claude_binary_path();
    let path_str = path.to_string_lossy();
    let lower = path_str.to_lowercase();

    // 根据文件扩展名选择解释器：
    //   .cmd / .bat -> cmd /c <path>
    //   .exe        -> 直接 spawn（PE 格式可由 Windows 直接加载）
    //   .sh 或无后缀 -> 视为 sh/bash 脚本，用 bash 执行
    //                   （用户机器上 Git for Windows 提供 bash）
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
        // 无后缀（npm 全局安装的 claude sh 脚本 wrapper），
        // Windows 上也用 bash 解释（Git Bash / WSL bash 都在 PATH）
        let mut c = Command::new("bash");
        c.arg(&path);
        c
    };

    let child = cmd.spawn()?;
    Ok(child.id())
}
