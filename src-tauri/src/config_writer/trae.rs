//! Trae IDE 配置写入器
//!
//! Trae 是字节跳动的 AI IDE，支持自定义 OpenAI 兼容 API。
//! CLI 配置: ~/.trae/trae_cli.yaml
//! IDE 配置: ~/.trae/settings.json
//!
//! 优先写入 trae_cli.yaml，同时更新 settings.json 的默认模型。

use super::{ConfigWriter, WriteContext};
use serde_yaml::{Mapping as YamlMap, Value as YamlValue};
use std::fs;
use std::path::PathBuf;

pub struct TraeWriter;

impl ConfigWriter for TraeWriter {
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String> {
        apply_trae(ctx)
    }
}

fn trae_yaml_path() -> Result<PathBuf, String> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .ok_or_else(|| "无法找到用户目录".to_string())?;
    Ok(home.join(".trae").join("trae_cli.yaml"))
}

fn trae_settings_path() -> Result<PathBuf, String> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .ok_or_else(|| "无法找到用户目录".to_string())?;
    Ok(home.join(".trae").join("settings.json"))
}

fn apply_trae(ctx: &WriteContext) -> Result<(), String> {
    let yaml_path = trae_yaml_path()?;
    let settings_path = trae_settings_path()?;

    if let Some(parent) = yaml_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("无法创建目录: {}", e))?;
    }

    // ===== 1. 写入 trae_cli.yaml =====
    let content = if yaml_path.exists() {
        fs::read_to_string(&yaml_path).map_err(|e| format!("读取失败: {}", e))?
    } else {
        String::new()
    };

    let mut root: YamlValue = if content.trim().is_empty() {
        YamlValue::Mapping(YamlMap::new())
    } else {
        serde_yaml::from_str(&content).map_err(|e| format!("YAML 解析失败: {}", e))?
    };

    let root_map = root.as_mapping_mut().ok_or_else(|| "根节点不是 mapping".to_string())?;

    let models_seq = root_map
        .entry(YamlValue::String("models".to_string()))
        .or_insert_with(|| YamlValue::Sequence(Vec::new()))
        .as_sequence_mut()
        .ok_or_else(|| "models 不是 sequence".to_string())?;

    // 移除同模型的旧条目
    models_seq.retain(|m| {
        m.get("open_ai")
            .and_then(|o| o.get("model"))
            .and_then(YamlValue::as_str)
            != Some(ctx.model_name)
        && m.get("claude")
            .and_then(|c| c.get("model"))
            .and_then(YamlValue::as_str)
            != Some(ctx.model_name)
    });

    let mut model_map = YamlMap::new();
    model_map.insert(
        YamlValue::String("name".to_string()),
        YamlValue::String(ctx.model_name.to_string()),
    );

    if ctx.anthropic_mode {
        let mut claude = YamlMap::new();
        claude.insert(YamlValue::String("base_url".to_string()), YamlValue::String(ctx.base_url.to_string()));
        claude.insert(YamlValue::String("model".to_string()), YamlValue::String(ctx.model_name.to_string()));
        claude.insert(YamlValue::String("api_key".to_string()), YamlValue::String(ctx.api_key.to_string()));
        model_map.insert(YamlValue::String("claude".to_string()), YamlValue::Mapping(claude));
    } else {
        let mut openai = YamlMap::new();
        openai.insert(YamlValue::String("base_url".to_string()), YamlValue::String(ctx.base_url.to_string()));
        openai.insert(YamlValue::String("model".to_string()), YamlValue::String(ctx.model_name.to_string()));
        openai.insert(YamlValue::String("api_key".to_string()), YamlValue::String(ctx.api_key.to_string()));
        openai.insert(YamlValue::String("by_azure".to_string()), YamlValue::Bool(false));
        model_map.insert(YamlValue::String("open_ai".to_string()), YamlValue::Mapping(openai));
    }

    models_seq.push(YamlValue::Mapping(model_map));

    let serialized = serde_yaml::to_string(&root).map_err(|e| format!("YAML 序列化失败: {}", e))?;
    let tmp_path = yaml_path.with_extension("tmp");
    {
        let mut file = fs::File::create(&tmp_path).map_err(|e| format!("创建临时文件失败: {}", e))?;
        use std::io::Write;
        file.write_all(serialized.as_bytes()).map_err(|e| format!("写入失败: {}", e))?;
        file.sync_all().map_err(|e| format!("同步失败: {}", e))?;
    }
    fs::rename(&tmp_path, &yaml_path).map_err(|e| format!("重命名失败: {}", e))?;

    // ===== 2. 写入 settings.json（设置默认模型） =====
    let settings_content = if settings_path.exists() {
        fs::read_to_string(&settings_path).map_err(|e| format!("读取 settings 失败: {}", e))?
    } else {
        String::new()
    };

    let mut settings: serde_json::Value = if settings_content.trim().is_empty() {
        serde_json::Value::Object(serde_json::Map::new())
    } else {
        serde_json::from_str(&settings_content).map_err(|e| format!("settings JSON 解析失败: {}", e))?
    };

    if let Some(obj) = settings.as_object_mut() {
        obj.insert(
            "trae.ai.model".to_string(),
            serde_json::Value::String(ctx.model_name.to_string()),
        );
        obj.insert(
            "trae.ai.enabled".to_string(),
            serde_json::Value::Bool(true),
        );
    }

    let settings_serialized = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    let settings_tmp = settings_path.with_extension("tmp");
    {
        let mut file = fs::File::create(&settings_tmp).map_err(|e| format!("创建临时文件失败: {}", e))?;
        use std::io::Write;
        file.write_all(settings_serialized.as_bytes()).map_err(|e| format!("写入失败: {}", e))?;
        file.sync_all().map_err(|e| format!("同步失败: {}", e))?;
    }
    fs::rename(&settings_tmp, &settings_path).map_err(|e| format!("重命名失败: {}", e))?;

    Ok(())
}
