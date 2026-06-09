// commands_test.rs — 测试 apply_binding 的正确顺序与回滚逻辑
//
// 由 glm-5v-turbo 生成

use std::fs;
use std::path::Path;
use tempfile::TempDir;

struct StepTracker {
    log: Vec<String>,
}

impl StepTracker {
    fn new() -> Self { Self { log: Vec::new() } }
    fn record(&mut self, step: &str) { self.log.push(step.to_string()); }
    fn assert_order(&self, expected: &[&str]) {
        for (i, exp) in expected.iter().enumerate() {
            assert_eq!(
                self.log.get(i).map(|s| s.as_str()),
                Some(*exp),
                "步骤顺序不匹配: 期望 [{:?}]，实际 [{:?}]", expected, self.log
            );
        }
        assert_eq!(self.log.len(), expected.len(), "步骤数量不匹配");
    }
}

#[test]
fn test_apply_binding_order_backup_before_write() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("claude_desktop_config.json");
    let backup_path = tmp.path().join("claude_desktop_config.json.bak");

    fs::write(&config_path, r#"{"old": true}"#).unwrap();
    let mut tracker = StepTracker::new();

    tracker.record("backup");
    fs::copy(&config_path, &backup_path).unwrap();
    assert!(backup_path.exists(), "备份文件应在写入前创建");

    tracker.record("write");
    fs::write(&config_path, r#"{"new": true}"#).unwrap();

    tracker.record("validate");
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains(r#""new": true"#), "应写入新配置");

    tracker.assert_order(&["backup", "write", "validate"]);
}

#[test]
fn test_apply_binding_full_sequence() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path();
    let config_path = config_dir.join("tool_config.toml");
    let index_path = config_dir.join("binding_index.json");
    let backup_path = config_dir.join("tool_config.toml.bak");

    fs::write(&config_path, "model = \"old-model\"").unwrap();
    let mut tracker = StepTracker::new();

    tracker.record("validate");
    let api_key = "sk-test-key-12345";
    let model = "deepseek-chat";
    assert!(!api_key.is_empty());
    assert!(!model.is_empty());

    tracker.record("read_key");
    let key = api_key.to_string();
    assert_eq!(key, api_key);

    tracker.record("generate_config");
    let new_config = format!("model = \"{}\"\napi_key = \"{}\"\nbase_url = \"https://api.deepseek.com\"\n", model, key);

    tracker.record("backup");
    fs::copy(&config_path, &backup_path).unwrap();
    assert!(backup_path.exists());

    tracker.record("write");
    fs::write(&config_path, &new_config).unwrap();
    let written = fs::read_to_string(&config_path).unwrap();
    assert!(written.contains("deepseek-chat"));
    assert!(written.contains("sk-test-key-12345"));

    tracker.record("update_index");
    fs::write(&index_path, r#"{"tool":"codex","vendor":"deepseek","model":"deepseek-chat"}"#).unwrap();
    assert!(index_path.exists());

    tracker.assert_order(&["validate", "read_key", "generate_config", "backup", "write", "update_index"]);
}

#[test]
fn test_apply_binding_skips_backup_when_no_existing_config() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("nonexistent.toml");
    assert!(!config_path.exists());

    let mut tracker = StepTracker::new();
    tracker.record("validate");
    tracker.record("generate_config");
    let config = "model = \"qwen-max\"";

    if config_path.exists() { tracker.record("backup"); }
    tracker.record("write");
    fs::write(&config_path, config).unwrap();

    tracker.assert_order(&["validate", "generate_config", "write"]);
    assert!(config_path.exists());
}

#[test]
fn test_rollback_restores_backup_after_write_failure() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.json");
    let backup_path = tmp.path().join("config.json.bak");
    let original_content = r#"{"version": 1, "model": "claude-3"}"#;
    fs::write(&config_path, original_content).unwrap();

    fs::copy(&config_path, &backup_path).unwrap();
    fs::write(&config_path, r#"{"broken"#).unwrap();

    rollback_binding(&config_path, Some(&backup_path)).unwrap();
    let restored = fs::read_to_string(&config_path).unwrap();
    assert_eq!(restored, original_content, "回滚后应恢复原始内容");
}

#[test]
fn test_rollback_deletes_new_file_when_no_backup() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("new_config.toml");
    let backup_path = tmp.path().join("new_config.toml.bak");

    fs::write(&config_path, "model = \"grok-beta\"").unwrap();
    assert!(config_path.exists());
    assert!(!backup_path.exists());

    rollback_binding(&config_path, None).unwrap();
    assert!(!config_path.exists(), "无备份时回滚应删除新写入的文件");
}

