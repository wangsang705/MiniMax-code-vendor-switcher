use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::common::atomic_io;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ClaudeSettings {
 #[serde(default, skip_serializing_if = "Option::is_none")]
 pub env: Option<HashMap<String, String>>,
 #[serde(flatten)]
 pub other: HashMap<String, serde_json::Value>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
 #[error("io: {0}")]
 Io(#[from] std::io::Error),
 #[error("io: {0}")]
 IoMsg(String),
 #[error("json: {0}")]
 Json(#[from] serde_json::Error),
}

pub fn read_settings(path: &Path) -> Result<ClaudeSettings, ConfigError> {
 if !path.exists() {
 return Ok(ClaudeSettings::default());
 }

 let content = fs::read_to_string(path)?;
 Ok(serde_json::from_str(&content)?)
}

///原子写入 Claude settings.json（带备份 +收紧权限，因为含 API Key）
pub fn write_env_atomic(path: &Path, settings: &ClaudeSettings) -> Result<(), ConfigError> {
 let json = serde_json::to_string_pretty(settings)?;
 atomic_io::atomic_write_secret(path, &json).map_err(|e| ConfigError::IoMsg(e.to_string()))
}

pub fn merge_and_write_env(
 path: &Path,
 env_updates: &HashMap<String, String>,
) -> Result<(), ConfigError> {
 let mut settings = read_settings(path)?;
 let mut env = settings.env.take().unwrap_or_default();
 for (key, value) in env_updates {
 env.insert(key.clone(), value.clone());
 }
 settings.env = Some(env);
 write_env_atomic(path, &settings)
}

#[cfg(test)]
mod tests {
 use super::*;
 use tempfile::TempDir;

 #[test]
 fn merge_preserves_existing_env() {
 let dir = TempDir::new().unwrap();
 let path = dir.path().join("settings.json");
 let mut initial = ClaudeSettings::default();
 let mut env = HashMap::new();
 env.insert("FOO".to_string(), "bar".to_string());
 initial.env = Some(env);
 write_env_atomic(&path, &initial).unwrap();

 let mut updates = HashMap::new();
 updates.insert("ANTHROPIC_BASE_URL".to_string(), "https://api.deepseek.com".to_string());
 updates.insert("ANTHROPIC_AUTH_TOKEN".to_string(), "sk-test".to_string());
 merge_and_write_env(&path, &updates).unwrap();

 let loaded = read_settings(&path).unwrap();
 let env = loaded.env.unwrap();
 assert_eq!(env.get("FOO").unwrap(), "bar");
 assert_eq!(env.get("ANTHROPIC_BASE_URL").unwrap(), "https://api.deepseek.com");
 assert_eq!(env.get("ANTHROPIC_AUTH_TOKEN").unwrap(), "sk-test");
 }

 #[test]
 fn creates_backup_on_second_write() {
 let dir = TempDir::new().unwrap();
 let path = dir.path().join("settings.json");
 write_env_atomic(&path, &ClaudeSettings::default()).unwrap();
 write_env_atomic(&path, &ClaudeSettings::default()).unwrap();

 let backup_dir = dir.path().join("backups");
 let entries = fs::read_dir(&backup_dir).unwrap().count();
 assert!(entries >=1);
 }
}
