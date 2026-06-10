//! Gemini 桌面端配置写入器
//!
//! Gemini 桌面端通过环境变量或 ~/.gemini/settings.json 配置。
//! 配置文件: ~/.gemini/settings.json

use super::{ConfigWriter, WriteContext};
use crate::common::atomic_io;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::path::PathBuf;

pub struct GeminiDesktopWriter;

impl ConfigWriter for GeminiDesktopWriter {
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String> {
        apply_gemini(ctx)
    }
}

fn gemini_config_path() -> Result<PathBuf, String> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .ok_or_else(|| "无法找到用户目录".to_string())?;
    Ok(home.join(".gemini").join("settings.json"))
}

fn apply_gemini(ctx: &WriteContext) -> Result<(), String> {
    let path = gemini_config_path()?;
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

    // 写入 env 配置（Gemini 桌面端读取环境变量）
    let env_obj = root_obj
        .entry("env".to_string())
        .or_insert_with(|| JsonValue::Object(JsonMap::new()))
        .as_object_mut()
        .ok_or_else(|| "env 不是 object".to_string())?;

    if ctx.anthropic_mode {
        env_obj.insert("ANTHROPIC_BASE_URL".to_string(), JsonValue::String(ctx.base_url.to_string()));
        env_obj.insert("ANTHROPIC_AUTH_TOKEN".to_string(), JsonValue::String(ctx.api_key.to_string()));
        env_obj.insert("ANTHROPIC_MODEL".to_string(), JsonValue::String(ctx.model_name.to_string()));
    } else {
        env_obj.insert("OPENAI_API_KEY".to_string(), JsonValue::String(ctx.api_key.to_string()));
        env_obj.insert("OPENAI_BASE_URL".to_string(), JsonValue::String(ctx.base_url.to_string()));
        env_obj.insert("OPENAI_MODEL".to_string(), JsonValue::String(ctx.model_name.to_string()));
    }

    root_obj.insert("provider".to_string(), JsonValue::String(ctx.provider_id.to_string()));
    root_obj.insert("model".to_string(), JsonValue::String(ctx.model_name.to_string()));

    let serialized = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    atomic_io::atomic_write_secret(&path, &serialized)
        .map_err(|e| format!("写入失败: {}", e))
}
