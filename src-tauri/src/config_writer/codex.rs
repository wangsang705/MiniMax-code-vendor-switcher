//! Codex 配置写入器
//!
//! codex-cli: 写入 ~/.codex/config.toml
//! codex-desktop: 写入 %APPDATA%/Codex/config.toml (Windows)

use super::{ConfigWriter, WriteContext};

pub struct CodexWriter;

impl ConfigWriter for CodexWriter {
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String> {
        let path = crate::tool_configs::codex_config_path()
            .map_err(|e| format!("获取 Codex 配置路径失败: {}", e))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }
        crate::tool_configs::apply_codex_to_path(
            &path,
            ctx.provider_id,
            ctx.provider_name,
            ctx.base_url,
            ctx.model_name,
            ctx.api_key,
            ctx.anthropic_mode,
        )
        .map_err(|e| format!("Codex 配置写入失败: {}", e))
    }
}

/// Codex Desktop 写入器
///
/// Codex Desktop 需要读取 %APPDATA%/Codex/config.toml 来获取模型提供商列表。
/// 写入和 codex-cli 相同的 TOML 结构，但路径不同。
pub struct CodexDesktopWriter;

impl ConfigWriter for CodexDesktopWriter {
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String> {
        let path = crate::tool_configs::codex_desktop_config_path()
            .map_err(|e| format!("获取 Codex Desktop 配置路径失败: {}", e))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }
        // 重用 codex-cli 相同的 TOML 内容生成逻辑
        crate::tool_configs::apply_codex_to_path(
            &path,
            ctx.provider_id,
            ctx.provider_name,
            ctx.base_url,
            ctx.model_name,
            ctx.api_key,
            ctx.anthropic_mode,
        )
        .map_err(|e| format!("Codex Desktop 配置写入失败: {}", e))
    }
}
