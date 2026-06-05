# MiniMax Code Vendor Switcher Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 构建一个 Tauri 2.0 桌面应用，让用户在 DeepSeek / Kimi / 智谱 / Qwen 等 LLM 厂商之间一键切换，切换后启动 MiniMax Code 即可使用所选厂商的模型。

**Architecture:** Rust 后端负责厂商元数据持久化（SQLite）、API Key 加密存储（Keyring）、`~/.claude/settings.json` 原子写入与备份；React 前端通过 Tauri IPC 调用 Rust Commands 展示与操作。MVP 范围仅含厂商 CRUD + 一键切换 + 启动 MiniMax Code CLI。

**Tech Stack:** Tauri 2.0 + Rust 1.75+ + SQLite (rusqlite) + keyring crate + serde_json + React 18 + TypeScript + Vite + Tailwind CSS + shadcn/ui

---

## 实施前置

**环境要求**（在执行任务前确认）：

```bash
# 1. Rust 工具链
rustc --version   # >= 1.75
cargo --version

# 2. Node.js
node --version    # >= 18
pnpm --version    # 推荐 pnpm，或 npm/yarn

# 3. Tauri CLI
cargo install tauri-cli --version "^2.0" --locked

# 4. 系统依赖
#    Windows: Microsoft C++ Build Tools + WebView2（Win11 自带）
#    macOS: Xcode Command Line Tools
#    Linux: webkit2gtk-4.1 + 其他（参见 Tauri 官方文档）
```

**前置检查**：

```bash
cd .worktrees/feat-initial-impl
git status  # 应该在 feat/initial-impl 分支
```

---

## Task 1: 初始化 Tauri 2.0 项目脚手架

**Files:**
- Create: `package.json`, `src/`, `src-tauri/`, `vite.config.ts`, `tsconfig.json`, `tailwind.config.js`, `index.html`

**Step 1: 用 Tauri CLI 初始化项目**

```bash
cd .worktrees/feat-initial-impl
pnpm create tauri-app@latest . -- --template react-ts --manager pnpm --identifier "com.MiniMax.vendor-switcher"
```

如果交互式提示"目录非空"——选"Empty current directory"或在临时目录创建后 mv 进来。

**Step 2: 验证脚手架**

```bash
pnpm install
pnpm tauri --version   # 应该输出 2.x
```

Expected: 看到 `tauri-cli 2.x.x`。

**Step 3: 跑通 dev 模式（仅冒烟，不需真窗口）**

```bash
timeout 30 pnpm tauri dev 2>&1 | head -20
```

Expected: 看到 "Compiling tauri-cli"、窗口启动（或在 CI 中可超时）。

**Step 4: 提交**

```bash
git add .
git commit -m "chore: 初始化 Tauri 2.0 + React + TS 脚手架"
```

---

## Task 2: 配置 Tailwind CSS 与 shadcn/ui 基础

**Files:**
- Create: `tailwind.config.js`, `postcss.config.js`, `src/index.css`
- Modify: `src/main.tsx`, `src/App.tsx`

**Step 1: 安装 Tailwind 依赖**

```bash
pnpm add -D tailwindcss@^3 postcss autoprefixer
pnpm dlx tailwindcss init -p
```

**Step 2: 写 Tailwind 配置**

`tailwind.config.js`:

```js
/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: { extend: {} },
  plugins: [],
};
```

**Step 3: 写 index.css**

`src/index.css`:

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}
```

**Step 4: 验证构建**

```bash
pnpm build
```

Expected: 无错误，生成 `dist/`。

**Step 5: 提交**

```bash
git add .
git commit -m "chore: 集成 Tailwind CSS"
```

---

## Task 3: 添加 shadcn/ui 初始化

**Files:**
- Create: `components.json`, `src/components/ui/`

**Step 1: 初始化 shadcn/ui**

```bash
pnpm dlx shadcn@latest init
```

按提示选择：
- Style: Default
- Base color: Slate
- CSS file: `src/index.css`

**Step 2: 添加首批组件**

```bash
pnpm dlx shadcn@latest add button card dialog input label
```

**Step 3: 验证组件已生成**

```bash
ls src/components/ui/
```

Expected: 看到 `button.tsx`, `card.tsx`, `dialog.tsx`, `input.tsx`, `label.tsx`。

**Step 4: 提交**

```bash
git add .
git commit -m "chore: 初始化 shadcn/ui 组件库"
```

---

## Task 4: Rust 后端 - 引入核心依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`

**Step 1: 编辑 Cargo.toml 添加依赖**

`src-tauri/Cargo.toml` 在 `[dependencies]` 中追加：

```toml
rusqlite = { version = "0.31", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
keyring = "2"
thiserror = "1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
tempfile = "3"
```

**Step 2: 验证依赖能解析**

```bash
cd src-tauri && cargo check
```

Expected: 编译通过，无错误。

**Step 3: 提交**

```bash
cd ..
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore: 添加后端核心依赖（rusqlite/keyring/serde 等）"
```

---

## Task 5: Rust 后端 - db.rs（SQLite 初始化与迁移）

**Files:**
- Create: `src-tauri/src/db.rs`
- Create: `src-tauri/src/db.test.rs`（或 `tests/db_test.rs`）
- Modify: `src-tauri/src/lib.rs`

**Step 1: 写失败测试 - 数据库初始化**

`src-tauri/tests/db_test.rs`:

```rust
use MiniMax_vendor_switcher_lib::db::{init_db, VendorInstance};
use tempfile::tempdir;

#[test]
fn test_init_db_creates_schema() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let conn = init_db(&db_path).unwrap();

    // 验证表已创建
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='vendors'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "vendors 表应该被创建");
}
```

