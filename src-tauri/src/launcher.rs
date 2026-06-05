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

fn which(cmd: &str) -> std::io::Result<PathBuf> {
    let path = std::env::var_os("PATH")
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "PATH not set"))?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(cmd);
        if candidate.is_file() {
            return Ok(candidate);
        }
        // Windows 上检查 .exe
        #[cfg(windows)]
        {
            let candidate_exe = dir.join(format!("{}.exe", cmd));
            if candidate_exe.is_file() {
                return Ok(candidate_exe);
            }
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "not found"))
}

pub fn launch_claude() -> std::io::Result<u32> {
    let path = claude_binary_path();
    let child = Command::new(&path).spawn()?;
    Ok(child.id())
}
