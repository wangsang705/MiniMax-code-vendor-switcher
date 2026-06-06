use std::path::PathBuf;

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

fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") || path == "~" {
        if let Some(home) = dirs_home() { return home.join(&path[2..]); }
    }
    PathBuf::from(path)
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

    // 写 .env (追加 API key)
    let env_content = if env_path.exists() {
        std::fs::read_to_string(&env_path)?
    } else {
        String::new()
    };
    if !env_content.contains("HERMES_API_KEY") {
        std::fs::write(&env_path, format!("{}HERMES_API_KEY={}\n", env_content, api_key))?;
    }

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

// 辅助函数
macro_rules! s {
    ($val:expr) => { serde_yaml::Value::String($val.to_string()) };
}
use s;
