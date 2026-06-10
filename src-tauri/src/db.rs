use crate::keyring_store;
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ===== 数据模型 =====

/// 工具平台（Claude Code, MiniMax Code, Codex CLI 等）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub category: String,        // 'cli', 'desktop', 'agent'
    pub config_path: Option<String>,
    pub config_format: String,   // 'json', 'yaml', 'json5', 'env'
    pub launch_command: Option<String>,
    pub launch_path: Option<String>,
    pub env_keys_json: Option<String>,
    pub detection_path_cmds: String, // JSON: ["claude", "minimax", ...]
    pub detection_files: String,     // JSON: 桌面端检测路径
    pub created_at: i64,
    pub updated_at: i64,
}

/// 厂商/供应商（API Key 通过 keyring 存储；api_key 列仅用于旧数据迁移）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub api_base: String,
    pub anthropic_mode: bool,
    pub has_api_key: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 模型
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Model {
    pub id: String,
    pub provider_id: String,
    pub name: String,
    pub model_id: String,
    pub context_length: i64,
    pub max_output: i64,
    pub supports_attachment: bool,
    pub supports_reasoning: bool,
    pub supports_tool_call: bool,
    pub supports_vision: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 工具-厂商绑定
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolBinding {
    pub id: String,
    pub tool_id: String,
    pub provider_id: String,
    pub model_id: String,
    pub keyring_key: Option<String>,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 旧版 VendorInstance（用于迁移）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VendorInstance {
    pub id: String,
    pub preset_id: Option<String>,
    pub name: String,
    pub api_base: String,
    pub model: String,
    pub keyring_key: String,
    pub created_at: i64,
    pub updated_at: i64,
}

// ===== 数据库初始化与迁移 =====

pub fn init_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    conn.execute_batch("ALTER TABLE providers ADD COLUMN api_key TEXT;").ok();

    conn.execute_batch(
        r#"
        -- 工具平台表
        CREATE TABLE IF NOT EXISTS tools (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL DEFAULT 'cli',
            config_path TEXT,
            config_format TEXT NOT NULL DEFAULT 'json',
            launch_command TEXT,
            launch_path TEXT,
            env_keys_json TEXT DEFAULT '[]',
            detection_path_cmds TEXT NOT NULL DEFAULT '[]',
            detection_files TEXT NOT NULL DEFAULT '[]',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        -- 厂商表
        CREATE TABLE IF NOT EXISTS providers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            api_base TEXT NOT NULL,
            anthropic_mode INTEGER NOT NULL DEFAULT 1,
            api_key TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        -- 模型表
        CREATE TABLE IF NOT EXISTS models (
            id TEXT PRIMARY KEY,
            provider_id TEXT NOT NULL,
            name TEXT NOT NULL,
            model_id TEXT NOT NULL,
            context_length INTEGER NOT NULL DEFAULT 128000,
            max_output INTEGER NOT NULL DEFAULT 8192,
            supports_attachment INTEGER NOT NULL DEFAULT 0,
            supports_reasoning INTEGER NOT NULL DEFAULT 1,
            supports_tool_call INTEGER NOT NULL DEFAULT 1,
            supports_vision INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (provider_id) REFERENCES providers(id) ON DELETE CASCADE
        );

        -- 工具-厂商绑定表
        CREATE TABLE IF NOT EXISTS tool_bindings (
            id TEXT PRIMARY KEY,
            tool_id TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            model_id TEXT NOT NULL,
            keyring_key TEXT,
            is_active INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (tool_id) REFERENCES tools(id) ON DELETE CASCADE,
            FOREIGN KEY (provider_id) REFERENCES providers(id) ON DELETE CASCADE,
            FOREIGN KEY (model_id) REFERENCES models(id) ON DELETE CASCADE
        );

        -- 保留旧 settings 表做迁移
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        -- 保留旧的 vendors 表做迁移
        CREATE TABLE IF NOT EXISTS vendors (
            id TEXT PRIMARY KEY,
            preset_id TEXT,
            name TEXT NOT NULL,
            api_base TEXT NOT NULL,
            model TEXT NOT NULL,
            keyring_key TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        "#,
    )?;

    // 将所有迁移和种子操作包裹在事务中，保证原子性
    conn.execute_batch("BEGIN")?;
    let migration_result = (|| -> Result<()> {
        migrate_from_v1(&conn)?;
        migrate_legacy_provider_keys(&conn)?;
        seed_default_tools(&conn)?;
        normalize_tool_configs(&conn)?;
        seed_default_providers(&conn)?;
        Ok(())
    })();
    if migration_result.is_ok() {
        conn.execute_batch("COMMIT")?;
    } else {
        conn.execute_batch("ROLLBACK")?;
        migration_result?; // 传播错误
    }

    Ok(conn)
}

// ===== 数据初始化 =====

fn seed_default_tools(conn: &Connection) -> Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let defaults = vec![
        ("claude-code-cli", "Claude Code CLI", "cli",
         Some("~/.claude/settings.json"), "json",
         Some("claude"), None::<&str>,
         r#"["claude"]"#, r#"[]"#),
        ("claude-desktop", "Claude 桌面端", "desktop",
         Some("~/.claude/settings.json"), "json",
         None, None,
         r#"[]"#, r#"["Claude.exe"]"#),
        ("minimax-code-cli", "MiniMax Code CLI", "cli",
         Some("~/.minimax/config.yaml"), "yaml",
         Some("minimax"), None,
         r#"["minimax"]"#, r#"[]"#),
        ("minimax-code-desktop", "MiniMax Code 桌面版", "desktop",
         Some("~/.minimax/config.yaml"), "yaml",
         None,
         None,
         r#"[]"#, r#"["MiniMax Code.exe"]"#),
        ("codex-cli", "Codex CLI", "cli",
         Some("~/.codex/config.toml"), "toml",
         Some("codex"), None,
         r#"["codex"]"#, r#"[]"#),
        ("codex-desktop", "Codex 桌面端", "desktop",
         None, "toml",
         None, None,
         r#"[]"#, r#"["Codex.exe"]"#),
        ("gemini-desktop", "Gemini 桌面端", "desktop",
         Some("~/.gemini/settings.json"), "json",
         None, None,
         r#"[]"#, r#"["Gemini.exe"]"#),
        // ===== AI IDE 工具 =====
        ("cursor-desktop", "Cursor", "desktop",
         None, "json",
         None, None,
         r#"[]"#, r#"["Cursor.exe"]"#),
        ("windsurf-desktop", "Windsurf", "desktop",
         None, "json",
         None, None,
         r#"[]"#, r#"["Windsurf.exe"]"#),
        ("trae-desktop", "Trae", "desktop",
         None, "json",
         None, None,
         r#"[]"#, r#"["Trae.exe"]"#),
        ("zed-desktop", "Zed", "desktop",
         None, "json",
         None, None,
         r#"[]"#, r#"["Zed.exe"]"#),
        ("qwen-code-cli", "Qwen Code CLI", "cli",
         Some("~/.qwen/settings.json"), "json",
         Some("qwen"), None,
         r#"["qwen"]"#, r#"[]"#),
        ("aider-cli", "Aider CLI", "cli",
         Some("~/.aider.conf.yml"), "yaml",
         Some("aider"), None,
         r#"["aider"]"#, r#"[]"#),
        ("coffee-cli", "Coffee CLI", "cli",
         Some("~/.coffee-cli/config.json"), "json",
         Some("coffee-cli"), None,
         r#"["coffee-cli"]"#, r#"[]"#),
        ("opencode-cli", "OpenCode CLI", "cli",
         Some("~/.opencode/config.json"), "json",
         Some("opencode"), None,
         r#"["opencode"]"#, r#"[]"#),
        ("kimi-cli", "Kimi CLI", "cli",
         Some("~/.kimi/config.toml"), "toml",
         Some("kimi"), None,
         r#"["kimi"]"#, r#"[]"#),
        ("grok-build", "Grok Build", "cli",
         Some("~/.grok/config.toml"), "toml",
         Some("grok"), None,
         r#"["grok"]"#, r#"[]"#),
        ("openclaw", "OpenClaw", "agent",
         Some("~/.openclaw/openclaw.json"), "json5",
         Some("openclaw"), None,
         r#"["openclaw"]"#, r#"[]"#),
        ("hermes-agent", "Hermes Agent", "agent",
         Some("~/.hermes/config.yaml"), "yaml",
         Some("hermes"), None,
         r#"["hermes"]"#, r#"[]"#),
        ("nanobot", "Nanobot", "agent",
         Some("~/.nanobot/config.json"), "json",
         Some("nanobot"), None,
         r#"["nanobot"]"#, r#"[]"#),
    ];

    for (id, name, cat, config_path, config_fmt, launch_cmd, launch_path, path_cmds, files) in defaults {
        conn.execute(
            "INSERT OR IGNORE INTO tools (id, name, category, config_path, config_format,
             launch_command, launch_path, detection_path_cmds, detection_files, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![id, name, cat, config_path, config_fmt,
                launch_cmd, launch_path, path_cmds, files, now, now],
        )?;
    }
    Ok(())
}

fn seed_default_providers(conn: &Connection) -> Result<()> {
    // 检查是否已有数据
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM providers", [], |row| row.get(0))
        .unwrap_or(0);
    if count > 0 {
        return Ok(());
    }

    let now = now_ts();

    // ==================== 插入厂商 ====================
    let providers = [
        ("deepseek",  "DeepSeek",          "https://api.deepseek.com/v1",                          true),
        ("zhipu",     "智谱AI / GLM",      "https://open.bigmodel.cn/api/paas/v4",                  false),
        ("qwen",      "通义千问 / Qwen",    "https://dashscope.aliyuncs.com/compatible-mode/v1",     false),
        ("moonshot",  "月之暗面 / Kimi",    "https://api.moonshot.cn/v1",                           false),
        ("minimax",   "MiniMax",            "https://api.minimax.chat/v1",                          true),
        ("anthropic", "Anthropic",          "https://api.anthropic.com/v1",                         true),
    ];

    for &(id, name, api_base, anthropic_mode) in &providers {
        conn.execute(
            "INSERT OR IGNORE INTO providers (id, name, api_base, anthropic_mode, api_key, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6)",
            rusqlite::params![id, name, api_base, anthropic_mode as i32, now, now],
        )?;
    }

    // ==================== 插入模型 ====================
    // (provider_id, model_id, name, context_length, max_output, supports_attachment, supports_reasoning, supports_tool_call, supports_vision)
    let models: Vec<(&str, &str, &str, i64, i64, i32, i32, i32, i32)> = vec![
        // ---- DeepSeek ----
        ("deepseek", "deepseek-chat",     "DeepSeek Chat",     128_000, 8192, 0, 0, 1, 0),
        ("deepseek", "deepseek-reasoner", "DeepSeek Reasoner", 128_000, 8192, 0, 1, 0, 0),
        // ---- 智谱AI / GLM ----
        ("zhipu", "glm-4-plus",   "GLM-4 Plus",   128_000, 8192, 0, 0, 1, 0),
        ("zhipu", "glm-4v-plus",  "GLM-4V Plus",    8_000, 8192, 0, 0, 0, 1),
        ("zhipu", "glm-5v-turbo", "GLM-5V Turbo",   8_000, 8192, 0, 0, 1, 1),
        // ---- 通义千问 / Qwen ----
        ("qwen", "qwen-plus",  "Qwen Plus",  131_000, 8192, 0, 0, 1, 0),
        ("qwen", "qwen-turbo", "Qwen Turbo", 1_000_000, 8192, 0, 0, 1, 0),
        ("qwen", "qwen-max",   "Qwen Max",     32_000, 8192, 0, 1, 1, 0),
        // ---- 月之暗面 / Kimi ----
        ("moonshot", "moonshot-v1-128k", "Moonshot V1 128K", 128_000, 8192, 0, 0, 0, 0),
        ("moonshot", "moonshot-v1-32k",  "Moonshot V1 32K",   32_000, 8192, 0, 0, 0, 0),
        // ---- MiniMax ----
        ("minimax", "MiniMax-M3",  "MiniMax M3",  128_000, 8192, 0, 0, 1, 0),
        ("minimax", "MiniMax-M2.7","MiniMax M2.7",128_000, 8192, 0, 0, 1, 0),
        // ---- Anthropic ----
        ("anthropic", "claude-sonnet-4-6", "Claude Sonnet 4.6", 200_000, 8192, 0, 1, 1, 0),
        ("anthropic", "claude-haiku-4-5",  "Claude Haiku 4.5",  200_000, 8192, 0, 0, 1, 0),
    ];

    for &(provider_id, model_id, name, context_length, max_output, attachment, reasoning, tool_call, vision) in &models {
        let id = format!("{}/{}", provider_id, model_id);
        conn.execute(
            "INSERT OR IGNORE INTO models (id, provider_id, name, model_id, context_length, max_output,
             supports_attachment, supports_reasoning, supports_tool_call, supports_vision, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![id, provider_id, name, model_id, context_length, max_output,
                attachment, reasoning, tool_call, vision, now, now],
        )?;
    }

    Ok(())
}

fn normalize_tool_configs(conn: &Connection) -> Result<()> {
    conn.execute(
        "UPDATE tools SET config_path = ?1, config_format = 'toml' WHERE id = 'codex-cli'",
        ["~/.codex/config.toml"],
    )?;
    conn.execute(
        "UPDATE tools SET config_path = ?1, config_format = 'json' WHERE id = 'opencode-cli'",
        ["~/.opencode/config.json"],
    )?;
    conn.execute(
        "UPDATE tools SET config_path = ?1, config_format = 'json' WHERE id = 'qwen-code-cli'",
        ["~/.qwen/settings.json"],
    )?;
    conn.execute(
        "UPDATE tools SET config_path = ?1, config_format = 'yaml' WHERE id = 'aider-cli'",
        ["~/.aider.conf.yml"],
    )?;
    conn.execute(
        "UPDATE tools SET config_path = ?1, config_format = 'toml' WHERE id = 'kimi-cli'",
        ["~/.kimi/config.toml"],
    )?;
    Ok(())
}

fn migrate_from_v1(conn: &Connection) -> Result<()> {
    let has_old: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='vendors'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);
    if !has_old { return Ok(()); }

    let migrated: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM settings WHERE key='schema_migrated' AND value='v2'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);
    if migrated { return Ok(()); }

    let mut stmt = conn.prepare(
        "SELECT id, preset_id, name, api_base, model, keyring_key, created_at, updated_at FROM vendors",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(VendorInstance {
            id: row.get(0)?, preset_id: row.get(1)?, name: row.get(2)?,
            api_base: row.get(3)?, model: row.get(4)?, keyring_key: row.get(5)?,
            created_at: row.get(6)?, updated_at: row.get(7)?,
        })
    })?;

    for row in rows.flatten() {
        let pid = row.preset_id.clone().unwrap_or_else(|| row.name.to_lowercase().replace(' ', "-"));
        conn.execute(
            "INSERT OR IGNORE INTO providers (id, name, api_base, anthropic_mode, api_key, created_at, updated_at)
             VALUES (?1, ?2, ?3, 1, NULL, ?4, ?5)",
            rusqlite::params![pid, row.name, row.api_base, row.created_at, row.updated_at],
        )?;
        let mid = format!("{}/{}", pid, row.model);
        conn.execute(
            "INSERT OR IGNORE INTO models (id, provider_id, name, model_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![mid, pid, row.model, row.model, row.created_at, row.updated_at],
        )?;
    }

    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('schema_migrated', 'v2')",
        [],
    )?;
    Ok(())
}

fn migrate_legacy_provider_keys(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, api_key FROM providers WHERE api_key IS NOT NULL AND trim(api_key) <> ''",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    for row in rows {
        let (provider_id, api_key) = row?;
        let account = format!("provider:{}", provider_id);
        if keyring_store::set_key("MiniMax-vendor-switcher", &account, &api_key).is_ok() {
            conn.execute(
                "UPDATE providers SET api_key = NULL WHERE id = ?1",
                [&provider_id],
            )?;
        }
    }

    Ok(())
}

// ===== 工具 CRUD =====

pub fn list_tools(conn: &Connection) -> Result<Vec<Tool>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, category, config_path, config_format, launch_command, launch_path,
                env_keys_json, detection_path_cmds, detection_files, created_at, updated_at
         FROM tools ORDER BY category, name",
    )?;
    let rows = stmt.query_map([], map_tool)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn get_tool(conn: &Connection, id: &str) -> Result<Option<Tool>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, category, config_path, config_format, launch_command, launch_path,
                env_keys_json, detection_path_cmds, detection_files, created_at, updated_at
         FROM tools WHERE id = ?1",
    )?;
    let mut iter = stmt.query_map([id], map_tool)?;
    Ok(iter.next().transpose()?)
}

fn map_tool(row: &rusqlite::Row) -> rusqlite::Result<Tool> {
    Ok(Tool {
        id: row.get(0)?, name: row.get(1)?, category: row.get(2)?,
        config_path: row.get(3)?, config_format: row.get(4)?,
        launch_command: row.get(5)?, launch_path: row.get(6)?,
        env_keys_json: row.get(7)?, detection_path_cmds: row.get(8)?,
        detection_files: row.get(9)?, created_at: row.get(10)?, updated_at: row.get(11)?,
    })
}

// ===== 厂商 CRUD =====

pub fn list_providers(conn: &Connection) -> Result<Vec<Provider>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, api_base, anthropic_mode, created_at, updated_at
         FROM providers ORDER BY name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Provider {
            id: row.get(0)?, name: row.get(1)?, api_base: row.get(2)?,
            anthropic_mode: row.get::<_, i32>(3)? != 0,
            has_api_key: false,
            created_at: row.get(4)?, updated_at: row.get(5)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn insert_provider(conn: &Connection, p: &Provider) -> Result<()> {
    conn.execute(
        "INSERT INTO providers (id, name, api_base, anthropic_mode, api_key, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6)",
        rusqlite::params![p.id, p.name, p.api_base, p.anthropic_mode as i32, p.created_at, p.updated_at],
    )?;
    Ok(())
}

pub fn delete_provider(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM providers WHERE id = ?1", [id])?;
    Ok(())
}

pub fn get_provider_legacy_api_key(conn: &Connection, id: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT api_key FROM providers WHERE id = ?1")?;
    let mut iter = stmt.query_map([id], |row| row.get::<_, Option<String>>(0))?;
    Ok(iter.next().transpose()?.flatten())
}

pub fn clear_provider_legacy_api_key(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("UPDATE providers SET api_key = NULL WHERE id = ?1", [id])?;
    Ok(())
}

pub fn update_provider(conn: &Connection, p: &Provider) -> Result<()> {
    conn.execute(
        "UPDATE providers SET name=?2, api_base=?3, anthropic_mode=?4, updated_at=?5 WHERE id=?1",
        rusqlite::params![p.id, p.name, p.api_base, p.anthropic_mode as i32, p.updated_at],
    )?;
    Ok(())
}

pub fn get_provider(conn: &Connection, id: &str) -> Result<Option<Provider>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, api_base, anthropic_mode, created_at, updated_at
         FROM providers WHERE id = ?1",
    )?;
    let mut iter = stmt.query_map([id], |row| {
        Ok(Provider {
            id: row.get(0)?, name: row.get(1)?, api_base: row.get(2)?,
            anthropic_mode: row.get::<_, i32>(3)? != 0,
            has_api_key: false,
            created_at: row.get(4)?, updated_at: row.get(5)?,
        })
    })?;
    Ok(iter.next().transpose()?)
}

// ===== 模型 CRUD =====

pub fn list_models(conn: &Connection) -> Result<Vec<Model>> {
    let mut stmt = conn.prepare(
        "SELECT id, provider_id, name, model_id, context_length, max_output,
                supports_attachment, supports_reasoning, supports_tool_call, supports_vision,
                created_at, updated_at
         FROM models ORDER BY provider_id, name",
    )?;
    let rows = stmt.query_map([], map_model)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn list_models_by_provider(conn: &Connection, provider_id: &str) -> Result<Vec<Model>> {
    let mut stmt = conn.prepare(
        "SELECT id, provider_id, name, model_id, context_length, max_output,
                supports_attachment, supports_reasoning, supports_tool_call, supports_vision,
                created_at, updated_at
         FROM models m WHERE m.provider_id = ?1 ORDER BY name",
    )?;
    let rows = stmt.query_map([provider_id], map_model)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn insert_model(conn: &Connection, m: &Model) -> Result<()> {
    conn.execute(
        "INSERT INTO models (id, provider_id, name, model_id, context_length, max_output,
         supports_attachment, supports_reasoning, supports_tool_call, supports_vision,
         created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![
            m.id, m.provider_id, m.name, m.model_id, m.context_length, m.max_output,
            m.supports_attachment as i32, m.supports_reasoning as i32,
            m.supports_tool_call as i32, m.supports_vision as i32,
            m.created_at, m.updated_at
        ],
    )?;
    Ok(())
}

pub fn update_model(conn: &Connection, m: &Model) -> Result<()> {
    conn.execute(
        "UPDATE models SET provider_id=?2, name=?3, model_id=?4, context_length=?5, max_output=?6,
         supports_attachment=?7, supports_reasoning=?8, supports_tool_call=?9, supports_vision=?10,
         updated_at=?11 WHERE id=?1",
        rusqlite::params![
            m.id, m.provider_id, m.name, m.model_id, m.context_length, m.max_output,
            m.supports_attachment as i32, m.supports_reasoning as i32,
            m.supports_tool_call as i32, m.supports_vision as i32, m.updated_at
        ],
    )?;
    Ok(())
}

pub fn delete_model(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM models WHERE id = ?1", [id])?;
    Ok(())
}

fn map_model(row: &rusqlite::Row) -> rusqlite::Result<Model> {
    Ok(Model {
        id: row.get(0)?, provider_id: row.get(1)?, name: row.get(2)?, model_id: row.get(3)?,
        context_length: row.get(4)?, max_output: row.get(5)?,
        supports_attachment: row.get::<_, i32>(6)? != 0,
        supports_reasoning: row.get::<_, i32>(7)? != 0,
        supports_tool_call: row.get::<_, i32>(8)? != 0,
        supports_vision: row.get::<_, i32>(9)? != 0,
        created_at: row.get(10)?, updated_at: row.get(11)?,
    })
}

// ===== 绑定 CRUD =====

pub fn list_bindings(conn: &Connection) -> Result<Vec<ToolBinding>> {
    let mut stmt = conn.prepare(
        "SELECT id, tool_id, provider_id, model_id, keyring_key, is_active, created_at, updated_at
         FROM tool_bindings ORDER BY tool_id",
    )?;
    let rows = stmt.query_map([], map_binding)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn get_active_binding(conn: &Connection, tool_id: &str) -> Result<Option<ToolBinding>> {
    let mut stmt = conn.prepare(
        "SELECT id, tool_id, provider_id, model_id, keyring_key, is_active, created_at, updated_at
         FROM tool_bindings WHERE tool_id = ?1 AND is_active = 1 LIMIT 1",
    )?;
    let mut iter = stmt.query_map([tool_id], map_binding)?;
    Ok(iter.next().transpose()?)
}

pub fn set_active_binding(conn: &Connection, binding_id: &str, tool_id: &str) -> Result<()> {
    let now = now_ts();
    conn.execute(
        "UPDATE tool_bindings SET is_active = 0, updated_at = ?1 WHERE tool_id = ?2",
        rusqlite::params![now, tool_id],
    )?;
    conn.execute(
        "UPDATE tool_bindings SET is_active = 1, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, binding_id],
    )?;
    Ok(())
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn upsert_binding(conn: &Connection, b: &ToolBinding) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO tool_bindings
         (id, tool_id, provider_id, model_id, keyring_key, is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            b.id, b.tool_id, b.provider_id, b.model_id, b.keyring_key,
            b.is_active as i32, b.created_at, b.updated_at
        ],
    )?;
    Ok(())
}

pub fn delete_binding(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM tool_bindings WHERE id = ?1", [id])?;
    Ok(())
}

pub fn list_bindings_by_tool(conn: &Connection, tool_id: &str) -> Result<Vec<ToolBinding>> {
    let mut stmt = conn.prepare(
        "SELECT id, tool_id, provider_id, model_id, keyring_key, is_active, created_at, updated_at
         FROM tool_bindings WHERE tool_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([tool_id], map_binding)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn list_bindings_by_provider(conn: &Connection, provider_id: &str) -> Result<Vec<ToolBinding>> {
    let mut stmt = conn.prepare(
        "SELECT id, tool_id, provider_id, model_id, keyring_key, is_active, created_at, updated_at
         FROM tool_bindings WHERE provider_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([provider_id], map_binding)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn delete_bindings_by_tool(conn: &Connection, tool_id: &str) -> Result<()> {
    conn.execute("DELETE FROM tool_bindings WHERE tool_id = ?1", [tool_id])?;
    Ok(())
}

pub fn delete_provider_cascade(conn: &mut Connection, provider_id: &str) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM tool_bindings WHERE provider_id = ?1", [provider_id])?;
    tx.execute("DELETE FROM models WHERE provider_id = ?1", [provider_id])?;
    tx.execute("DELETE FROM providers WHERE id = ?1", [provider_id])?;
    tx.commit()?;
    Ok(())
}

fn map_binding(row: &rusqlite::Row) -> rusqlite::Result<ToolBinding> {
    Ok(ToolBinding {
        id: row.get(0)?, tool_id: row.get(1)?, provider_id: row.get(2)?,
        model_id: row.get(3)?, keyring_key: row.get(4)?,
        is_active: row.get::<_, i32>(5)? != 0,
        created_at: row.get(6)?, updated_at: row.get(7)?,
    })
}

// ===== 旧版兼容 =====
// 保留给旧 commands 过渡用
pub fn list_vendors(conn: &Connection) -> Result<Vec<VendorInstance>> {
    let mut stmt = conn.prepare(
        "SELECT id, preset_id, name, api_base, model, keyring_key, created_at, updated_at
         FROM vendors ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(VendorInstance {
            id: row.get(0)?, preset_id: row.get(1)?, name: row.get(2)?,
            api_base: row.get(3)?, model: row.get(4)?, keyring_key: row.get(5)?,
            created_at: row.get(6)?, updated_at: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub fn get_vendor(conn: &Connection, id: &str) -> Result<Option<VendorInstance>> {
    let mut stmt = conn.prepare(
        "SELECT id, preset_id, name, api_base, model, keyring_key, created_at, updated_at
         FROM vendors WHERE id = ?1",
    )?;
    let mut iter = stmt.query_map([id], |row| {
        Ok(VendorInstance {
            id: row.get(0)?, preset_id: row.get(1)?, name: row.get(2)?,
            api_base: row.get(3)?, model: row.get(4)?, keyring_key: row.get(5)?,
            created_at: row.get(6)?, updated_at: row.get(7)?,
        })
    })?;
    Ok(iter.next().transpose()?)
}

pub fn insert_vendor(conn: &Connection, v: &VendorInstance) -> Result<()> {
    conn.execute(
        "INSERT INTO vendors (id, preset_id, name, api_base, model, keyring_key, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            v.id, v.preset_id, v.name, v.api_base, v.model, v.keyring_key, v.created_at, v.updated_at
        ],
    )?;
    Ok(())
}

pub fn update_vendor(conn: &Connection, v: &VendorInstance) -> Result<()> {
    conn.execute(
        "UPDATE vendors SET preset_id=?2, name=?3, api_base=?4, model=?5,
         keyring_key=?6, updated_at=?7 WHERE id=?1",
        rusqlite::params![
            v.id, v.preset_id, v.name, v.api_base, v.model, v.keyring_key, v.updated_at
        ],
    )?;
    Ok(())
}

pub fn delete_vendor(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM vendors WHERE id = ?1", [id])?;
    Ok(())
}
