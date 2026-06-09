use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

#[derive(Debug, thiserror::Error)]
pub enum ToolConfigError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("yaml: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("toml deserialize: {0}")]
    TomlDe(#[from] toml::de::Error),
    #[error("toml serialize: {0}")]
    TomlSer(#[from] toml::ser::Error),
    #[error("invalid config: {0}")]
    Invalid(String),
}

fn provider_package(anthropic_mode: bool) -> &'static str {
    if anthropic_mode {
        "@ai-sdk/anthropic"
    } else {
        "@ai-sdk/openai"
    }
}

fn ensure_parent_dir(path: &Path) -> Result<(), ToolConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn write_atomic(path: &Path, content: &str) -> Result<(), ToolConfigError> {
    ensure_parent_dir(path)?;
    let tmp_path = path.with_extension("tmp");
    {
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
    }
    fs::rename(tmp_path, path)?;
    Ok(())
}

fn home_dir() -> Result<PathBuf, ToolConfigError> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .ok_or_else(|| ToolConfigError::Invalid("无法找到用户目录".to_string()))
}

pub fn codex_config_path() -> Result<PathBuf, ToolConfigError> {
    Ok(home_dir()?.join(".codex").join("config.toml"))
}

pub fn codex_desktop_config_path() -> Result<PathBuf, ToolConfigError> {
    #[cfg(windows)]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return Ok(PathBuf::from(appdata).join("Codex").join("config.toml"));
        }
    }
    // Fallback: same as CLI
    codex_config_path()
}

pub fn opencode_config_path() -> Result<PathBuf, ToolConfigError> {
    Ok(home_dir()?.join(".opencode").join("config.json"))
}

pub fn qwen_config_path() -> Result<PathBuf, ToolConfigError> {
    Ok(home_dir()?.join(".qwen").join("settings.json"))
}

pub fn kimi_config_path() -> Result<PathBuf, ToolConfigError> {
    Ok(home_dir()?.join(".kimi").join("config.toml"))
}

pub fn aider_config_path() -> Result<PathBuf, ToolConfigError> {
    Ok(home_dir()?.join(".aider.conf.yml"))
}

pub fn grok_config_path() -> Result<PathBuf, ToolConfigError> {
    Ok(home_dir()?.join(".grok").join("config.toml"))
}

/// OpenAI Chat Completions 模式时，清理 api_base 中的 /anthropic 后缀
fn strip_anthropic_suffix(base: &str) -> String {
    base.trim_end_matches('/')
        .trim_end_matches("/anthropic")
        .to_string()
}

pub fn apply_codex(
    provider_id: &str,
    provider_name: &str,
    api_base: &str,
    model: &str,
    api_key: &str,
    anthropic_mode: bool,
) -> Result<(), ToolConfigError> {
    let path = codex_config_path()?;
    apply_codex_to_path(&path, provider_id, provider_name, api_base, model, api_key, anthropic_mode)
}

pub(crate) fn apply_codex_to_path(
    path: &Path,
    provider_id: &str,
    provider_name: &str,
    api_base: &str,
    model: &str,
    api_key: &str,
    anthropic_mode: bool,
) -> Result<(), ToolConfigError> {
    let content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let mut root = if content.trim().is_empty() {
        TomlValue::Table(toml::map::Map::new())
    } else {
        toml::from_str::<TomlValue>(&content)?
    };

    let table = root
        .as_table_mut()
        .ok_or_else(|| ToolConfigError::Invalid("Codex 配置根节点不是 table".to_string()))?;

    table.insert("model_provider".to_string(), TomlValue::String(provider_id.to_string()));
    table.insert("model".to_string(), TomlValue::String(model.to_string()));

    let provider_tables = table
        .entry("model_providers".to_string())
        .or_insert_with(|| TomlValue::Table(toml::map::Map::new()))
        .as_table_mut()
        .ok_or_else(|| ToolConfigError::Invalid("model_providers 不是 table".to_string()))?;

    let provider_table = provider_tables
        .entry(provider_id.to_string())
        .or_insert_with(|| TomlValue::Table(toml::map::Map::new()))
        .as_table_mut()
        .ok_or_else(|| ToolConfigError::Invalid("provider table 不是 table".to_string()))?;

    let wire_api = if anthropic_mode { "responses" } else { "chat_completions" };
    let effective_base = if anthropic_mode {
        api_base.to_string()
    } else {
        strip_anthropic_suffix(api_base)
    };

    provider_table.insert("name".to_string(), TomlValue::String(provider_name.to_string()));
    provider_table.insert("base_url".to_string(), TomlValue::String(effective_base));
    provider_table.insert("wire_api".to_string(), TomlValue::String(wire_api.to_string()));
    provider_table.insert(
        "experimental_bearer_token".to_string(),
        TomlValue::String(api_key.to_string()),
    );

    let serialized = toml::to_string_pretty(&root)?;
    write_atomic(&path, &serialized)
}

