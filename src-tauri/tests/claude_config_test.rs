use tauri_app_lib::claude_config::{read_settings, write_env_atomic, ClaudeSettings};
use std::collections::HashMap;
use tempfile::tempdir;

#[test]
fn test_write_env_merges_existing() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("settings.json");

    // 初始写入
    let mut initial = ClaudeSettings::default();
    let mut env = HashMap::new();
    env.insert("FOO".into(), "bar".into());
    initial.env = Some(env);
    write_env_atomic(&path, &initial).unwrap();

    // 修改 env
    let current = read_settings(&path).unwrap();
    let mut updated = current.clone();
    let mut new_env = updated.env.clone().unwrap_or_default();
    new_env.insert("ANTHROPIC_BASE_URL".into(), "https://api.deepseek.com".into());
    new_env.insert("ANTHROPIC_AUTH_TOKEN".into(), "sk-test".into());
    updated.env = Some(new_env);
    write_env_atomic(&path, &updated).unwrap();

    // 验证
    let final_settings = read_settings(&path).unwrap();
    let env = final_settings.env.unwrap();
    assert_eq!(env.get("FOO").unwrap(), "bar");
    assert_eq!(env.get("ANTHROPIC_BASE_URL").unwrap(), "https://api.deepseek.com");
    assert_eq!(env.get("ANTHROPIC_AUTH_TOKEN").unwrap(), "sk-test");
}

#[test]
fn test_backup_created() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("settings.json");
    write_env_atomic(&path, &ClaudeSettings::default()).unwrap();
    write_env_atomic(&path, &ClaudeSettings::default()).unwrap();

    // 备份目录应至少有一个文件
    let backup_dir = dir.path().join("backups");
    let entries: Vec<_> = std::fs::read_dir(&backup_dir).unwrap().collect();
    assert!(entries.len() >= 1, "应至少有一个备份文件");
}
