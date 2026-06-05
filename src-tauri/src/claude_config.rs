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
    // 保留其他未识别字段
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
    let settings: ClaudeSettings = serde_json::from_str(&content)?;
    Ok(settings)
}

pub fn write_env_atomic(path: &Path, settings: &ClaudeSettings) -> Result<(), ConfigError> {
    // 备份原文件
    if path.exists() {
        let backup_dir = path.parent().unwrap().join("backups");
        fs::create_dir_all(&backup_dir)?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ConfigError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
            .as_millis();
        let backup_path: PathBuf = backup_dir.join(format!("settings.{}.json", timestamp));
        fs::copy(path, &backup_path)?;
    }

    // 原子写：先写临时文件再 rename
    let tmp_path = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(settings)?;
    {
        let mut f = fs::File::create(&tmp_path)?;
        f.write_all(json.as_bytes())?;
        f.sync_all()?;
    }
    fs::rename(&tmp_path, path)?;

    // 设置 0600 权限（Unix；Windows 上由 ACL 控制）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = fs::metadata(path)?.permissions();
        perm.set_mode(0o600);
        fs::set_permissions(path, perm)?;
    }

    Ok(())
}
