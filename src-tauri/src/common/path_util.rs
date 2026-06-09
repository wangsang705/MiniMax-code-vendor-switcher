use std::path::PathBuf;

/// 获取用户主目录
///
/// Windows 上使用 USERPROFILE，Unix 上使用 HOME。
/// 这是项目中唯一的 `dirs_home` 实现，所有模块统一调用此函数。
pub fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

/// 展开 ~ 为用户目录
///
/// - `~/foo/bar` → `/home/user/foo/bar`（或 Windows 等效路径）
/// - `~` → `/home/user`
/// - 其他路径原样返回
pub fn expand_path(path: &str) -> PathBuf {
    let path = path.trim();
    if path == "~" {
        home_dir().unwrap_or_else(|| PathBuf::from("~"))
    } else if let Some(rest) = path.strip_prefix("~/") {
        home_dir()
            .map(|h| h.join(rest))
            .unwrap_or_else(|| PathBuf::from(path))
    } else {
        PathBuf::from(path)
    }
}

/// Tauri 中 ~/.claude/settings.json 路径
pub fn claude_settings_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".claude").join("settings.json"))
}

/// Tauri 中 ~/.minimax/config.yaml 路径
pub fn minimax_config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".minimax").join("config.yaml"))
}

/// Tauri 中 ~/.codex/config.toml 路径
pub fn codex_config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".codex").join("config.toml"))
}

/// Tauri 中 ~/.opencode/config.json 路径
pub fn opencode_config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".opencode").join("config.json"))
}

/// Tauri 中 ~/.qwen/settings.json 路径
pub fn qwen_config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".qwen").join("settings.json"))
}

/// Tauri 中 ~/.kimi/config.toml 路径
pub fn kimi_config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".kimi").join("config.toml"))
}

/// Tauri 中 ~/.aider.conf.yml 路径
pub fn aider_config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".aider.conf.yml"))
}

/// Tauri 中 ~/.grok/config.toml 路径
pub fn grok_config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".grok").join("config.toml"))
}

/// Tauri 中 ~/.openclaw/openclaw.json 路径
pub fn openclaw_config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".openclaw").join("openclaw.json"))
}

/// Tauri 中 ~/.hermes/config.yaml 路径
pub fn hermes_config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".hermes").join("config.yaml"))
}

/// Tauri 中 ~/.nanobot/config.json 路径
pub fn nanobot_config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".nanobot").join("config.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_dir_returns_some() {
        assert!(home_dir().is_some(), "home_dir() 应返回 Some");
    }

    #[test]
    fn test_expand_path_tilde() {
        let home = home_dir().unwrap();
        assert_eq!(expand_path("~"), home);
    }

    #[test]
    fn test_expand_path_tilde_prefix() {
        let home = home_dir().unwrap();
        let expected = home.join(".claude").join("settings.json");
        assert_eq!(expand_path("~/.claude/settings.json"), expected);
    }

    #[test]
    fn test_expand_path_absolute() {
        let path = "/absolute/path";
        assert_eq!(expand_path(path), PathBuf::from(path));
    }
}