pub fn apply_opencode(
    provider_id: &str,
    provider_name: &str,
    api_base: &str,
    model: &str,
    api_key: &str,
    anthropic_mode: bool,
) -> Result<(), ToolConfigError> {
    let path = opencode_config_path()?;
    apply_opencode_to_path(&path, provider_id, provider_name, api_base, model, api_key, anthropic_mode)
}

fn apply_opencode_to_path(
    path: &Path,
    provider_id: &str,
    provider_name: &str,
    api_base: &str,
    model: &str,
    api_key: &str,
    anthropic_mode: bool,
) -> Result<(), ToolConfigError> {
    let content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let mut root = if content.trim().is_empty() {
        JsonValue::Object(JsonMap::new())
    } else {
        serde_json::from_str::<JsonValue>(&content)?
    };

    let root_obj = root
        .as_object_mut()
        .ok_or_else(|| ToolConfigError::Invalid("OpenCode 配置根节点不是 object".to_string()))?;

    root_obj.insert(
        "model".to_string(),
        JsonValue::String(format!("{}/{}", provider_id, model)),
    );

    let provider_obj = root_obj
        .entry("provider".to_string())
        .or_insert_with(|| JsonValue::Object(JsonMap::new()))
        .as_object_mut()
        .ok_or_else(|| ToolConfigError::Invalid("provider 不是 object".to_string()))?;

    let mut models_obj = JsonMap::new();
    models_obj.insert(
        model.to_string(),
        serde_json::json!({
            "name": model,
            "reasoning": true,
            "tool_call": true,
            "limit": {
                "context": 128000,
                "output": 8192
            }
        }),
    );

    provider_obj.insert(
        provider_id.to_string(),
        serde_json::json!({
            "id": provider_id,
            "name": provider_name,
            "npm": provider_package(anthropic_mode),
            "models": JsonValue::Object(models_obj),
            "options": {
                "apiKey": api_key,
                "baseURL": api_base
            }
        }),
    );

    let serialized = serde_json::to_string_pretty(&root)?;
    write_atomic(&path, &serialized)
}

pub fn apply_qwen(
    provider_id: &str,
    provider_name: &str,
    api_base: &str,
    model: &str,
    api_key: &str,
    anthropic_mode: bool,
) -> Result<(), ToolConfigError> {
    let path = qwen_config_path()?;
    apply_qwen_to_path(&path, provider_id, provider_name, api_base, model, api_key, anthropic_mode)
}

fn apply_qwen_to_path(
    path: &Path,
    provider_id: &str,
    provider_name: &str,
    api_base: &str,
    model: &str,
    api_key: &str,
    anthropic_mode: bool,
) -> Result<(), ToolConfigError> {
    let content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let mut root = if content.trim().is_empty() {
        JsonValue::Object(JsonMap::new())
    } else {
        serde_json::from_str::<JsonValue>(&content)?
    };

    let root_obj = root
        .as_object_mut()
        .ok_or_else(|| ToolConfigError::Invalid("Qwen 配置根节点不是 object".to_string()))?;

    let env_key = if anthropic_mode {
        format!("{}_ANTHROPIC_API_KEY", provider_id.to_uppercase().replace('-', "_"))
    } else {
        format!("{}_OPENAI_API_KEY", provider_id.to_uppercase().replace('-', "_"))
    };
    let auth_type = if anthropic_mode { "anthropic" } else { "openai" };

    let env_obj = root_obj
        .entry("env".to_string())
        .or_insert_with(|| JsonValue::Object(JsonMap::new()))
        .as_object_mut()
        .ok_or_else(|| ToolConfigError::Invalid("env 不是 object".to_string()))?;
    env_obj.insert(env_key.clone(), JsonValue::String(api_key.to_string()));

    root_obj.insert(
        "security".to_string(),
        serde_json::json!({
            "auth": {
                "selectedType": auth_type
            }
        }),
    );
    root_obj.insert(
        "model".to_string(),
        serde_json::json!({
            "name": model
        }),
    );

    let model_providers_obj = root_obj
        .entry("modelProviders".to_string())
        .or_insert_with(|| JsonValue::Object(JsonMap::new()))
        .as_object_mut()
        .ok_or_else(|| ToolConfigError::Invalid("modelProviders 不是 object".to_string()))?;

    let provider_entry = serde_json::json!({
        "id": model,
        "name": provider_name,
        "envKey": env_key,
        "baseUrl": api_base,
        "generationConfig": {
            "contextWindowSize": 128000,
            "samplingParams": {
                "max_tokens": 8192
            }
        }
    });
    let providers_for_type = model_providers_obj
        .entry(auth_type.to_string())
        .or_insert_with(|| JsonValue::Array(Vec::new()))
        .as_array_mut()
        .ok_or_else(|| ToolConfigError::Invalid("modelProviders 条目不是 array".to_string()))?;
    providers_for_type.retain(|existing| {
        existing
            .get("envKey")
            .and_then(JsonValue::as_str)
            != Some(env_key.as_str())
            && existing.get("id").and_then(JsonValue::as_str) != Some(model)
    });
    providers_for_type.push(provider_entry);

    let serialized = serde_json::to_string_pretty(&root)?;
    write_atomic(&path, &serialized)
}

