use tauri_app_lib::minimax_config;
use tempfile::tempdir;

#[test]
fn test_apply_provider_new_creates_entry() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.yaml");

    minimax_config::apply_provider(&path, "deepseek", "DeepSeek", "https://api.deepseek.com", "deepseek-chat", "sk-test123")
        .unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("deepseek:"), "应包含 deepseek provider");
    assert!(content.contains("sk-test123"), "应包含 apiKey");
    assert!(content.contains("https://api.deepseek.com"), "应包含 baseURL");
    assert!(content.contains("defaultModel: deepseek/deepseek-chat"), "应更新 defaultModel");

    // 关键验证：模型条目必须使用 name: 字段名，而不是模型名本身
    // 正确:  deepseek-chat:\n        name: deepseek-chat
    // 错误:  deepseek-chat:\n        deepseek-chat: deepseek-chat
    assert!(
        content.contains("name: deepseek-chat"),
        "模型条目必须包含 'name:' 字段 (MiniMax 识别要求)\n生成的内容:\n{}",
        content
    );
}

#[test]
fn test_apply_provider_update_overwrites_options() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.yaml");

    minimax_config::apply_provider(&path, "deepseek", "DeepSeek", "https://api.deepseek.com/v1", "deepseek-chat", "sk-old")
        .unwrap();
    minimax_config::apply_provider(&path, "deepseek", "DeepSeek", "https://api.deepseek.com/v2", "deepseek-chat", "sk-new")
        .unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("sk-new"));
    assert!(content.contains("https://api.deepseek.com/v2"));
    assert!(!content.contains("sk-old"));
}

#[test]
fn test_apply_provider_preserves_existing_sections() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.yaml");

    // 模拟现有的完整 MiniMax 配置（含 nexus 等额外字段）
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
    std::fs::write(&path, initial).unwrap();

    // 应用新 provider
    minimax_config::apply_provider(&path, "deepseek", "DeepSeek", "https://api.deepseek.com", "deepseek-chat", "sk-ds")
        .unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    // 原有 minimax provider 还在
    assert!(content.contains("minimax:"), "应保留 minimax provider");
    assert!(content.contains("sk-xxx"), "应保留 minimax apiKey");
    // nexus 段还在
    assert!(content.contains("nexus:"), "应保留 nexus 段");
    assert!(content.contains("providerID: minimax"), "应保留 nexus model");
    // 新 provider 已添加
    assert!(content.contains("deepseek:"), "应添加 deepseek provider");
    assert!(content.contains("sk-ds"), "应写入 deepseek apiKey");
    // defaultModel 已更新
    assert!(content.contains("defaultModel: deepseek/deepseek-chat"), "应更新 defaultModel");
}

#[test]
fn test_backup_created() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.yaml");

    // 第一次写入
    minimax_config::apply_provider(&path, "deepseek", "DeepSeek", "https://api.deepseek.com", "deepseek-chat", "sk-1")
        .unwrap();
    // 第二次写入（触发备份）
    minimax_config::apply_provider(&path, "deepseek", "DeepSeek", "https://api.deepseek.com", "deepseek-chat", "sk-2")
        .unwrap();

    // 备份目录应至少有一个文件
    let backup_dir = dir.path().join("backups");
    let entries: Vec<_> = std::fs::read_dir(&backup_dir).unwrap().collect();
    assert!(entries.len() >= 1, "应至少有一个备份文件");
}
