use super::{ConfigWriter, WriteContext};

pub struct NanoBotWriter;

impl ConfigWriter for NanoBotWriter {
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String> {
        crate::agent_adapters::apply_nanobot(ctx.provider_id, ctx.base_url, ctx.model_name, ctx.api_key)
            .map_err(|e| format!("Nanobot 配置写入失败: {}", e))
    }
}