pub fn apply_kimi(
    provider_id: &str,
    provider_name: &str,
    api_base: &str,
    model: &str,
    api_key: &str,
    anthropic_mode: bool,
) -> Result<(), ToolConfigError> {
    let path = kimi_config_path()?;
    apply_kimi_to_path(&path, provider_id, provider_name, api_base, model, api_key, anthropic_mode)
}

fn apply_kimi_to_path(
    path: &Path,
    provider_id: &str,
    provider_name: &str,
    api_base: &str,
    model: &str,
    api_key: &str,
    anthropic_mode: bool,
) -> Result<(), ToolConfigError> {
    let content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let mut root = if content.trim().is_empty() {
        TomlValue::Table(toml::map::Map::new())
    } else {
        toml::from_str::<TomlValue>(&content)?
    };

    let table = root
        .as_table_mut()
        .ok_or_else(|| ToolConfigError::Invalid("Kimi 配置根节点不是 table".to_string()))?;

    let provider_type = if provider_id == "kimi" {
        "kimi"
    } else if anthropic_mode {
        "anthropic"
    } else {
        "openai"
    };
    let model_alias = format!("{}/{}", provider_id, model);

    table.insert("default_model".to_string(), TomlValue::String(model_alias.clone()));

    let providers_table = table
        .entry("providers".to_string())
        .or_insert_with(|| TomlValue::Table(toml::map::Map::new()))
        .as_table_mut()
        .ok_or_else(|| ToolConfigError::Invalid("providers 不是 table".to_string()))?;

    let provider_table = providers_table
        .entry(provider_id.to_string())
        .or_insert_with(|| TomlValue::Table(toml::map::Map::new()))
        .as_table_mut()
        .ok_or_else(|| ToolConfigError::Invalid("provider entry 不是 table".to_string()))?;

    provider_table.insert("type".to_string(), TomlValue::String(provider_type.to_string()));
    provider_table.insert("base_url".to_string(), TomlValue::String(api_base.to_string()));
    provider_table.insert("api_key".to_string(), TomlValue::String(api_key.to_string()));
    provider_table.insert("name".to_string(), TomlValue::String(provider_name.to_string()));

    let models_table = table
        .entry("models".to_string())
        .or_insert_with(|| TomlValue::Table(toml::map::Map::new()))
        .as_table_mut()
        .ok_or_else(|| ToolConfigError::Invalid("models 不是 table".to_string()))?;

    let model_table = models_table
        .entry(model_alias)
        .or_insert_with(|| TomlValue::Table(toml::map::Map::new()))
        .as_table_mut()
        .ok_or_else(|| ToolConfigError::Invalid("model entry 不是 table".to_string()))?;

    model_table.insert("provider".to_string(), TomlValue::String(provider_id.to_string()));
    model_table.insert("model".to_string(), TomlValue::String(model.to_string()));
    model_table.insert("max_context_size".to_string(), TomlValue::Integer(128000));

    let serialized = toml::to_string_pretty(&root)?;
    write_atomic(&path, &serialized)
}

pub fn apply_aider(
    api_base: &str,
    model: &str,
    api_key: &str,
    anthropic_mode: bool,
) -> Result<(), ToolConfigError> {
    let path = aider_config_path()?;
    apply_aider_to_path(&path, api_base, model, api_key, anthropic_mode)
}

