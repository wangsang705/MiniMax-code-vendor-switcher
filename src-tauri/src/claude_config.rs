use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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

pub fn write_env_atomic(path: &Path, settings: &ClaudeSettings) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if path.exists() {
        let backup_dir = path.parent().unwrap_or_else(|| Path::new(".")).join("backups");
        fs::create_dir_all(&backup_dir)?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ConfigError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
            .as_millis();
        let backup_path: PathBuf = backup_dir.join(format!("settings.{}.json", timestamp));
        fs::copy(path, &backup_path)?;
    }

    let tmp_path = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(settings)?;
    {
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(json.as_bytes())?;
        file.sync_all()?;
    }
    fs::rename(&tmp_path, path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = fs::metadata(path)?.permissions();
        perm.set_mode(0o600);
        fs::set_permissions(path, perm)?;
    }

    Ok(())
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
        assert!(entries >= 1);
    }
}