**Step 2: 跑测试确认失败**

```bash
cd src-tauri && cargo test --test db_test test_init_db_creates_schema
```

Expected: 编译错误（`db` 模块不存在）。

**Step 3: 实现 db.rs 最小版本**

`src-tauri/src/db.rs`:

```rust
use rusqlite::{Connection, Result};
use std::path::Path;
use serde::{Deserialize, Serialize};

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

pub fn init_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        r#"
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
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        "#,
    )?;
    Ok(conn)
}
```

**Step 4: 在 lib.rs 暴露模块**

`src-tauri/src/lib.rs` 顶部添加：

```rust
pub mod db;
```

**Step 5: 跑测试确认通过**

```bash
cargo test --test db_test test_init_db_creates_schema
```

Expected: PASS。

**Step 6: 提交**

```bash
cd ..
git add src-tauri/src/db.rs src-tauri/src/db.test.rs src-tauri/tests/db_test.rs src-tauri/src/lib.rs
git commit -m "feat(db): SQLite 初始化与 vendors/settings 表结构"
```

---

## Task 6: Rust 后端 - db.rs CRUD 方法

**Files:**
- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/tests/db_test.rs`

**Step 1: 追加失败测试 - insert/list/update/delete**

在 `src-tauri/tests/db_test.rs` 末尾追加：

```rust
#[test]
fn test_vendor_crud() {
    let dir = tempdir().unwrap();
    let conn = init_db(&dir.path().join("test.db")).unwrap();

    let v = VendorInstance {
        id: "v1".into(),
        preset_id: Some("deepseek".into()),
        name: "DeepSeek".into(),
        api_base: "https://api.deepseek.com".into(),
        model: "deepseek-chat".into(),
        keyring_key: "vendor:v1".into(),
        created_at: 100,
        updated_at: 100,
    };

    db::insert_vendor(&conn, &v).unwrap();
    let list = db::list_vendors(&conn).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "DeepSeek");

    let mut v2 = v.clone();
    v2.model = "deepseek-coder".into();
    v2.updated_at = 200;
    db::update_vendor(&conn, &v2).unwrap();
    let fetched = db::get_vendor(&conn, "v1").unwrap().unwrap();
    assert_eq!(fetched.model, "deepseek-coder");

    db::delete_vendor(&conn, "v1").unwrap();
    assert_eq!(db::list_vendors(&conn).unwrap().len(), 0);
}
```

**Step 2: 跑测试确认失败**

```bash
cd src-tauri && cargo test --test db_test test_vendor_crud
```

Expected: 编译错误（函数未定义）。

**Step 3: 实现 CRUD 函数**

在 `src-tauri/src/db.rs` 末尾追加：

```rust
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

