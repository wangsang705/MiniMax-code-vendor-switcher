//! Zed 编辑器配置写入器
//!
//! Zed 通过 settings.json 的 language_models.openai 块配置自定义 AI 端点。
//! 配置文件 (Windows): %APPDATA%/Zed/settings.json
//! 配置文件 (macOS/Linux): ~/.config/zed/settings.json

use super::{ConfigWriter, WriteContext};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::path::PathBuf;

pub struct ZedWriter;

impl ConfigWriter for ZedWriter {
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String> {
        apply_zed(ctx)
    }
}

fn zed_settings_path() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA")
            .map(|p| PathBuf::from(p).join("Zed").join("settings.json"))
            .ok_or_else(|| "APPDATA 环境变量未设置".to_string())
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME")
            .map(|p| PathBuf::from(p).join(".config").join("zed").join("settings.json"))
            .ok_or_else(|| "HOME 环境变量未设置".to_string())
    }
}

fn apply_zed(ctx: &WriteContext) -> Result<(), String> {
    let path = zed_settings_path()?;
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

    // ===== language_models.openai 配置 =====
    let lm_obj = root_obj
        .entry("language_models".to_string())
        .or_insert_with(|| JsonValue::Object(JsonMap::new()))
        .as_object_mut()
        .ok_or_else(|| "language_models 不是 object".to_string())?;

    let openai_obj = lm_obj
        .entry("openai".to_string())
        .or_insert_with(|| JsonValue::Object(JsonMap::new()))
        .as_object_mut()
        .ok_or_else(|| "openai 不是 object".to_string())?;

    openai_obj.insert("version".to_string(), JsonValue::String("1".to_string()));
    openai_obj.insert("api_url".to_string(), JsonValue::String(ctx.base_url.to_string()));

    let models = openai_obj
        .entry("available_models".to_string())
        .or_insert_with(|| JsonValue::Array(Vec::new()))
        .as_array_mut()
        .ok_or_else(|| "available_models 不是 array".to_string())?;

    // 移除同名称的旧模型
    models.retain(|m| m.get("name").and_then(JsonValue::as_str) != Some(ctx.model_name));

    let model_entry = serde_json::json!({
        "name": ctx.model_name,
        "display_name": format!("{} ({})", ctx.provider_name, ctx.model_name),
        "max_tokens": 128000
    });
    models.push(model_entry);

    // ===== assistant.default_model =====
    let assistant_obj = root_obj
        .entry("assistant".to_string())
        .or_insert_with(|| JsonValue::Object(JsonMap::new()))
        .as_object_mut()
        .ok_or_else(|| "assistant 不是 object".to_string())?;

    assistant_obj.insert("enabled".to_string(), JsonValue::Bool(true));
    assistant_obj.insert(
        "default_model".to_string(),
        serde_json::json!({
            "provider": "openai",
            "model": ctx.model_name
        }),
    );

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
