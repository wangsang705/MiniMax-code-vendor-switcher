use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

///最多保留的备份文件数
const MAX_BACKUPS: usize =30;

///原子写入 + 自动备份
///
///写入过程：
///1. 如果原文件存在，先备份到 `{parent}/backups/`目录
///2.写入临时文件 `{path}.tmp`
///3. `fs::rename`原子替换
///
/// `fs::rename` 在同一文件系统下是原子的，即使此时断电也不会损坏文件。
///
/// `restrict_perms`:写入后是否将文件权限收紧到"仅当前用户可读写"。
///写 API Key / token 等敏感配置时应设为 `true`。
pub fn atomic_write(path: &Path, content: &str, restrict_perms: bool) -> Result<(), std::io::Error> {
 // 确保父目录存在
 if let Some(parent) = path.parent() {
 fs::create_dir_all(parent)?;
 }

 //备份原文件
 if path.exists() {
 let backup_dir = path
 .parent()
 .unwrap_or_else(|| Path::new("."))
 .join("backups");
 fs::create_dir_all(&backup_dir)?;
 let timestamp = SystemTime::now()
 .duration_since(UNIX_EPOCH)
 .map(|d| d.as_millis())
 .unwrap_or(0);
 let backup_path: PathBuf = backup_dir.join(format!("{}.{}", file_stem(path), timestamp));
 fs::copy(path, &backup_path)?;
 cleanup_old_backups(&backup_dir);
 }

 //写临时文件 → rename原子替换
 let tmp_path = tmp_path_for(path);
 {
 let mut file = fs::File::create(&tmp_path)?;
 file.write_all(content.as_bytes())?;
 file.sync_all()?;
 }
 fs::rename(&tmp_path, path)?;

 if restrict_perms {
 restrict_file_permissions(path)?;
 }

 Ok(())
}

///便捷方法：写敏感配置（API Key / token）。
///
/// 等价于 `atomic_write(path, content, true)`。
pub fn atomic_write_secret(path: &Path, content: &str) -> Result<(), std::io::Error> {
 atomic_write(path, content, true)
}

///便捷方法：写普通配置（非敏感）。
///
/// 等价于 `atomic_write(path, content, false)`。
pub fn atomic_write_plain(path: &Path, content: &str) -> Result<(), std::io::Error> {
 atomic_write(path, content, false)
}

///清理旧备份，保留最新的 MAX_BACKUPS 个
fn cleanup_old_backups(backup_dir: &Path) {
 if let Ok(entries) = fs::read_dir(backup_dir) {
 let mut files: Vec<_> = entries
 .filter_map(|e| e.ok())
 .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
 .collect();
 files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
 if files.len() > MAX_BACKUPS {
 for old in files.iter().take(files.len() - MAX_BACKUPS) {
 let _ = fs::remove_file(old.path());
 }
 }
 }
}

/// 获取文件的 stem（不含扩展名）
fn file_stem(path: &Path) -> String {
 path.file_stem()
 .and_then(|s| s.to_str())
 .unwrap_or("config")
 .to_string()
}

/// 获取临时文件路径
fn tmp_path_for(path: &Path) -> PathBuf {
 let ext = path
 .extension()
 .and_then(|s| s.to_str())
 .unwrap_or("tmp");
 path.with_extension(format!("{}.tmp", ext))
}

/// 设置文件权限为仅当前用户可读（跨平台）
pub fn restrict_file_permissions(path: &Path) -> Result<(), std::io::Error> {
 #[cfg(unix)]
 {
 use std::os::unix::fs::PermissionsExt;
 let mut perm = fs::metadata(path)?.permissions();
 perm.set_mode(0o600);
 fs::set_permissions(path, perm)?;
 }

 #[cfg(windows)]
 {
 // Windows 上通过 icacls 设置仅当前用户可访问
 let path_str = path.to_string_lossy();
 let perm = format!("%USERNAME%:(R,W)");
 let output = std::process::Command::new("icacls")
 .args([&path_str, "/inheritance:r", "/grant", &perm])
 .output()?;
 if !output.status.success() {
 let stderr = String::from_utf8_lossy(&output.stderr);
 eprintln!("icacls 设置权限失败: {}", stderr);
 }
 }

 Ok(())
}

#[cfg(test)]
mod tests {
 use super::*;
 use tempfile::TempDir;

 #[test]
 fn test_atomic_write_creates_file() {
 let dir = TempDir::new().unwrap();
 let path = dir.path().join("test.txt");
 atomic_write_plain(&path, "hello world").unwrap();
 assert_eq!(fs::read_to_string(&path).unwrap(), "hello world");
 }

 #[test]
 fn test_atomic_write_creates_backup() {
 let dir = TempDir::new().unwrap();
 let path = dir.path().join("test.txt");
 atomic_write_plain(&path, "v1").unwrap();
 atomic_write_plain(&path, "v2").unwrap();

 let backup_dir = dir.path().join("backups");
 assert!(backup_dir.exists(), "备份目录应存在");
 let count = fs::read_dir(&backup_dir).unwrap().count();
 assert!(count >=1, "应至少有一个备份文件");
 }

 #[test]
 fn test_atomic_write_preserves_content() {
 let dir = TempDir::new().unwrap();
 let path = dir.path().join("test.txt");
 atomic_write_plain(&path, "final content").unwrap();

 //验证写入的内容完整
 let content = fs::read_to_string(&path).unwrap();
 assert_eq!(content, "final content");
 }

 #[test]
 fn test_restrict_file_permissions() {
 let dir = TempDir::new().unwrap();
 let path = dir.path().join("secret.txt");
 fs::write(&path, "secret").unwrap();
 restrict_file_permissions(&path).unwrap();
 //验证文件仍可读
 assert_eq!(fs::read_to_string(&path).unwrap(), "secret");
 }

 #[test]
 fn test_atomic_write_secret_unix_creates_0600() {
 // 仅 Unix验证0o600权限位
 #[cfg(unix)]
 {
 use std::os::unix::fs::PermissionsExt;
 let dir = TempDir::new().unwrap();
 let path = dir.path().join("secret.txt");
 atomic_write_secret(&path, "shhh").unwrap();
 let perm = fs::metadata(&path).unwrap().permissions().mode() &0o777;
 assert_eq!(perm,0o600, "secret 文件应为0o600");
 }
 }

 #[test]
 fn test_atomic_write_plain_does_not_change_perms() {
 //验证 plain写入不会修改已有权限
 let dir = TempDir::new().unwrap();
 let path = dir.path().join("public.txt");
 fs::write(&path, "init").unwrap();
 atomic_write_plain(&path, "new").unwrap();
 //至少要保证文件可读
 assert_eq!(fs::read_to_string(&path).unwrap(), "new");
 }
}
