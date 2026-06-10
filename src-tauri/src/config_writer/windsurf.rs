//! Windsurf IDE 配置写入器
//!
//! Windsurf 使用网页端 BYOK 配置 API Key，不支持 settings.json 自定义端点。
//! 我们写入 ~/.codeium/windsurf/config.json 作为环境变量提示，
//! 实际 API 切换通过启动时注入环境变量实现。
//!
//! 配置文件: ~/.codeium/windsurf/config.json

use super::{ConfigWriter, WriteContext};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::path::PathBuf;

pub struct WindsurfWriter;

impl ConfigWriter for WindsurfWriter {
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String> {
        apply_windsurf(ctx)
    }
}

fn windsurf_config_path() -> Result<PathBuf, String> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .ok_or_else(|| "无法找到用户目录".to_string())?;
    Ok(home.join(".codeium").join("windsurf").join("config.json"))
}

fn apply_windsurf(ctx: &WriteContext) -> Result<(), String> {
    let path = windsurf_config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("无法创建目录: {}", e))?;
    }

    let content = if path.exists() {
        fs::read_to_string(&path).map_err(|e| format!("读取失败: {}", e))?
    } else {
        String::new()
    };

    let mut root: JsonValue = if content.trim().is_empty() {
        JsonValue::Object(JsonMap::new())
    } else {
        serde_json::from_str(&content).map_err(|e| format!("JSON 解析失败: {}", e))?
    };

    let root_obj = root.as_object_mut().ok_or_else(|| "根节点不是 object".to_string())?;

    // 写入厂商配置（Windsurf 启动时注入环境变量）
    root_obj.insert("provider".to_string(), JsonValue::String(ctx.provider_id.to_string()));
    root_obj.insert("model".to_string(), JsonValue::String(ctx.model_name.to_string()));

    let env_obj = root_obj
        .entry("env".to_string())
        .or_insert_with(|| JsonValue::Object(JsonMap::new()))
        .as_object_mut()
        .ok_or_else(|| "env 不是 object".to_string())?;

    if ctx.anthropic_mode {
        env_obj.insert("ANTHROPIC_BASE_URL".to_string(), JsonValue::String(ctx.base_url.to_string()));
        env_obj.insert("ANTHROPIC_AUTH_TOKEN".to_string(), JsonValue::String(ctx.api_key.to_string()));
    } else {
        env_obj.insert("OPENAI_API_KEY".to_string(), JsonValue::String(ctx.api_key.to_string()));
        env_obj.insert("OPENAI_BASE_URL".to_string(), JsonValue::String(ctx.base_url.to_string()));
    }

    let serialized = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    let tmp_path = path.with_extension("tmp");
    {
        let mut file = fs::File::create(&tmp_path).map_err(|e| format!("创建临时文件失败: {}", e))?;
        use std::io::Write;
        file.write_all(serialized.as_bytes()).map_err(|e| format!("写入失败: {}", e))?;
        file.sync_all().map_err(|e| format!("同步失败: {}", e))?;
    }
    fs::rename(&tmp_path, &path).map_err(|e| format!("重命名失败: {}", e))?;
    Ok(())
}
