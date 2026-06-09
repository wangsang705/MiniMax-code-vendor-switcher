use super::{ConfigWriter, WriteContext};

pub struct KimiWriter;

impl ConfigWriter for KimiWriter {
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String> {
        crate::tool_configs::apply_kimi(ctx.provider_id, ctx.provider_name, ctx.base_url, ctx.model_name, ctx.api_key, ctx.anthropic_mode)
            .map_err(|e| format!("Kimi 配置写入失败: {}", e))
    }
}
