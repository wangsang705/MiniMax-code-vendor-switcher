use std::path::PathBuf;

// Helper macro for YAML value construction
macro_rules! s {
    ($val:expr) => { serde_yaml::Value::String($val.to_string()) };
}

/// Agent 配置写入结果
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("yaml: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("config path not found")]
    NoConfigPath,
}

fn dirs_home() -> Option<PathBuf> {
    #[cfg(windows)] { std::env::var_os("USERPROFILE").map(PathBuf::from) }
    #[cfg(not(windows))] { std::env::var_os("HOME").map(PathBuf::from) }
}

// ===== OpenClaw 适配器 =====
// 配置文件: ~/.openclaw/openclaw.json (JSON5)
// 模型设置: agents.defaults.model

pub fn apply_openclaw(provider_name: &str, api_base: &str, model: &str, api_key: &str) -> Result<(), AgentError> {
    let home = dirs_home().ok_or(AgentError::NoConfigPath)?;
    let path = home.join(".openclaw").join("openclaw.json");
    std::fs::create_dir_all(path.parent().unwrap()).ok();

    // 读取或创建配置
    let mut config: serde_json::Value = if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // 设置 agents.defaults.model
    let model_val = format!("{}/{}", provider_name, model);

    // 如果还没有 agents 段就创建
    if config.get("agents").is_none() {
        config["agents"] = serde_json::json!({"defaults": {}});
    }
    config["agents"]["defaults"]["model"] = serde_json::Value::String(model_val);
    config["agents"]["defaults"]["base_url"] = serde_json::Value::String(api_base.to_string());
    config["agents"]["defaults"]["api_key"] = serde_json::Value::String(api_key.to_string());
    config["provider"] = serde_json::json!({
        "name": provider_name,
        "base_url": api_base,
        "api_key": api_key,
    });

    // 写回文件（JSON5 格式，使用 pretty 打印）
    let json_str = serde_json::to_string_pretty(&config)?;
    std::fs::write(&path, json_str)?;
    Ok(())
}

// ===== Hermes Agent 适配器 =====
// 配置文件: ~/.hermes/config.yaml (YAML)
// 模型设置: model.default, model.provider, model.base_url, .env 存 API key

pub fn apply_hermes(provider_id: &str, api_base: &str, model: &str, api_key: &str) -> Result<(), AgentError> {
    let home = dirs_home().ok_or(AgentError::NoConfigPath)?;
    let config_path = home.join(".hermes").join("config.yaml");
    let env_path = home.join(".hermes").join(".env");
    std::fs::create_dir_all(config_path.parent().unwrap()).ok();

    // 读取或创建 config.yaml
    let mut config: serde_yaml::Value = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        serde_yaml::from_str(&content).unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()))
    } else {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    };

    let root = config.as_mapping_mut().unwrap();

    // model.default
    root.insert(
        serde_yaml::Value::String("model".to_string()),
        serde_yaml::Value::Mapping({
            let mut m = serde_yaml::Mapping::new();
            m.insert(s!("default"), s!(format!("{}/{}", provider_id, model)));
            m.insert(s!("provider"), s!(provider_id));
            m.insert(s!("base_url"), s!(api_base));
            m
        }),
    );

    // 写 config.yaml
    let yaml_str = serde_yaml::to_string(&config)?;
    std::fs::write(&config_path, yaml_str)?;

    // 写 .env（始终更新 API Key，删除旧行写入新行）
    let env_content = if env_path.exists() {
        let old = std::fs::read_to_string(&env_path)?;
        // 移除已有 HERMES_API_KEY 行
        old.lines()
            .filter(|line| !line.starts_with("HERMES_API_KEY="))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        String::new()
    };
    // 追加换行（如果内容非空且不以换行结尾）
    let separator = if env_content.is_empty() || env_content.ends_with('\n') { "" } else { "\n" };
    std::fs::write(&env_path, format!("{}{}HERMES_API_KEY={}\n", env_content, separator, api_key))?;

    Ok(())
}

// ===== Nanobot 适配器 =====
// 配置文件: ~/.nanobot/config.json (JSON)
// 模型设置: provider + model 段

