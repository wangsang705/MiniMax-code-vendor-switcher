//! Claude 配置写入器
//!
//! claude-code-cli 和 claude-desktop 共用同一路径 (~/.claude/settings.json)，
//! 所以两个 Writer 结构体共享同一个写入函数。

use std::collections::HashMap;

use super::{ConfigWriter, WriteContext};

/// Claude CLI 写入器
pub struct ClaudeCliWriter;

impl ConfigWriter for ClaudeCliWriter {
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String> {
        write_claude_config(ctx)
    }
}

/// Claude Desktop 写入器
pub struct ClaudeDesktopWriter;

impl ConfigWriter for ClaudeDesktopWriter {
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String> {
        write_claude_config(ctx)
    }
}

/// 共用写入逻辑：构造 env HashMap → merge_and_write_env
fn write_claude_config(ctx: &WriteContext) -> Result<(), String> {
    let home = crate::common::path_util::home_dir().ok_or("无法找到用户目录")?;
    let path = home.join(".claude").join("settings.json");

    // 创建父目录
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let mut updates = HashMap::new();
    updates.insert("ANTHROPIC_BASE_URL".to_string(), ctx.base_url.to_string());
    updates.insert("ANTHROPIC_AUTH_TOKEN".to_string(), ctx.api_key.to_string());
    updates.insert("ANTHROPIC_MODEL".to_string(), ctx.model_name.to_string());

    crate::claude_config::merge_and_write_env(&path, &updates)
        .map_err(|e| format!("Claude 配置写入失败: {}", e))
}
