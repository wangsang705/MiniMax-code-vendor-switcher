use std::fs;
use std::path::Path;

use crate::common::atomic_io;

/// MiniMax Config Error
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
 #[error("io: {0}")]
 Io(#[from] std::io::Error),
 #[error("io: {0}")]
 IoMsg(String),
 #[error("yaml: {0}")]
 Yaml(#[from] serde_yaml::Error),
 #[error("invalid config: {0}")]
 Invalid(String),
}

//辅助宏：快速创建 String → serde_yaml::Value键
macro_rules! s {
 ($val:expr) => {
 serde_yaml::Value::String($val.to_string())
 };
}

/// 生成默认的模型配置条目（用于新建 provider 时）
fn generate_model_entry(model_name: &str) -> serde_yaml::Value {
 let mut m = serde_yaml::Mapping::new();
 m.insert(s!("name"), s!(model_name));
 m.insert(s!("attachment"), serde_yaml::Value::Bool(false));
 m.insert(s!("reasoning"), serde_yaml::Value::Bool(true));
 m.insert(s!("temperature"), serde_yaml::Value::Bool(true));
 m.insert(s!("tool_call"), serde_yaml::Value::Bool(true));

 let mut limit = serde_yaml::Mapping::new();
 limit.insert(s!("context"), serde_yaml::Value::Number(serde_yaml::Number::from(128_000u64)));
 limit.insert(s!("output"), serde_yaml::Value::Number(serde_yaml::Number::from(8_192u64)));
 m.insert(s!("limit"), serde_yaml::Value::Mapping(limit));

 let mut modals = serde_yaml::Mapping::new();
 modals.insert(
 s!("input"),
 serde_yaml::Value::Sequence(vec![s!("text")]),
 );
 modals.insert(
 s!("output"),
 serde_yaml::Value::Sequence(vec![s!("text")]),
 );
 m.insert(s!("modalities"), serde_yaml::Value::Mapping(modals));

 serde_yaml::Value::Mapping(m)
}

/// 创建全新的 provider 配置（provider尚不存在时用）
fn create_provider_entry(
 provider_name: &str,
 model: &str,
 api_base: &str,
 api_key: &str,
) -> serde_yaml::Value {
 let mut provider = serde_yaml::Mapping::new();
 provider.insert(s!("name"), s!(provider_name));
 provider.insert(s!("npm"), s!("@ai-sdk/anthropic"));

 // models
 let mut models = serde_yaml::Mapping::new();
 models.insert(s!(model), generate_model_entry(model));
 provider.insert(s!("models"), serde_yaml::Value::Mapping(models));

 // whitelist
 provider.insert(
 s!("whitelist"),
 serde_yaml::Value::Sequence(vec![s!(model)]),
 );

 // options
 let mut opts = serde_yaml::Mapping::new();
 opts.insert(s!("apiKey"), s!(api_key));
 opts.insert(s!("baseURL"), s!(api_base));
 provider.insert(s!("options"), serde_yaml::Value::Mapping(opts));

 serde_yaml::Value::Mapping(provider)
}

///读取 MiniMax配置文件，返回 serde_yaml::Value
pub fn read_config(path: &Path) -> Result<serde_yaml::Value, ConfigError> {
 if !path.exists() {
 return Ok(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
 }
 let content = fs::read_to_string(path)?;
 let val: serde_yaml::Value = serde_yaml::from_str(&content)?;
 Ok(val)
}

///原子写入 YAML 文件（带备份 +收紧权限，因为含 API Key）
pub fn write_config_atomic(path: &Path, config: &serde_yaml::Value) -> Result<(), ConfigError> {
 let yaml_str = serde_yaml::to_string(config)?;
 atomic_io::atomic_write_secret(path, &yaml_str).map_err(|e| ConfigError::IoMsg(e.to_string()))
}

/// 将一个厂商的配置写入 MiniMax config.yaml
///
/// - 如果 provider 已存在，只更新 options.apiKey / options.baseURL
/// - 如果 provider 不存在，创建完整条目（含 model、whitelist 等）
/// - 同时更新 defaultModel
pub fn apply_provider(
 path: &Path,
 provider_id: &str,
 provider_name: &str,
 api_base: &str,
 model: &str,
 api_key: &str,
) -> Result<(), ConfigError> {
 let mut config = read_config(path)?;

 // 确保根节点是 Mapping
 let root = config
 .as_mapping_mut()
 .ok_or_else(|| ConfigError::Invalid("config root is not a mapping".into()))?;

 // ----- provider段 -----
 let provider_section: &mut serde_yaml::Mapping = root
 .entry(s!("provider"))
 .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()))
 .as_mapping_mut()
 .ok_or_else(|| ConfigError::Invalid("provider section is not a mapping".into()))?;

 if provider_section.contains_key(&s!(provider_id)) {
 // provider 已存在 → 只更新 options
 let entry = provider_section
 .get_mut(&s!(provider_id))
 .unwrap()
 .as_mapping_mut()
 .ok_or_else(|| ConfigError::Invalid("provider entry is not a mapping".into()))?;

 let options = entry
 .entry(s!("options"))
 .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()))
 .as_mapping_mut()
 .ok_or_else(|| ConfigError::Invalid("provider options is not a mapping".into()))?;

 options.insert(s!("apiKey"), s!(api_key));
 options.insert(s!("baseURL"), s!(api_base));
 } else {
 // provider 不存在 → 创建完整条目
 provider_section.insert(
 s!(provider_id),
 create_provider_entry(provider_name, model, api_base, api_key),
 );
 }

 // ----- defaultModel -----
 root.insert(
 s!("defaultModel"),
 s!(format!("{}/{}", provider_id, model)),
 );

 write_config_atomic(path, &config)
}

