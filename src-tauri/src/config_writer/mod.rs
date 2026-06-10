//! 配置写入层：策略模式 + 注册表
//!
//! 每个工具（或工具族）实现 `ConfigWriter` trait，通过 `WriterRegistry` 注册。
//! 新增工具只需两步：
//!   1. 在 `config_writer/` 下新建文件，实现 `ConfigWriter`
//!   2. 在 `create_default_registry()` 中加一行 `register`

mod aider;
mod claude;
mod coffee;
mod codex;
mod cursor;
mod gemini;
mod grok;
mod hermes;
mod kimi;
mod minimax;
mod nanobot;
mod openclaw;
mod opencode;
mod qwen;
mod trae;
mod windsurf;
mod zed;

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

pub use aider::AiderWriter;
pub use claude::{ClaudeCliWriter, ClaudeDesktopWriter};
pub use coffee::CoffeeWriter;
pub use codex::{CodexDesktopWriter, CodexWriter};
pub use cursor::CursorWriter;
pub use gemini::GeminiDesktopWriter;
pub use grok::GrokWriter;
pub use hermes::HermesWriter;
pub use kimi::KimiWriter;
pub use minimax::MiniMaxWriter;
pub use nanobot::NanoBotWriter;
pub use openclaw::OpenClawWriter;
pub use opencode::OpenCodeWriter;
pub use qwen::QwenWriter;
pub use trae::TraeWriter;
pub use windsurf::WindsurfWriter;
pub use zed::ZedWriter;

// ---------------------------------------------------------------------------
// WriteContext
// ---------------------------------------------------------------------------

/// 配置写入上下文 —— 一个 Writer 需要的全部信息。
#[derive(Debug)]
pub struct WriteContext<'a> {
    /// 厂商 ID（用于某些配置中的 provider 标识）
    pub provider_id: &'a str,
    /// 厂商显示名称（用于 UI 回显）
    pub provider_name: &'a str,
    /// API Base URL
    pub base_url: &'a str,
    /// 模型名
    pub model_name: &'a str,
    /// API Key
    pub api_key: &'a str,
    /// 是否使用 Anthropic 兼容协议
    pub anthropic_mode: bool,
}

// ---------------------------------------------------------------------------
// ConfigWriter trait
// ---------------------------------------------------------------------------

/// 配置写入器 trait。
///
/// 每个工具（或工具族）实现此 trait，负责将配置写入该工具对应的配置文件。
/// `Send + Sync` 确保可安全存储在全局静态注册表中。
pub trait ConfigWriter: Send + Sync {
    /// 根据 `ctx` 写入工具的配置文件。
    fn write_config(&self, ctx: &WriteContext) -> Result<(), String>;
}

// ---------------------------------------------------------------------------
// WriterRegistry
// ---------------------------------------------------------------------------

/// 写入器注册表。
///
/// `tool_id → Arc<dyn ConfigWriter>` 映射，O(1) 查找。
/// 同一个 Writer 实例可通过多个 tool_id 共享（如 codex-cli + codex-desktop）。
pub struct WriterRegistry {
    writers: HashMap<String, Arc<dyn ConfigWriter>>,
}

impl WriterRegistry {
    pub fn new() -> Self {
        Self { writers: HashMap::new() }
    }

    /// 将一个 Writer 注册到多个 tool_id。
    pub fn register(&mut self, tool_ids: &[&str], writer: Arc<dyn ConfigWriter>) {
        for id in tool_ids {
            self.writers.insert(id.to_string(), Arc::clone(&writer));
        }
    }

    /// 根据 tool_id 查找写入器。
    pub fn get(&self, tool_id: &str) -> Option<Arc<dyn ConfigWriter>> {
        self.writers.get(tool_id).cloned()
    }

    /// 一站式调用：查找 + 写入，找不到时自动返回错误。
    pub fn write_config(&self, tool_id: &str, ctx: &WriteContext) -> Result<(), String> {
        self.writers
            .get(tool_id)
            .ok_or_else(|| format!("暂不支持的工具: {}", tool_id))?
            .write_config(ctx)
    }

    /// 返回所有已注册的 tool_id。
    pub fn supported_tool_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.writers.keys().cloned().collect();
        ids.sort();
        ids
    }
}

impl Default for WriterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// 全局注册表单例
// ---------------------------------------------------------------------------