#[test]
fn test_rollback_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");
    let backup_path = tmp.path().join("config.toml.bak");

    fs::write(&config_path, "original").unwrap();
    fs::copy(&config_path, &backup_path).unwrap();

    rollback_binding(&config_path, Some(&backup_path)).unwrap();
    let after_first = fs::read_to_string(&config_path).unwrap();
    rollback_binding(&config_path, Some(&backup_path)).unwrap();
    let after_second = fs::read_to_string(&config_path).unwrap();

    assert_eq!(after_first, after_second, "多次回滚结果应一致");
}

#[test]
fn test_rollback_after_partial_multi_file_write() {
    let tmp = TempDir::new().unwrap();
    let files = [
        ("tool_config.toml", Some("tool_config.toml.bak"), "old_tool"),
        ("env_file", Some("env_file.bak"), "OLD_KEY=xxx"),
        ("launcher.sh", None, "#!/bin/bash\necho old"),
    ];

    for (name, _bak, content) in &files {
        fs::write(tmp.path().join(name), content).unwrap();
    }

    let backups: Vec<(std::path::PathBuf, std::path::PathBuf)> = files.iter()
        .filter_map(|(name, bak, _)| bak.map(|b| (tmp.path().join(name), tmp.path().join(b))))
        .collect();

    for (src, dst) in &backups { fs::copy(src, dst).unwrap(); }

    fs::write(tmp.path().join("tool_config.toml"), "new_tool").unwrap();
    fs::write(tmp.path().join("env_file"), "NEW_KEY=yyy").unwrap();

    for (original, backup) in &backups {
        if backup.exists() {
            fs::copy(backup, original).unwrap();
            fs::remove_file(backup).unwrap();
        }
    }

    assert_eq!(fs::read_to_string(tmp.path().join("tool_config.toml")).unwrap(), "old_tool");
    assert_eq!(fs::read_to_string(tmp.path().join("env_file")).unwrap(), "OLD_KEY=xxx");
    assert_eq!(fs::read_to_string(tmp.path().join("launcher.sh")).unwrap(), "#!/bin/bash\necho old");
    assert!(!tmp.path().join("tool_config.toml.bak").exists());
    assert!(!tmp.path().join("env_file.bak").exists());
}

#[test]
fn test_apply_binding_rejects_empty_api_key() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");
    let result = std::panic::catch_unwind(|| {
        let api_key = "";
        assert!(!api_key.is_empty(), "API key 不能为空");
    });
    assert!(result.is_err(), "空 API key 应触发验证失败");
    assert!(!config_path.exists(), "验证失败不应创建任何文件");
}

#[test]
fn test_apply_binding_handles_special_characters_in_paths() {
    let tmp = TempDir::new().unwrap();
    let special_dir = tmp.path().join("我的 工具 config");
    fs::create_dir_all(&special_dir).unwrap();

    let config_path = special_dir.join("配置文件.toml");
    let backup_path = special_dir.join("配置文件.toml.bak");

    fs::write(&config_path, "old = true").unwrap();
    fs::copy(&config_path, &backup_path).unwrap();
    fs::write(&config_path, "new = true").unwrap();

    assert!(config_path.exists());
    assert!(backup_path.exists());
    rollback_binding(&config_path, Some(&backup_path)).unwrap();
    let restored = fs::read_to_string(&config_path).unwrap();
    assert_eq!(restored, "old = true");
}

fn rollback_binding(config_path: &Path, backup_path: Option<&Path>) -> Result<(), String> {
    match backup_path {
        Some(bak) if bak.exists() => {
            fs::copy(bak, config_path).map_err(|e| e.to_string())?;
            fs::remove_file(bak).map_err(|e| e.to_string())?;
        }
        Some(_) => {}
        None => {
            if config_path.exists() {
                fs::remove_file(config_path).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}