#[cfg(test)]
mod tests {
 use super::*;
 use tempfile::TempDir;

 #[test]
 fn test_apply_provider_new() {
 let dir = TempDir::new().unwrap();
 let path = dir.path().join("config.yaml");

 apply_provider(&path, "deepseek", "DeepSeek", "https://api.deepseek.com", "deepseek-chat", "sk-test123").unwrap();

 let content = fs::read_to_string(&path).unwrap();
 assert!(content.contains("deepseek"));
 assert!(content.contains("sk-test123"));
 assert!(content.contains("https://api.deepseek.com"));
 assert!(content.contains("deepseek-chat"));
 assert!(content.contains("defaultModel: deepseek/deepseek-chat"));
 }

 #[test]
 fn test_apply_provider_update_existing() {
 let dir = TempDir::new().unwrap();
 let path = dir.path().join("config.yaml");

 // 先写入一次
 apply_provider(&path, "deepseek", "DeepSeek", "https://api.deepseek.com/v1", "deepseek-chat", "sk-old").unwrap();

 // 再写入（更新）
 apply_provider(&path, "deepseek", "DeepSeek", "https://api.deepseek.com/v2", "deepseek-chat", "sk-new").unwrap();

 let content = fs::read_to_string(&path).unwrap();
 assert!(content.contains("sk-new"));
 assert!(content.contains("https://api.deepseek.com/v2"));
 assert!(!content.contains("sk-old"));
 }

 #[test]
 fn test_apply_provider_preserves_other_providers() {
 let dir = TempDir::new().unwrap();
 let path = dir.path().join("config.yaml");

 apply_provider(&path, "deepseek", "DeepSeek", "https://api.deepseek.com", "deepseek-chat", "sk-1").unwrap();
 apply_provider(&path, "kimi", "Kimi", "https://api.moonshot.cn/v1", "moonshot-v1-128k", "sk-2").unwrap();

 let content = fs::read_to_string(&path).unwrap();
 assert!(content.contains("deepseek"));
 assert!(content.contains("kimi"));
 }

 #[test]
 fn test_apply_provider_preserves_existing_sections() {
 let dir = TempDir::new().unwrap();
 let path = dir.path().join("config.yaml");

 // 先创建一个包含 nexus 的完整配置
 let initial = r#"logLevel: info
provider:
 minimax:
 name: MiniMax
 npm: '@ai-sdk/anthropic'
 models:
 MiniMax-M3:
 name: MiniMax-M3
 whitelist:
 - MiniMax-M3
 options:
 apiKey: sk-xxx
 baseURL: https://agent.minimaxi.com/mavis/api/v1/llm/v1
defaultModel: minimax/MiniMax-M3
nexus:
 enabled: true
 model:
 providerID: minimax
 modelID: MiniMax-M3
"#;
 fs::write(&path, initial).unwrap();

 // 应用新 provider
 apply_provider(&path, "deepseek", "DeepSeek", "https://api.deepseek.com", "deepseek-chat", "sk-ds").unwrap();

 let content = fs::read_to_string(&path).unwrap();
 //原有 minimax provider还在
 assert!(content.contains("minimax:"));
 assert!(content.contains("sk-xxx"));
 // nexus段还在
 assert!(content.contains("nexus:"));
 assert!(content.contains("providerID: minimax"));
 // 新 provider 已添加
 assert!(content.contains("deepseek:"));
 assert!(content.contains("sk-ds"));
 // defaultModel 已更新
 assert!(content.contains("defaultModel: deepseek/deepseek-chat"));
 }

 #[test]
 fn test_atomic_io_used() {
 //验证备份目录在多次写入后存在
 let dir = TempDir::new().unwrap();
 let path = dir.path().join("config.yaml");
 apply_provider(&path, "deepseek", "DeepSeek", "https://api.deepseek.com", "deepseek-chat", "sk-1").unwrap();
 apply_provider(&path, "kimi", "Kimi", "https://api.moonshot.cn/v1", "moonshot-v1-128k", "sk-2").unwrap();
 let backup_dir = dir.path().join("backups");
 assert!(backup_dir.exists(), "应有备份目录");
 }
}