fn apply_aider_to_path(
    path: &Path,
    api_base: &str,
    model: &str,
    api_key: &str,
    anthropic_mode: bool,
) -> Result<(), ToolConfigError> {
    if anthropic_mode {
        return Err(ToolConfigError::Invalid(
            "Aider 当前只接入 OpenAI 兼容厂商，请选择 OpenAI 格式厂商".to_string(),
        ));
    }
    let content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        String::new()
    };

    let mut root = if content.trim().is_empty() {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    } else {
        serde_yaml::from_str::<serde_yaml::Value>(&content)
            .unwrap_or_else(|_| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()))
    };

    let mapping = root
        .as_mapping_mut()
        .ok_or_else(|| ToolConfigError::Invalid("Aider 配置根节点不是 mapping".to_string()))?;

    mapping.insert(
        serde_yaml::Value::String("model".to_string()),
        serde_yaml::Value::String(model.to_string()),
    );
    mapping.insert(
        serde_yaml::Value::String("openai-api-base".to_string()),
        serde_yaml::Value::String(api_base.to_string()),
    );
    mapping.insert(
        serde_yaml::Value::String("openai-api-key".to_string()),
        serde_yaml::Value::String(api_key.to_string()),
    );

    let serialized = serde_yaml::to_string(&root)?;
    write_atomic(&path, &serialized)
}