fn create_default_registry() -> WriterRegistry {
    let mut reg = WriterRegistry::new();

    reg.register(&["minimax-code-cli", "minimax-code-desktop"], Arc::new(MiniMaxWriter));
    reg.register(&["claude-code-cli"], Arc::new(ClaudeCliWriter));
    reg.register(&["claude-desktop"], Arc::new(ClaudeDesktopWriter));
    reg.register(&["codex-cli"], Arc::new(CodexWriter));
    reg.register(&["codex-desktop"], Arc::new(CodexDesktopWriter));
    reg.register(&["opencode-cli"], Arc::new(OpenCodeWriter));
    reg.register(&["qwen-code-cli"], Arc::new(QwenWriter));
    reg.register(&["aider-cli"], Arc::new(AiderWriter));
    reg.register(&["grok-build"], Arc::new(GrokWriter));
    reg.register(&["coffee-cli"], Arc::new(CoffeeWriter));
    reg.register(&["cursor-desktop"], Arc::new(CursorWriter));
    reg.register(&["gemini-desktop"], Arc::new(GeminiDesktopWriter));
    reg.register(&["windsurf-desktop"], Arc::new(WindsurfWriter));
    reg.register(&["trae-desktop"], Arc::new(TraeWriter));
    reg.register(&["zed-desktop"], Arc::new(ZedWriter));
    reg.register(&["kimi-cli"], Arc::new(KimiWriter));
    reg.register(&["openclaw"], Arc::new(OpenClawWriter));
    reg.register(&["hermes-agent"], Arc::new(HermesWriter));
    reg.register(&["nanobot"], Arc::new(NanoBotWriter));

    reg
}

static REGISTRY: OnceLock<WriterRegistry> = OnceLock::new();

/// 获取全局写入器注册表。
/// 首次调用时初始化，后续零开销。
pub fn get_registry() -> &'static WriterRegistry {
    REGISTRY.get_or_init(create_default_registry)
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_tools_registered() {
        let reg = get_registry();
        let ids = reg.supported_tool_ids();
        assert!(ids.contains(&"minimax-code-cli".to_string()));
        assert!(ids.contains(&"claude-code-cli".to_string()));
        assert!(ids.contains(&"codex-cli".to_string()));
        assert!(ids.contains(&"aider-cli".to_string()));
        assert!(ids.contains(&"codex-desktop".to_string()));
        assert!(ids.contains(&"claude-desktop".to_string()));
        assert!(ids.contains(&"kimi-cli".to_string()));
        assert!(ids.contains(&"qwen-code-cli".to_string()));
        assert!(ids.contains(&"opencode-cli".to_string()));
        assert!(ids.contains(&"grok-build".to_string()));
        assert!(ids.contains(&"openclaw".to_string()));
        assert!(ids.contains(&"hermes-agent".to_string()));
        assert!(ids.contains(&"nanobot".to_string()));
        assert!(ids.contains(&"minimax-code-desktop".to_string()));
        assert!(ids.contains(&"coffee-cli".to_string()));
        assert!(ids.contains(&"cursor-desktop".to_string()));
        assert!(ids.contains(&"gemini-desktop".to_string()));
        assert!(ids.contains(&"windsurf-desktop".to_string()));
        assert!(ids.contains(&"trae-desktop".to_string()));
        assert!(ids.contains(&"zed-desktop".to_string()));
        assert_eq!(ids.len(), 20);
    }

    #[test]
    fn test_unknown_tool_returns_error() {
        let reg = get_registry();
        let ctx = WriteContext {
            provider_id: "test",
            provider_name: "Test",
            base_url: "https://test.com",
            model_name: "test-model",
            api_key: "sk-test",
            anthropic_mode: false,
        };
        let result = reg.write_config("unknown-tool", &ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("暂不支持"));
    }

    #[test]
    fn test_claude_writers_resolve_separately() {
        // claude-code-cli 和 claude-desktop 是不同 struct，但都能被查找
        let reg = get_registry();
        assert!(reg.get("claude-code-cli").is_some());
        assert!(reg.get("claude-desktop").is_some());
    }

    #[test]
    fn test_minimax_cli_and_desktop_share_writer() {
        let reg = get_registry();
        let cli = reg.get("minimax-code-cli");
        let desktop = reg.get("minimax-code-desktop");
        assert!(cli.is_some());
        assert!(desktop.is_some());
    }

    #[test]
    fn test_get_nonexistent_returns_none() {
        let reg = get_registry();
        assert!(reg.get("does-not-exist").is_none());
    }

    #[test]
    fn test_new_registry_starts_empty() {
        let reg = WriterRegistry::new();
        assert!(reg.supported_tool_ids().is_empty());
    }

    #[test]
    fn test_register_and_lookup_roundtrip() {
        struct DummyWriter;
        impl ConfigWriter for DummyWriter {
            fn write_config(&self, _ctx: &WriteContext) -> Result<(), String> {
                Ok(())
            }
        }

        let mut reg = WriterRegistry::new();
        reg.register(&["dummy-a", "dummy-b"], Arc::new(DummyWriter));
        let ids = reg.supported_tool_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"dummy-a".to_string()));
        assert!(ids.contains(&"dummy-b".to_string()));
    }
}
