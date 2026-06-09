//! Qwen Code CLI 配置写入器

use super::{ConfigWriter, WriteContext};

pub struct QwenWriter;

impl ConfigWriter for QwenWriter {
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String> {
        crate::tool_configs::apply_qwen(
            ctx.provider_id,
            ctx.provider_name,
            ctx.base_url,
            ctx.model_name,
            ctx.api_key,
            ctx.anthropic_mode,
        )
        .map_err(|e| format!("Qwen 配置写入失败: {}", e))
    }
}
