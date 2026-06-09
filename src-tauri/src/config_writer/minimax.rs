//! MiniMax 配置写入器（CLI + Desktop 共用）

use super::{ConfigWriter, WriteContext};

pub struct MiniMaxWriter;

impl ConfigWriter for MiniMaxWriter {
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String> {
        let path = crate::common::path_util::minimax_config_path()
            .ok_or("无法找到用户目录")?;
        // 创建目录
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }
        crate::minimax_config::apply_provider(
            &path,
            ctx.provider_id,
            ctx.provider_name,
            ctx.base_url,
            ctx.model_name,
            ctx.api_key,
        )
        .map_err(|e| format!("MiniMax 配置写入失败: {}", e))
    }
}