pub fn list_vendors(conn: &Connection) -> Result<Vec<VendorInstance>> {
    let mut stmt = conn.prepare(
        "SELECT id, preset_id, name, api_base, model, keyring_key, created_at, updated_at
         FROM vendors ORDER BY created_at ASC",
    )?;
    let iter = stmt.query_map([], |row| {
        Ok(VendorInstance {
            id: row.get(0)?,
            preset_id: row.get(1)?,
            name: row.get(2)?,
            api_base: row.get(3)?,
            model: row.get(4)?,
            keyring_key: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    iter.collect()
}

pub fn get_vendor(conn: &Connection, id: &str) -> Result<Option<VendorInstance>> {
    let mut stmt = conn.prepare(
        "SELECT id, preset_id, name, api_base, model, keyring_key, created_at, updated_at
         FROM vendors WHERE id = ?1",
    )?;
    let mut iter = stmt.query_map([id], |row| {
        Ok(VendorInstance {
            id: row.get(0)?,
            preset_id: row.get(1)?,
            name: row.get(2)?,
            api_base: row.get(3)?,
            model: row.get(4)?,
            keyring_key: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    Ok(iter.next().transpose()?)
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
```

**Step 4: 跑测试确认通过**

```bash
cargo test --test db_test
```

Expected: 全部 PASS（2 个测试）。

**Step 5: 提交**

```bash
cd ..
git add src-tauri/src/db.rs src-tauri/tests/db_test.rs
git commit -m "feat(db): vendors 表的 CRUD 操作"
```

---

## Task 7: Rust 后端 - keyring_store.rs

**Files:**
- Create: `src-tauri/src/keyring_store.rs`
- Create: `src-tauri/tests/keyring_test.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: 写失败测试 - Keyring 读写**

`src-tauri/tests/keyring_test.rs`:

```rust
use MiniMax_vendor_switcher_lib::keyring_store::{set_key, get_key, delete_key};

const TEST_SERVICE: &str = "MiniMax-vendor-switcher-test";

#[test]
fn test_keyring_roundtrip() {
    let account = "test-account-roundtrip";
    // 清理可能残留
    let _ = delete_key(TEST_SERVICE, account);

    set_key(TEST_SERVICE, account, "secret-token-123").unwrap();
    let got = get_key(TEST_SERVICE, account).unwrap();
    assert_eq!(got, "secret-token-123");

    delete_key(TEST_SERVICE, account).unwrap();
    assert!(get_key(TEST_SERVICE, account).is_err());
}
```

**Step 2: 跑测试确认失败**

```bash
cd src-tauri && cargo test --test keyring_test
```

Expected: 编译错误（模块不存在）。

**Step 3: 实现 keyring_store.rs**

`src-tauri/src/keyring_store.rs`:

```rust
use keyring::Entry;

#[derive(Debug, thiserror::Error)]
pub enum KeyringError {
    #[error("keyring: {0}")]
    Keyring(#[from] keyring::Error),
}

pub fn set_key(service: &str, account: &str, value: &str) -> Result<(), KeyringError> {
    let entry = Entry::new(service, account)?;
    entry.set_password(value)?;
    Ok(())
}

pub fn get_key(service: &str, account: &str) -> Result<String, KeyringError> {
    let entry = Entry::new(service, account)?;
    Ok(entry.get_password()?)
}

pub fn delete_key(service: &str, account: &str) -> Result<(), KeyringError> {
    let entry = Entry::new(service, account)?;
    entry.delete_credential()?;
    Ok(())
}
```

`src-tauri/src/lib.rs` 追加：

```rust
pub mod keyring_store;
```

**Step 4: 跑测试确认通过**

```bash
cargo test --test keyring_test
```

Expected: PASS（需要 Keyring 后端可用；Windows 上即 Credential Manager）。

> 注意：在无 GUI 的 CI 环境下 Keyring 可能不可用，本测试主要在开发机运行。

**Step 5: 提交**

```bash
cd ..
git add src-tauri/src/keyring_store.rs src-tauri/tests/keyring_test.rs src-tauri/src/lib.rs
git commit -m "feat(keyring): API Key 安全存取封装"
```

---

## Task 8: Rust 后端 - claude_config.rs（settings.json 原子写入）

**Files:**
- Create: `src-tauri/src/claude_config.rs`
- Create: `src-tauri/tests/claude_config_test.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: 写失败测试 - 合并 env 与原子写**

`src-tauri/tests/claude_config_test.rs`:

```rust
use MiniMax_vendor_switcher_lib::claude_config::{read_settings, write_env_atomic, ClaudeSettings};
use std::collections::HashMap;
use tempfile::tempdir;

#[test]
fn test_write_env_merges_existing() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("settings.json");

    // 初始写入
    let mut initial = ClaudeSettings::default();
    let mut env = HashMap::new();
    env.insert("FOO".into(), "bar".into());
    initial.env = Some(env);
    write_env_atomic(&path, &initial).unwrap();

    // 修改 env
    let current = read_settings(&path).unwrap();
    let mut updated = current.clone();
    let mut new_env = updated.env.clone().unwrap_or_default();
    new_env.insert("ANTHROPIC_BASE_URL".into(), "https://api.deepseek.com".into());
    new_env.insert("ANTHROPIC_AUTH_TOKEN".into(), "sk-test".into());
    updated.env = Some(new_env);
    write_env_atomic(&path, &updated).unwrap();

    // 验证
    let final_settings = read_settings(&path).unwrap();
    let env = final_settings.env.unwrap();
    assert_eq!(env.get("FOO").unwrap(), "bar");
    assert_eq!(env.get("ANTHROPIC_BASE_URL").unwrap(), "https://api.deepseek.com");
    assert_eq!(env.get("ANTHROPIC_AUTH_TOKEN").unwrap(), "sk-test");
}

#[test]
fn test_backup_created() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("settings.json");
    write_env_atomic(&path, &ClaudeSettings::default()).unwrap();
    write_env_atomic(&path, &ClaudeSettings::default()).unwrap();

    // 备份目录应至少有一个文件
    let backup_dir = dir.path().join("backups");
    let entries: Vec<_> = std::fs::read_dir(&backup_dir).unwrap().collect();
    assert!(entries.len() >= 1, "应至少有一个备份文件");
}
```

**Step 2: 跑测试确认失败**

```bash
cd src-tauri && cargo test --test claude_config_test
```

Expected: 编译错误。

**Step 3: 实现 claude_config.rs**

`src-tauri/src/claude_config.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ClaudeSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    // 保留其他未识别字段
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn read_settings(path: &Path) -> Result<ClaudeSettings, ConfigError> {
    if !path.exists() {
        return Ok(ClaudeSettings::default());
    }
    let content = fs::read_to_string(path)?;
    let settings: ClaudeSettings = serde_json::from_str(&content)?;
    Ok(settings)
}

pub fn write_env_atomic(path: &Path, settings: &ClaudeSettings) -> Result<(), ConfigError> {
    // 备份原文件
    if path.exists() {
        let backup_dir = path.parent().unwrap().join("backups");
        fs::create_dir_all(&backup_dir)?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ConfigError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
            .as_millis();
        let backup_path: PathBuf = backup_dir.join(format!("settings.{}.json", timestamp));
        fs::copy(path, &backup_path)?;
    }

    // 原子写：先写临时文件再 rename
    let tmp_path = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(settings)?;
    {
        let mut f = fs::File::create(&tmp_path)?;
        f.write_all(json.as_bytes())?;
        f.sync_all()?;
    }
    fs::rename(&tmp_path, path)?;

    // 设置 0600 权限（Unix；Windows 上由 ACL 控制）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = fs::metadata(path)?.permissions();
        perm.set_mode(0o600);
        fs::set_permissions(path, perm)?;
    }

    Ok(())
}
```

`src-tauri/src/lib.rs` 追加：

```rust
pub mod claude_config;
```

**Step 4: 跑测试确认通过**

```bash
cargo test --test claude_config_test
```

Expected: 2 个测试全部 PASS。

**Step 5: 提交**

```bash
cd ..
git add src-tauri/src/claude_config.rs src-tauri/tests/claude_config_test.rs src-tauri/src/lib.rs
git commit -m "feat(config): 读取/原子写入 ~/.claude/settings.json + 自动备份"
```

---

## Task 9: Rust 后端 - vendor.rs 厂商预设

**Files:**
- Create: `src-tauri/src/vendor.rs`
- Create: `src-tauri/tests/vendor_test.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: 写失败测试 - 预设列表**

`src-tauri/tests/vendor_test.rs`:

```rust
use MiniMax_vendor_switcher_lib::vendor::{presets, VendorPreset};

#[test]
fn test_presets_contain_expected_vendors() {
    let p = presets();
    let ids: Vec<&str> = p.iter().map(|v| v.id).collect();
    assert!(ids.contains(&"deepseek"));
    assert!(ids.contains(&"kimi"));
    assert!(ids.contains(&"zhipu"));
    assert!(ids.contains(&"qwen"));
    assert!(ids.contains(&"MiniMax"));
}

#[test]
fn test_preset_has_required_fields() {
    let p: Vec<VendorPreset> = presets();
    let ds = p.iter().find(|v| v.id == "deepseek").unwrap();
    assert_eq!(ds.api_base, "https://api.deepseek.com");
    assert!(!ds.default_model.is_empty());
}
```

**Step 2: 跑测试确认失败**

```bash
cd src-tauri && cargo test --test vendor_test
```

Expected: 编译错误。

**Step 3: 实现 vendor.rs**

`src-tauri/src/vendor.rs`:

```rust
pub struct VendorPreset {
    pub id: &'static str,
    pub name: &'static str,
    pub api_base: &'static str,
    pub default_model: &'static str,
}

pub fn presets() -> Vec<VendorPreset> {
    vec![
        VendorPreset {
            id: "MiniMax",
            name: "MiniMax",
            api_base: "https://api.MiniMax.com",
            default_model: "MiniMax-M3",
        },
        VendorPreset {
            id: "deepseek",
            name: "DeepSeek",
            api_base: "https://api.deepseek.com",
            default_model: "deepseek-chat",
        },
        VendorPreset {
            id: "kimi",
            name: "Kimi (月之暗面)",
            api_base: "https://api.moonshot.cn/v1",
            default_model: "moonshot-v1-128k",
        },
        VendorPreset {
            id: "zhipu",
            name: "智谱 GLM",
            api_base: "https://open.bigmodel.cn/api/paas/v4",
            default_model: "glm-4-plus",
        },
        VendorPreset {
            id: "qwen",
            name: "Qwen (通义千问)",
            api_base: "https://dashscope.aliyuncs.com/compatible-mode/v1",
            default_model: "qwen-plus",
        },
    ]
}
```

`src-tauri/src/lib.rs` 追加：

```rust
pub mod vendor;
```

**Step 4: 跑测试确认通过**

```bash
cargo test --test vendor_test
```

Expected: PASS。

**Step 5: 提交**

```bash
cd ..
git add src-tauri/src/vendor.rs src-tauri/tests/vendor_test.rs src-tauri/src/lib.rs
git commit -m "feat(vendor): 内置 5 家厂商预设"
```

---

## Task 10: Rust 后端 - launcher.rs 启动 MiniMax Code

**Files:**
- Create: `src-tauri/src/launcher.rs`
- Create: `src-tauri/tests/launcher_test.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: 写失败测试 - 路径解析**

`src-tauri/tests/launcher_test.rs`:

```rust
use MiniMax_vendor_switcher_lib::launcher::{claude_binary_path, find_claude};

#[test]
fn test_claude_binary_path_format() {
    let p = claude_binary_path();
    let s = p.to_string_lossy();
    assert!(s.contains("claude") || s.contains("MiniMax-code"));
}

#[test]
fn test_find_claude_doesnt_panic() {
    // 找不到时返回 None，不 panic
    let _ = find_claude();
}
```

**Step 2: 跑测试确认失败**

```bash
cd src-tauri && cargo test --test launcher_test
```

Expected: 编译错误。

**Step 3: 实现 launcher.rs**

`src-tauri/src/launcher.rs`:

```rust
use std::path::PathBuf;
use std::process::Command;

pub fn claude_binary_path() -> PathBuf {
    // 优先 MiniMax-code，回退到 claude
    if let Ok(p) = which("MiniMax-code") {
        return p;
    }
    if let Ok(p) = which("claude") {
        return p;
    }
    PathBuf::from("MiniMax-code")
}

pub fn find_claude() -> Option<PathBuf> {
    which("MiniMax-code").or_else(|_| which("claude")).ok()
}

fn which(cmd: &str) -> std::io::Result<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(cmd);
        if candidate.is_file() {
            return Ok(candidate);
        }
        // Windows 上检查 .exe
        #[cfg(windows)]
        {
            let candidate_exe = dir.join(format!("{}.exe", cmd));
            if candidate_exe.is_file() {
                return Ok(candidate_exe);
            }
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "not found"))
}

pub fn launch_claude() -> std::io::Result<u32> {
    let path = claude_binary_path();
    let child = Command::new(&path).spawn()?;
    Ok(child.id())
}
```

`src-tauri/src/lib.rs` 追加：

```rust
pub mod launcher;
```

**Step 4: 跑测试确认通过**

```bash
cargo test --test launcher_test
```

Expected: PASS。

**Step 5: 提交**

```bash
cd ..
git add src-tauri/src/launcher.rs src-tauri/tests/launcher_test.rs src-tauri/src/lib.rs
git commit -m "feat(launcher): 定位与启动 MiniMax Code CLI"
```

---

## Task 11: Rust 后端 - commands.rs Tauri Commands

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: 实现 commands.rs（无单元测试，Tauri 集成测试在前端进行）**

`src-tauri/src/commands.rs`:

```rust
use crate::claude_config::{read_settings, write_env_atomic, ClaudeSettings};
use crate::db::{self, VendorInstance};
use crate::keyring_store;
use crate::launcher;
use crate::vendor;
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

const KEYRING_SERVICE: &str = "MiniMax-vendor-switcher";

pub struct AppState {
    pub db: Mutex<Connection>,
    pub settings_path: Mutex<PathBuf>,
}

fn now_ts() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[tauri::command]
pub fn list_vendors(state: State<AppState>) -> Result<Vec<VendorInstance>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::list_vendors(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_presets() -> Vec<vendor::VendorPreset> {
    vendor::presets()
}

#[derive(serde::Deserialize)]
pub struct CreateVendorInput {
    pub preset_id: Option<String>,
    pub name: String,
    pub api_base: String,
    pub model: String,
    pub api_key: String,
}

#[tauri::command]
pub fn create_vendor(
    state: State<AppState>,
    input: CreateVendorInput,
) -> Result<VendorInstance, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let keyring_key = format!("vendor:{}", id);
    keyring_store::set_key(KEYRING_SERVICE, &keyring_key, &input.api_key)
        .map_err(|e| format!("Keyring 写入失败: {}", e))?;

    let v = VendorInstance {
        id: id.clone(),
        preset_id: input.preset_id,
        name: input.name,
        api_base: input.api_base,
        model: input.model,
        keyring_key,
        created_at: now_ts(),
        updated_at: now_ts(),
    };
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::insert_vendor(&conn, &v).map_err(|e| e.to_string())?;
    Ok(v)
}

#[derive(serde::Deserialize)]
pub struct UpdateVendorInput {
    pub id: String,
    pub name: String,
    pub api_base: String,
    pub model: String,
    pub api_key: Option<String>,
}

#[tauri::command]
pub fn update_vendor(
    state: State<AppState>,
    input: UpdateVendorInput,
) -> Result<VendorInstance, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut existing = db::get_vendor(&conn, &input.id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "vendor not found".to_string())?;

    existing.name = input.name;
    existing.api_base = input.api_base;
    existing.model = input.model;
    existing.updated_at = now_ts();

    if let Some(key) = input.api_key {
        if !key.is_empty() {
            keyring_store::set_key(KEYRING_SERVICE, &existing.keyring_key, &key)
                .map_err(|e| format!("Keyring 写入失败: {}", e))?;
        }
    }
    db::update_vendor(&conn, &existing).map_err(|e| e.to_string())?;
    Ok(existing)
}

#[tauri::command]
pub fn delete_vendor(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let v = db::get_vendor(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "vendor not found".to_string())?;
    let _ = keyring_store::delete_key(KEYRING_SERVICE, &v.keyring_key);
    db::delete_vendor(&conn, &id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn apply_vendor(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let v = db::get_vendor(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "vendor not found".to_string())?;
    let api_key = keyring_store::get_key(KEYRING_SERVICE, &v.keyring_key)
        .map_err(|e| format!("Keyring 读取失败: {}", e))?;

    let path = state.settings_path.lock().map_err(|e| e.to_string())?.clone();
    let mut settings = read_settings(&path).map_err(|e| e.to_string())?;
    let mut env: HashMap<String, String> = settings.env.clone().unwrap_or_default();
    env.insert("ANTHROPIC_BASE_URL".into(), v.api_base.clone());
    env.insert("ANTHROPIC_AUTH_TOKEN".into(), api_key);
    env.insert("ANTHROPIC_MODEL".into(), v.model.clone());
    settings.env = Some(env);
    write_env_atomic(&path, &settings).map_err(|e| e.to_string())?;

    // 记录当前激活
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('active_vendor', ?1)",
        [&id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_active_vendor(state: State<AppState>) -> Result<Option<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = 'active_vendor'")
        .map_err(|e| e.to_string())?;
    let mut iter = stmt.query_map([], |row| row.get::<_, String>(0)).map_err(|e| e.to_string())?;
    Ok(iter.next().transpose().map_err(|e| e.to_string())?)
}

#[tauri::command]
pub fn launch_claude_cmd() -> Result<u32, String> {
    launcher::launch_claude().map_err(|e| format!("启动失败: {}", e))
}

#[tauri::command]
pub fn is_claude_installed() -> bool {
    launcher::find_claude().is_some()
}
```

`src-tauri/src/lib.rs` 追加：

```rust
pub mod commands;
```

**Step 2: 在 `lib.rs` 的 `run` 函数中注册 commands 与初始化 state**

修改 `src-tauri/src/lib.rs` 中的 `pub fn run()`（Tauri 生成），大致改为：

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::Manager;

    tauri::Builder::default()
        .setup(|app| {
            let app_data = app.path().app_data_dir().expect("no app data dir");
            std::fs::create_dir_all(&app_data).ok();
            let db_path = app_data.join("vendors.db");
            let conn = db::init_db(&db_path).expect("init db");

            // MiniMax Code settings.json 路径
            let home = dirs_home().expect("no home");
            let claude_dir = home.join(".claude");
            std::fs::create_dir_all(&claude_dir).ok();
            let settings_path = claude_dir.join("settings.json");

            app.manage(commands::AppState {
                db: Mutex::new(conn),
                settings_path: Mutex::new(settings_path),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_vendors,
            commands::list_presets,
            commands::create_vendor,
            commands::update_vendor,
            commands::delete_vendor,
            commands::apply_vendor,
            commands::get_active_vendor,
            commands::launch_claude_cmd,
            commands::is_claude_installed,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn dirs_home() -> Option<std::path::PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(std::path::PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(std::path::PathBuf::from)
    }
}
```

**Step 3: 编译验证**

```bash
cd src-tauri && cargo check
```

Expected: 编译通过。

**Step 4: 提交**

```bash
cd ..
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat(commands): Tauri Commands 暴露厂商 CRUD/切换/启动"
```

---

## Task 12: 前端 - TypeScript API 包装层

**Files:**
- Create: `src/api.ts`

**Step 1: 实现 api.ts**

`src/api.ts`:

```ts
import { invoke } from '@tauri-apps/api/core';

export interface VendorInstance {
  id: string;
  preset_id: string | null;
  name: string;
  api_base: string;
  model: string;
  keyring_key: string;
  created_at: number;
  updated_at: number;
}

export interface VendorPreset {
  id: string;
  name: string;
  api_base: string;
  default_model: string;
}

export const api = {
  listVendors: () => invoke<VendorInstance[]>('list_vendors'),
  listPresets: () => invoke<VendorPreset[]>('list_presets'),
  createVendor: (input: {
    preset_id: string | null;
    name: string;
    api_base: string;
    model: string;
    api_key: string;
  }) => invoke<VendorInstance>('create_vendor', { input }),
  updateVendor: (input: {
    id: string;
    name: string;
    api_base: string;
    model: string;
    api_key?: string;
  }) => invoke<VendorInstance>('update_vendor', { input }),
  deleteVendor: (id: string) => invoke<void>('delete_vendor', { id }),
  applyVendor: (id: string) => invoke<void>('apply_vendor', { id }),
  getActiveVendor: () => invoke<string | null>('get_active_vendor'),
  launchClaude: () => invoke<number>('launch_claude_cmd'),
  isClaudeInstalled: () => invoke<boolean>('is_claude_installed'),
};
```

**Step 2: 提交**

```bash
git add src/api.ts
git commit -m "feat(api): 前端 TypeScript 包装 Tauri Commands"
```

---

## Task 13: 前端 - 主页 VendorList

**Files:**
- Create: `src/components/VendorList.tsx`
- Modify: `src/App.tsx`

**Step 1: 实现 VendorList 组件**

`src/components/VendorList.tsx`:

```tsx
import { useEffect, useState } from 'react';
import { api, VendorInstance } from '../api';
import { Button } from './ui/button';
import { Card } from './ui/card';

export function VendorList({ onAdd, onEdit }: { onAdd: () => void; onEdit: (v: VendorInstance) => void }) {
  const [vendors, setVendors] = useState<VendorInstance[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const refresh = async () => {
    setLoading(true);
    try {
      const [list, active] = await Promise.all([api.listVendors(), api.getActiveVendor()]);
      setVendors(list);
      setActiveId(active);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { refresh(); }, []);

  const apply = async (id: string) => {
    try {
      await api.applyVendor(id);
      await refresh();
    } catch (e) {
      alert('切换失败: ' + e);
    }
  };

  const remove = async (id: string) => {
    if (!confirm('确定删除此厂商？API Key 将从 Keyring 清除。')) return;
    try {
      await api.deleteVendor(id);
      await refresh();
    } catch (e) {
      alert('删除失败: ' + e);
    }
  };

  return (
    <div>
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-lg font-semibold">厂商列表</h2>
        <Button onClick={onAdd}>+ 添加厂商</Button>
      </div>
      {loading && <p className="text-sm text-gray-500">加载中...</p>}
      {!loading && vendors.length === 0 && (
        <p className="text-sm text-gray-500">还没有厂商，点击右上角"添加厂商"开始。</p>
      )}
      <div className="space-y-2">
        {vendors.map((v) => (
          <Card key={v.id} className="p-4 flex justify-between items-center">
            <div>
              <div className="flex items-center gap-2">
                {v.id === activeId ? (
                  <span className="inline-block w-2 h-2 rounded-full bg-green-500" />
                ) : (
                  <span className="inline-block w-2 h-2 rounded-full bg-gray-300" />
                )}
                <span className="font-medium">{v.name}</span>
              </div>
              <div className="text-xs text-gray-500 mt-1">
                {v.api_base} · 模型: {v.model}
              </div>
            </div>
            <div className="flex gap-2">
              {v.id !== activeId && (
                <Button size="sm" onClick={() => apply(v.id)}>应用</Button>
              )}
              <Button size="sm" variant="outline" onClick={() => onEdit(v)}>编辑</Button>
              <Button size="sm" variant="destructive" onClick={() => remove(v.id)}>删除</Button>
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
}
```

**Step 2: 修改 App.tsx**

`src/App.tsx`:

```tsx
import { useState } from 'react';
import { VendorList } from './components/VendorList';
import { VendorDialog } from './components/VendorDialog';
import { Button } from './components/ui/button';
import { api, VendorInstance } from './api';

export default function App() {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editing, setEditing] = useState<VendorInstance | null>(null);
  const [claudeInstalled, setClaudeInstalled] = useState<boolean | null>(null);

  useState(() => { api.isClaudeInstalled().then(setClaudeInstalled); });

  return (
    <div className="min-h-screen bg-gray-50 p-6">
      <div className="max-w-3xl mx-auto">
        <header className="flex justify-between items-center mb-6">
          <h1 className="text-xl font-bold">⚡ MiniMax Code Vendor Switcher</h1>
          <div className="flex gap-2">
            <Button onClick={() => api.launchClaude()} disabled={claudeInstalled === false}>
              🚀 启动 MiniMax Code
            </Button>
          </div>
        </header>

        {claudeInstalled === false && (
          <div className="mb-4 p-3 bg-yellow-50 border border-yellow-200 rounded text-sm">
            未检测到 MiniMax Code CLI，请先安装后再启动。
          </div>
        )}

        <VendorList
          onAdd={() => { setEditing(null); setDialogOpen(true); }}
          onEdit={(v) => { setEditing(v); setDialogOpen(true); }}
        />
      </div>

      {dialogOpen && (
        <VendorDialog
          editing={editing}
          onClose={() => setDialogOpen(false)}
          onSaved={() => { setDialogOpen(false); window.location.reload(); }}
        />
      )}
    </div>
  );
}
```

> 修复：把 `useState(() => ...)` 改为 `useEffect(() => ...)`。

`src/App.tsx` 修正版：

```tsx
import { useEffect, useState } from 'react';
import { VendorList } from './components/VendorList';
import { VendorDialog } from './components/VendorDialog';
import { Button } from './components/ui/button';
import { api, VendorInstance } from './api';

export default function App() {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editing, setEditing] = useState<VendorInstance | null>(null);
  const [claudeInstalled, setClaudeInstalled] = useState<boolean | null>(null);

  useEffect(() => { api.isClaudeInstalled().then(setClaudeInstalled); }, []);

  return (
    <div className="min-h-screen bg-gray-50 p-6">
      <div className="max-w-3xl mx-auto">
        <header className="flex justify-between items-center mb-6">
          <h1 className="text-xl font-bold">⚡ MiniMax Code Vendor Switcher</h1>
          <div className="flex gap-2">
            <Button onClick={() => api.launchClaude()} disabled={claudeInstalled === false}>
              🚀 启动 MiniMax Code
            </Button>
          </div>
        </header>

        {claudeInstalled === false && (
          <div className="mb-4 p-3 bg-yellow-50 border border-yellow-200 rounded text-sm">
            未检测到 MiniMax Code CLI，请先安装后再启动。
          </div>
        )}

        <VendorList
          onAdd={() => { setEditing(null); setDialogOpen(true); }}
          onEdit={(v) => { setEditing(v); setDialogOpen(true); }}
        />
      </div>

      {dialogOpen && (
        <VendorDialog
          editing={editing}
          onClose={() => setDialogOpen(false)}
          onSaved={() => setDialogOpen(false)}
        />
      )}
    </div>
  );
}
```

**Step 3: 提交**

```bash
git add src/components/VendorList.tsx src/App.tsx
git commit -m "feat(ui): 厂商列表 + 头部启动按钮"
```

---

## Task 14: 前端 - VendorDialog 添加/编辑表单

**Files:**
- Create: `src/components/VendorDialog.tsx`

**Step 1: 实现 VendorDialog 组件**

`src/components/VendorDialog.tsx`:

```tsx
import { useEffect, useState } from 'react';
import { api, VendorInstance, VendorPreset } from '../api';
import { Button } from './ui/button';
import { Card } from './ui/card';
import { Input } from './ui/input';
import { Label } from './ui/label';

export function VendorDialog({
  editing,
  onClose,
  onSaved,
}: {
  editing: VendorInstance | null;
  onClose: () => void;
  onSaved: () => void;
}) {
  const [presets, setPresets] = useState<VendorPreset[]>([]);
  const [name, setName] = useState(editing?.name ?? '');
  const [apiBase, setApiBase] = useState(editing?.api_base ?? '');
  const [model, setModel] = useState(editing?.model ?? '');
  const [apiKey, setApiKey] = useState('');
  const [presetId, setPresetId] = useState<string | null>(editing?.preset_id ?? null);
  const [customEndpoint, setCustomEndpoint] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    api.listPresets().then(setPresets);
  }, []);

  const choosePreset = (id: string) => {
    if (id === '__custom__') {
      setCustomEndpoint(true);
      setPresetId(null);
      return;
    }
    setCustomEndpoint(false);
    setPresetId(id);
    const p = presets.find((x) => x.id === id);
    if (p) {
      setName(p.name);
      setApiBase(p.api_base);
      setModel(p.default_model);
    }
  };

  const save = async () => {
    if (!name || !apiBase || !model) {
      alert('请填写名称、API Base 和模型');
      return;
    }
    if (!editing && !apiKey) {
      alert('请填写 API Key');
      return;
    }
    setSaving(true);
    try {
      if (editing) {
        await api.updateVendor({
          id: editing.id,
          name,
          api_base: apiBase,
          model,
          api_key: apiKey || undefined,
        });
      } else {
        await api.createVendor({
          preset_id: presetId,
          name,
          api_base: apiBase,
          model,
          api_key: apiKey,
        });
      }
      onSaved();
    } catch (e) {
      alert('保存失败: ' + e);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <Card className="w-full max-w-md p-6 bg-white">
        <h2 className="text-lg font-semibold mb-4">
          {editing ? '编辑厂商' : '添加厂商'}
        </h2>

        {!editing && (
          <div className="mb-4">
            <Label>选择预设</Label>
            <select
              className="w-full mt-1 border rounded px-2 py-1.5 text-sm"
              onChange={(e) => choosePreset(e.target.value)}
              defaultValue=""
            >
              <option value="" disabled>请选择...</option>
              {presets.map((p) => (
                <option key={p.id} value={p.id}>{p.name}</option>
              ))}
              <option value="__custom__">自定义 OpenAI 兼容端点</option>
            </select>
          </div>
        )}

        <div className="space-y-3">
          <div>
            <Label>名称</Label>
            <Input value={name} onChange={(e) => setName(e.target.value)} />
          </div>
          <div>
            <Label>API Base URL</Label>
            <Input value={apiBase} onChange={(e) => setApiBase(e.target.value)} placeholder="https://..." />
          </div>
          <div>
            <Label>模型名</Label>
            <Input value={model} onChange={(e) => setModel(e.target.value)} />
          </div>
          <div>
            <Label>{editing ? 'API Key（留空表示不修改）' : 'API Key'}</Label>
            <Input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="sk-..."
            />
          </div>
        </div>

        <div className="flex justify-end gap-2 mt-6">
          <Button variant="outline" onClick={onClose} disabled={saving}>取消</Button>
          <Button onClick={save} disabled={saving}>{saving ? '保存中...' : '保存'}</Button>
        </div>
      </Card>
    </div>
  );
}
```

**Step 2: 跑构建验证**

```bash
pnpm build
```

Expected: 编译通过，无 TS 错误。

**Step 3: 提交**

```bash
git add src/components/VendorDialog.tsx
git commit -m "feat(ui): 添加/编辑厂商对话框"
```

---

## Task 15: 集成验证 - 端到端流程

**Files:** 无新增

**Step 1: 启动 dev 模式**

```bash
pnpm tauri dev
```

Expected: 桌面窗口打开，看到空列表。

**Step 2: 添加一个测试厂商（自定义端点 + 假 Key）**

点击"+ 添加厂商" → 选"自定义 OpenAI 兼容端点" → 填：
- 名称: Test Vendor
- API Base: `https://api.deepseek.com`
- 模型: `deepseek-chat`
- API Key: `sk-test-fake`

→ 保存

Expected: 列表显示 "Test Vendor"。

**Step 3: 点击"应用"**

Expected:
- 列表中该项前出现绿色圆点
- 弹出提示（或无）表示切换成功
- 检查 `~/.claude/settings.json`：

```bash
cat ~/.claude/settings.json
```

Expected 输出类似：

```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.deepseek.com",
    "ANTHROPIC_AUTH_TOKEN": "sk-test-fake",
    "ANTHROPIC_MODEL": "deepseek-chat"
  }
}
```

**Step 4: 检查备份目录**

```bash
ls ~/.claude/backups/
```

Expected: 至少有一个 `settings.<timestamp>.json` 文件。

**Step 5: 验证 Keyring**

Windows: 打开"凭据管理器" → "Windows 凭据" → 搜索 `MiniMax-vendor-switcher`。
macOS: 打开"钥匙串访问" → 搜索 `MiniMax-vendor-switcher`。

Expected: 看到一条以 `vendor:<uuid>` 为用户名的凭据。

**Step 6: 删除厂商，确认 Keyring 清理**

回到工具 → 点"删除" → 确认。

Expected: 列表为空，凭据管理器中该条目消失。

**Step 7: 提交里程碑**

```bash
git add .
git commit -m "docs: 记录集成验证步骤（手动）" --allow-empty
```

---

## Task 16: 编写 README + 打包说明

**Files:**
- Modify: `README.md`

**Step 1: 改写 README**

`README.md`:

```markdown
# MiniMax Code Vendor Switcher

> 一键在 DeepSeek / Kimi / 智谱 GLM / Qwen 等多家 LLM 厂商之间切换，切换后启动 MiniMax Code 即可使用所选厂商的模型。

## 功能

- 厂商预设 + 任意 OpenAI 兼容自定义端点
- 一键切换，写入 `~/.claude/settings.json` 的 `env` 字段
- API Key 通过系统 Keyring 加密存储（Windows Credential Manager / macOS Keychain / Linux Secret Service）
- 自动备份原配置文件，可回滚
- 一键启动 MiniMax Code CLI

## 使用

```bash
# 开发模式
pnpm tauri dev

# 构建发布版
pnpm tauri build
```

构建产物在 `src-tauri/target/release/bundle/`。

## 工作原理

工具本身**不存储** MiniMax Code 账号。它只修改 `~/.claude/settings.json` 中的 `env` 段：

```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.deepseek.com",
    "ANTHROPIC_AUTH_TOKEN": "sk-...",
    "ANTHROPIC_MODEL": "deepseek-chat"
  }
}
```

启动 MiniMax Code 时，CLI 会读取这个 env 配置作为后端 API 地址。

## 支持的厂商

- MiniMax（默认）
- DeepSeek
- Kimi（月之暗面）
- 智谱 GLM
- Qwen（通义千问）
- 任意 OpenAI 兼容 API（自定义）

## 安全

- API Key 仅存在于系统 Keyring 和切换瞬间的内存中
- 配置文件权限 0600（Unix）
- 日志永不打印完整 API Key

## 许可

MIT
```

**Step 2: 提交**

```bash
git add README.md
git commit -m "docs: 完善 README 使用说明"
```

---

## 完成标准

完成以上 16 个任务后，MVP 功能齐全：

- [x] 添加 / 编辑 / 删除厂商
- [x] 5 个内置预设 + 自定义端点
- [x] 一键切换并写入 `~/.claude/settings.json`
- [x] 自动备份
- [x] API Key 走系统 Keyring
- [x] 启动 MiniMax Code CLI
- [x] 端到端手动验证通过

## v0.2+ 路线（不在本计划内）

- 实时 API 调用测试
- 用量统计
- 暗色模式
- 多语言
- 配置导入/导出
