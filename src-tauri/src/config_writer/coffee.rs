//! Coffee CLI 配置写入器
//!
//! Coffee CLI 是一个 OpenAI 兼容的 AI 编码 CLI 工具。
//! 配置文件: ~/.coffee-cli/config.json

use super::{ConfigWriter, WriteContext};

pub struct CoffeeWriter;

impl ConfigWriter for CoffeeWriter {
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String> {
        crate::tool_configs::apply_coffee(
            ctx.provider_id,
            ctx.provider_name,
            ctx.base_url,
            ctx.model_name,
            ctx.api_key,
            ctx.anthropic_mode,
        )
        .map_err(|e| format!("Coffee CLI 配置写入失败: {}", e))
    }
}
