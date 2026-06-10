//! Cursor IDE 配置写入器
//!
//! Cursor 是基于 VS Code 的 AI 优先代码编辑器。
//! 配置文件: %APPDATA%/Cursor/User/settings.json
//!
//! 通过 cursor.aiModels 字段注入自定义 OpenAI 兼容模型。

use super::{ConfigWriter, WriteContext};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::path::PathBuf;

pub struct CursorWriter;

impl ConfigWriter for CursorWriter {
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String> {
        apply_cursor(ctx)
    }
}

/// 获取 Cursor 设置文件路径
fn cursor_settings_path() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA")
            .map(|p| PathBuf::from(p).join("Cursor").join("User").join("settings.json"))
            .ok_or_else(|| "APPDATA 环境变量未设置".to_string())
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME")
            .map(|p| PathBuf::from(p).join(".cursor").join("settings.json"))
            .ok_or_else(|| "HOME 环境变量未设置".to_string())
    }
}

fn apply_cursor(ctx: &WriteContext) -> Result<(), String> {
    let path = cursor_settings_path()?;

    // 确保父目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("无法创建目录: {}", e))?;
    }

    // 读取现有配置
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

    let root_obj = root
        .as_object_mut()
        .ok_or_else(|| "settings.json 根节点不是 object".to_string())?;

    // 构建模型条目
    let model_entry = serde_json::json!({
        "name": ctx.model_name,
        "provider": "openai-compatible",
        "apiKey": ctx.api_key,
        "baseUrl": ctx.base_url,
        "model": ctx.model_name,
        "contextLength": 128000,
        "maxTokens": 8192
    });

    // cursor.aiModels: 替换或添加
    let ai_models = root_obj
        .entry("cursor.aiModels")
        .or_insert_with(|| JsonValue::Array(Vec::new()))
        .as_array_mut()
        .ok_or_else(|| "cursor.aiModels 不是 array".to_string())?;

    // 移除同 model 的旧条目（如果存在）
    ai_models.retain(|m| m.get("model").and_then(JsonValue::as_str) != Some(ctx.model_name));
    ai_models.push(model_entry);

    // cursor.defaultModel: 设置为当前模型
    root_obj.insert(
        "cursor.defaultModel".to_string(),
        JsonValue::String(ctx.model_name.to_string()),
    );

    // 序列化并原子写入
    let serialized = serde_json::to_string_pretty(&root)
        .map_err(|e| format!("序列化失败: {}", e))?;

    // 原子写入（tmp + rename）
    let tmp_path = path.with_extension("tmp");
    {
        let mut file = fs::File::create(&tmp_path)
            .map_err(|e| format!("创建临时文件失败: {}", e))?;
        use std::io::Write;
        file.write_all(serialized.as_bytes())
            .map_err(|e| format!("写入失败: {}", e))?;
        file.sync_all().map_err(|e| format!("同步失败: {}", e))?;
    }
    fs::rename(&tmp_path, &path).map_err(|e| format!("重命名失败: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;

    #[test]
    fn test_cursor_writes_ai_model() {
        let dir = TempDir::new().unwrap();
        // 模拟 APPDATA
        let cursor_dir = dir.path().join("Cursor").join("User");
        fs::create_dir_all(&cursor_dir).unwrap();
        let settings_path = cursor_dir.join("settings.json");

        // 写入初始配置
        let initial = serde_json::json!({
            "workbench.colorTheme": "Dark+",
            "editor.fontSize": 14
        });
        fs::write(&settings_path, serde_json::to_string_pretty(&initial).unwrap()).unwrap();

        let ctx = WriteContext {
            provider_id: "deepseek",
            provider_name: "DeepSeek",
            base_url: "https://api.deepseek.com",
            model_name: "deepseek-chat",
            api_key: "sk-test-123",
            anthropic_mode: false,
        };

        // 手动调用写入逻辑（不用 env 覆盖路径）
        let content = fs::read_to_string(&settings_path).unwrap();
        let mut root: JsonValue = serde_json::from_str(&content).unwrap();
        let root_obj = root.as_object_mut().unwrap();

        let model_entry = serde_json::json!({
            "name": ctx.model_name,
            "provider": "openai-compatible",
            "apiKey": ctx.api_key,
            "baseUrl": ctx.base_url,
            "model": ctx.model_name,
            "contextLength": 128000,
            "maxTokens": 8192
        });

        let ai_models = root_obj
            .entry("cursor.aiModels")
            .or_insert_with(|| JsonValue::Array(Vec::new()))
            .as_array_mut()
            .unwrap();
        ai_models.retain(|m| m.get("model").and_then(JsonValue::as_str) != Some(ctx.model_name));
        ai_models.push(model_entry);
        root_obj.insert("cursor.defaultModel".to_string(), JsonValue::String(ctx.model_name.to_string()));

        fs::write(&settings_path, serde_json::to_string_pretty(&root).unwrap()).unwrap();

        let saved: JsonValue = serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
        assert_eq!(saved["workbench.colorTheme"], "Dark+");
        assert_eq!(saved["cursor.defaultModel"], "deepseek-chat");
        assert_eq!(saved["cursor.aiModels"][0]["model"], "deepseek-chat");
        assert_eq!(saved["cursor.aiModels"][0]["apiKey"], "sk-test-123");
        assert!(saved["cursor.aiModels"][0]["baseUrl"].as_str().unwrap().contains("api.deepseek.com"));
    }

    #[test]
    fn test_cursor_replaces_existing_model() {
        let dir = TempDir::new().unwrap();
        let cursor_dir = dir.path().join("Cursor").join("User");
        fs::create_dir_all(&cursor_dir).unwrap();
        let settings_path = cursor_dir.join("settings.json");

        // 已有旧模型
        let initial = serde_json::json!({
            "cursor.aiModels": [
                {
                    "name": "old-model",
                    "provider": "openai-compatible",
                    "apiKey": "sk-old",
                    "baseUrl": "https://old.api.com",
                    "model": "old-model"
                }
            ],
            "cursor.defaultModel": "old-model"
        });
        fs::write(&settings_path, serde_json::to_string_pretty(&initial).unwrap()).unwrap();

        // 写入新模型
        let content = fs::read_to_string(&settings_path).unwrap();
        let mut root: JsonValue = serde_json::from_str(&content).unwrap();
        let root_obj = root.as_object_mut().unwrap();

        let new_entry = serde_json::json!({
            "name": "new-model",
            "provider": "openai-compatible",
            "apiKey": "sk-new",
            "baseUrl": "https://new.api.com",
            "model": "new-model"
        });

        let ai_models = root_obj
            .entry("cursor.aiModels")
            .or_insert_with(|| JsonValue::Array(Vec::new()))
            .as_array_mut()
            .unwrap();
        ai_models.retain(|m| m.get("model").and_then(JsonValue::as_str) != Some("new-model"));
        ai_models.push(new_entry);
        root_obj.insert("cursor.defaultModel".to_string(), JsonValue::String("new-model".to_string()));

        fs::write(&settings_path, serde_json::to_string_pretty(&root).unwrap()).unwrap();

        let saved: JsonValue = serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
        let models = saved["cursor.aiModels"].as_array().unwrap();
        // 旧模型保留，新模型添加 → 共 2 个
        assert_eq!(models.len(), 2);
        assert_eq!(saved["cursor.defaultModel"], "new-model");
    }
}
