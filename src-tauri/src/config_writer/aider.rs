//! Aider 配置写入器

use super::{ConfigWriter, WriteContext};

pub struct AiderWriter;

impl ConfigWriter for AiderWriter {
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String> {
        crate::tool_configs::apply_aider(
            ctx.base_url,
            ctx.model_name,
            ctx.api_key,
            ctx.anthropic_mode,
        )
        .map_err(|e| format!("Aider 配置写入失败: {}", e))
    }
}