pub fn apply_nanobot(provider_id: &str, api_base: &str, model: &str, api_key: &str) -> Result<(), AgentError> {
    let home = dirs_home().ok_or(AgentError::NoConfigPath)?;
    let path = home.join(".nanobot").join("config.json");
    std::fs::create_dir_all(path.parent().unwrap()).ok();

    let mut config: serde_json::Value = if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    config["provider"] = serde_json::json!({
        "id": provider_id,
        "api_base": api_base,
        "api_key": api_key,
    });
    config["model"] = serde_json::Value::String(model.to_string());

    let json_str = serde_json::to_string_pretty(&config)?;
    std::fs::write(&path, json_str)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serialize env-dependent tests to prevent HOME/USERPROFILE conflicts.
    static HOME_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_home<F>(f: F)
    where
        F: FnOnce(&std::path::Path),
    {
        let _guard = HOME_LOCK.lock().unwrap();
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_owned();
        let old = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
        // Prefer HOME (Unix) else USERPROFILE (Windows)
        let key = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
        std::env::set_var(key, &path);
        f(&path);
        if let Some(v) = old {
            std::env::set_var(key, v);
        } else {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn test_openclaw_creates_config() {
        with_temp_home(|home| {
            apply_openclaw("DeepSeek", "https://api.deepseek.com", "deepseek-chat", "sk-test123").unwrap();
            let p = home.join(".openclaw").join("openclaw.json");
            assert!(p.exists());
            let c = std::fs::read_to_string(&p).unwrap();
            assert!(c.contains("deepseek-chat"));
            assert!(c.contains("sk-test123"));
        });
    }

    #[test]
    fn test_openclaw_updates_existing() {
        with_temp_home(|home| {
            let p = home.join(".openclaw").join("openclaw.json");
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            let init = serde_json::json!({"custom": {"keep": true}});
            std::fs::write(&p, serde_json::to_string_pretty(&init).unwrap()).unwrap();
            apply_openclaw("DeepSeek", "https://api.deepseek.com", "deepseek-chat", "sk-test").unwrap();
            let saved: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
            assert_eq!(saved["custom"]["keep"], true);
            assert_eq!(saved["agents"]["defaults"]["model"], "DeepSeek/deepseek-chat");
        });
    }

    #[test]
    fn test_hermes_creates_config_and_env() {
        with_temp_home(|home| {
            apply_hermes("deepseek", "https://api.deepseek.com", "deepseek-chat", "sk-test").unwrap();
            assert!(home.join(".hermes").join("config.yaml").exists());
            assert!(home.join(".hermes").join(".env").exists());
            let env = std::fs::read_to_string(home.join(".hermes").join(".env")).unwrap();
            assert!(env.contains("HERMES_API_KEY=sk-test"));
        });
    }

    #[test]
    fn test_hermes_env_updates_key() {
        with_temp_home(|home| {
            let env = home.join(".hermes").join(".env");
            std::fs::create_dir_all(env.parent().unwrap()).unwrap();
            std::fs::write(&env, "OTHER_VAR=hello\nHERMES_API_KEY=old_key\nSOME_VAR=world\n").unwrap();
            apply_hermes("deepseek", "https://api.deepseek.com", "deepseek-chat", "sk-new").unwrap();
            let c = std::fs::read_to_string(&env).unwrap();
            assert!(c.contains("HERMES_API_KEY=sk-new"));
            assert!(c.contains("OTHER_VAR=hello"));
            assert!(!c.contains("old_key"));
        });
    }

    #[test]
    fn test_nanobot_creates_config() {
        with_temp_home(|home| {
            apply_nanobot("deepseek", "https://api.deepseek.com", "deepseek-chat", "sk-test").unwrap();
            let p = home.join(".nanobot").join("config.json");
            assert!(p.exists());
            let saved: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
            assert_eq!(saved["provider"]["id"], "deepseek");
            assert_eq!(saved["model"], "deepseek-chat");
        });
    }

    #[test]
    fn test_nanobot_updates_provider() {
        with_temp_home(|home| {
            let p = home.join(".nanobot").join("config.json");
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            let init = serde_json::json!({"saved": true});
            std::fs::write(&p, serde_json::to_string_pretty(&init).unwrap()).unwrap();
            apply_nanobot("kimi", "https://api.moonshot.cn/v1", "moonshot-v1-128k", "sk-new").unwrap();
            let saved: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
            assert_eq!(saved["saved"], true);
            assert_eq!(saved["provider"]["id"], "kimi");
        });
    }
}
