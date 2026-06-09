use super::{ConfigWriter, WriteContext};

pub struct HermesWriter;

impl ConfigWriter for HermesWriter {
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String> {
        crate::agent_adapters::apply_hermes(ctx.provider_id, ctx.base_url, ctx.model_name, ctx.api_key)
            .map_err(|e| format!("Hermes Agent 配置写入失败: {}", e))
    }
}
