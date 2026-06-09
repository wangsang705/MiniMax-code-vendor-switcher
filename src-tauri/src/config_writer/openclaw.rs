use super::{ConfigWriter, WriteContext};

pub struct OpenClawWriter;

impl ConfigWriter for OpenClawWriter {
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String> {
        crate::agent_adapters::apply_openclaw(ctx.provider_name, ctx.base_url, ctx.model_name, ctx.api_key)
            .map_err(|e| format!("OpenClaw 配置写入失败: {}", e))
    }
}