pub fn apply_grok(
    provider_id: &str,
    provider_name: &str,
    api_base: &str,
    model: &str,
    api_key: &str,
    anthropic_mode: bool,
) -> Result<(), ToolConfigError> {
    if anthropic_mode {
        return Err(ToolConfigError::Invalid(
            "Grok Build 当前只接入 OpenAI 兼容厂商，请选择 OpenAI 格式厂商".to_string(),
        ));
    }

    let path = grok_config_path()?;
    let content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let mut root = if content.trim().is_empty() {
        TomlValue::Table(toml::map::Map::new())
    } else {
        toml::from_str::<TomlValue>(&content)?
    };

    let table = root
        .as_table_mut()
        .ok_or_else(|| ToolConfigError::Invalid("Grok 配置根节点不是 table".to_string()))?;

    table.insert("model".to_string(), TomlValue::String(model.to_string()));
    table.insert("provider".to_string(), TomlValue::String(provider_id.to_string()));

    let providers_table = table
        .entry("providers".to_string())
        .or_insert_with(|| TomlValue::Table(toml::map::Map::new()))
        .as_table_mut()
        .ok_or_else(|| ToolConfigError::Invalid("providers 不是 table".to_string()))?;

    let provider_table = providers_table
        .entry(provider_id.to_string())
        .or_insert_with(|| TomlValue::Table(toml::map::Map::new()))
        .as_table_mut()
        .ok_or_else(|| ToolConfigError::Invalid("provider entry 不是 table".to_string()))?;

    provider_table.insert("name".to_string(), TomlValue::String(provider_name.to_string()));
    provider_table.insert("base_url".to_string(), TomlValue::String(api_base.to_string()));
    provider_table.insert("api_key".to_string(), TomlValue::String(api_key.to_string()));

    let serialized = toml::to_string_pretty(&root)?;
    write_atomic(&path, &serialized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn qwen_merge_preserves_unrelated_providers() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        let initial = serde_json::json!({
            "modelProviders": {
                "openai": [
                    {
                        "id": "old-model",
                        "name": "Old Provider",
                        "envKey": "OLD_OPENAI_API_KEY",
                        "baseUrl": "https://example.com/v1"
                    }
                ]
            }
        });
        fs::write(&path, serde_json::to_string_pretty(&initial).unwrap()).unwrap();
        apply_qwen_to_path(
            &path,
            "deepseek",
            "DeepSeek",
            "https://api.deepseek.com/v1",
            "deepseek-chat",
            "sk-test",
            false,
        )
        .unwrap();

        let saved: JsonValue = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let entries = saved["modelProviders"]["openai"].as_array().unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|item| item["id"] == "old-model"));
        assert!(entries.iter().any(|item| item["id"] == "deepseek-chat"));
    }

    #[test]
    fn codex_wires_api_by_mode() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        // Anthropic 模式 → wire_api = "responses"
        apply_codex_to_path(
            &path,
            "deepseek",
            "DeepSeek",
            "https://api.deepseek.com/anthropic",
            "deepseek-chat",
            "sk-test",
            true,
        )
        .unwrap();
        let parsed: TomlValue = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(
            parsed["model_providers"]["deepseek"]["wire_api"].as_str(),
            Some("responses")
        );

        // OpenAI 模式 → wire_api = "chat_completions" + 去掉 /anthropic 后缀
        let path2 = dir.path().join("config2.toml");
        apply_codex_to_path(
            &path2,
            "deepseek",
            "DeepSeek",
            "https://api.deepseek.com/anthropic",
            "deepseek-chat",
            "sk-test",
            false,
        )
        .unwrap();
        let parsed2: TomlValue = toml::from_str(&fs::read_to_string(&path2).unwrap()).unwrap();
        assert_eq!(
            parsed2["model_providers"]["deepseek"]["wire_api"].as_str(),
            Some("chat_completions")
        );
        assert_eq!(
            parsed2["model_providers"]["deepseek"]["base_url"].as_str(),
            Some("https://api.deepseek.com")  // /anthropic 已被剥离
        );
    }

    #[test]
    fn opencode_merges_provider_and_model() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        fs::write(
            &path,
            serde_json::to_string_pretty(&serde_json::json!({
                "theme": "system",
                "provider": {
                    "legacy": {
                        "name": "Legacy",
                        "options": { "apiKey": "old" }
                    }
                }
            }))
            .unwrap(),
        )
        .unwrap();

        apply_opencode_to_path(
            &path,
            "deepseek",
            "DeepSeek",
            "https://api.deepseek.com/v1",
            "deepseek-chat",
            "sk-test",
            false,
        )
        .unwrap();

        let saved: JsonValue = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(saved["theme"], "system");
        assert!(saved["provider"]["legacy"].is_object());
        assert_eq!(saved["provider"]["deepseek"]["options"]["baseURL"], "https://api.deepseek.com/v1");
        assert_eq!(saved["model"], "deepseek/deepseek-chat");
    }

    #[test]
    fn kimi_writes_default_model_and_provider() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        apply_kimi_to_path(
            &path,
            "kimi",
            "Kimi",
            "https://api.moonshot.cn/v1",
            "moonshot-v1-128k",
            "sk-test",
            false,
        )
        .unwrap();

        let parsed: TomlValue = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(
            parsed["default_model"].as_str(),
            Some("kimi/moonshot-v1-128k")
        );
        assert_eq!(parsed["providers"]["kimi"]["type"].as_str(), Some("kimi"));
        assert_eq!(
            parsed["providers"]["kimi"]["base_url"].as_str(),
            Some("https://api.moonshot.cn/v1")
        );
    }

    #[test]
    fn aider_rejects_anthropic_mode() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".aider.conf.yml");
        let result = apply_aider_to_path(
            &path,
            "https://api.deepseek.com/anthropic",
            "deepseek-chat",
            "sk-test",
            true,
        );
        assert!(result.is_err());
    }

    #[test]
    fn grok_writes_provider_and_model() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        let mut root = TomlValue::Table(toml::map::Map::new());
        let table = root.as_table_mut().unwrap();
        table.insert("theme".to_string(), TomlValue::String("dark".to_string()));
        fs::write(&path, toml::to_string_pretty(&root).unwrap()).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let mut parsed = toml::from_str::<TomlValue>(&content).unwrap();
        let table = parsed.as_table_mut().unwrap();
        table.insert("model".to_string(), TomlValue::String("grok-4".to_string()));
        table.insert("provider".to_string(), TomlValue::String("deepseek".to_string()));
        let providers_table = table
            .entry("providers".to_string())
            .or_insert_with(|| TomlValue::Table(toml::map::Map::new()))
            .as_table_mut()
            .unwrap();
        let provider_table = providers_table
            .entry("deepseek".to_string())
            .or_insert_with(|| TomlValue::Table(toml::map::Map::new()))
            .as_table_mut()
            .unwrap();
        provider_table.insert("base_url".to_string(), TomlValue::String("https://api.deepseek.com/v1".to_string()));
        provider_table.insert("api_key".to_string(), TomlValue::String("sk-test".to_string()));
        fs::write(&path, toml::to_string_pretty(&parsed).unwrap()).unwrap();

        let saved: TomlValue = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(saved["theme"].as_str(), Some("dark"));
        assert_eq!(saved["model"].as_str(), Some("grok-4"));
        assert_eq!(saved["provider"].as_str(), Some("deepseek"));
        assert_eq!(saved["providers"]["deepseek"]["base_url"].as_str(), Some("https://api.deepseek.com/v1"));
    }
}
