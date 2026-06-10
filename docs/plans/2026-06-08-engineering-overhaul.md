# 观景 VISTA — 工程整改开发书

> **版本**: v1.1
> **日期**: 2026-06-08
> **状态**: 待审阅
> **分析方法**: 全量静态分析 + cargo test/clippy/build(dev+release) + npm build(tsc+vite) + GLM-5V-Turbo 深度审计

---

## 目录

1. [项目重新定档](#1-项目重新定档)
2. [当前问题全景](#2-当前问题全景)
3. [架构整改方案](#3-架构整改方案)
4. [阶段性执行计划](#4-阶段性执行计划)
5. [测试策略](#5-测试策略)
6. [附录：预计工时](#6-预计工时)

---

## 1. 项目重新定档

### 定位修正

```
原定: MiniMax Code Vendor Switcher — 单一工具的厂商切换器
修正: 观景 VISTA — 中文 AI 编码工具的统一配置控制台
```

### 核心命题

`观景` 解决一个核心问题：用户手上有 18+ 个 AI 编码工具（Claude Code / MiniMax Code / Codex CLI / Qwen / Aider / Kimi / OpenCode / Grok Build / OpenClaw / Hermes / Nanobot...），每个工具有**不同的配置文件格式、不同的环境变量名、不同的 API 协议**。观景要做的事情是：

> **用户只需要录入一次厂商和 API Key，观景自动为每个已安装的工具生成正确的配置文件。**

切换工具时，用户不需要知道那个工具用 YAML 还是 TOML、环境变量叫 `ANTHROPIC_AUTH_TOKEN` 还是 `OPENAI_API_KEY`。

### MVP 边界

| 范围 | 包含 |
|------|------|
| **核心功能** | 厂商 CRUD、Keyring 安全存储、工具检测、配置写入、绑定管理 |
| **必做** | 安全加固、数据一致性、错误处理、配置正确性 |
| **不做（v0.1）** | 多语言、暗色模式、用量统计、配置导入/导出、实时 API 测试、代理设置、插件系统 |

---

## 2. 当前问题全景

### 2.1 严重程度分级

| 等级 | 含义 | 数量 |
|------|------|------|
| 🔴 **P0 阻塞** | 功能不可用、数据损坏、安全风险 | 8 |
| 🟠 **P1 严重** | 功能异常、用户体验严重受损 | 10 |
| 🟡 **P2 一般** | 非核心路径问题、边界情况 | 10 |
| 🔵 **P3 优化** | 代码质量、可维护性 | 8 |
| ⚪ **结构问题** | 架构层面需要重构的决策 | 5 |

### 2.2 完整问题清单

#### 🔴 P0 — 阻塞级

| # | 问题 | 影响 | 根因 |
|---|------|------|------|
| P0-1 | **MiniMax 预设 API URL 写错** | `vendor.rs:13` → `api.MiniMax.com` 应改为 `api.minimaxi.com`。预设配置直接无法连接 | 录入时笔误 |
| P0-2 | **Invoke-Expression 远程代码执行** | `installer.rs:98` → 用 `Invoke-Expression` 执行远程下载脚本。MITM/源站被入侵 = 用户机器被控 | 安装方式实现不当 |
| P0-3 | **API Key 以纯文本无保护写入磁盘（Windows）** | `claude_config.rs:78-84` → 0600 权限只在 Unix 生效，Windows 上所有用户可读 | 跨平台权限遗漏 |
| P0-4 | **绑定流程操作顺序错误导致数据不一致** | `commands.rs:244-282` → 顺序：Keyring写入 → DB事务 → 清理旧Keyring → 配置文件。若最后一步配置文件写入失败，DB已提交、旧Keyring已清理，应用处于坏状态无法恢复 | 无 Saga 回滚模式 |
| P0-5 | **`std::sync::Mutex` 阻塞 Tauri 异步线程池** | `commands.rs` 全文件 → Tauri 2.0 Command 默认运行在异步线程池。所有 `state.db.lock()` 使用标准库 Mutex，锁竞争时**硬阻塞**异步线程。高并发导致整个应用 UI 卡死 | 同步锁用于异步上下文 |
| P0-6 | **并发绑定竞态 — 两个请求互相删除对方 Key** | `commands.rs:285-287` → 用户快速点击两次"保存模型配置"：线程A写入Key_A→DB生效，线程B写入Key_B→DB生效并拿到旧Key列表（含Key_A）→删除Key_A。最终Key_A消失，线程A的绑定配置不可用 | 无工具级别细粒度锁 |
| P0-7 | **并发配置写入竞态 — 文件错乱** | `commands.rs:290` → 两次并发 `apply_config_for_tool` 可能同时读写同一配置文件，内容交错损坏 | 无文件锁/排队机制 |
| P0-8 | **旧 `apply_vendor` 写 minimax.yaml，新 `apply_binding` 写各工具路径** | `lib.rs:34-35` + `commands.rs:127-131` → `apply_vendor` 固定写 `~/.minimax/config.yaml`。用户通过UI执行"切换厂商"时，旧命令把Claude Code的配置写到了MiniMax配置文件 | 新旧命令数据流割裂，双轨制配置 |

#### 🟠 P1 — 严重级

| # | 问题 | 影响 | 根因 |
|---|------|------|------|
| P1-1 | **持 Mutex 锁期间做文件 I/O** | `commands.rs` 多处 → `state.db.lock()` 后在锁内调用 Keyring / 文件写入。慢 I/O 会阻塞所有 Tauri Command，UI 直接卡死 | 锁粒度过粗 + 同步锁用于异步 |
| P1-2 | **手动 SQLite 事务嵌套导致隐式提交** | `db.rs:659` + `commands.rs:258` → 若在外层事务中调用 `delete_provider_cascade`，其 `BEGIN` 会隐式 COMMIT 外层事务，数据部分丢失 | 无事务管理器 |
| P1-3 | **切换工具后旧厂商/模型状态未清除** | `App.tsx:544-553` → `getToolBinding` 只在有 binding 时重置选择，无 binding 时保留旧值，用户无感知 | 状态重置逻辑缺失 |
| P1-4 | **`get_tool_binding` 全表扫描** | `commands.rs:433` → `list_models` 拉取所有模型再 `find`，大量模型时显著变慢（O(n) vs O(1)） | 缺少按 ID 查询 |
| P1-5 | **`update_vendor` 持锁期间更新配置** | `commands.rs:89-107` → 持有 DB 锁的同时调用 `minimax_config::apply_provider`（文件读写），违反锁层级原则 | 锁设计错误 |
| P1-6 | **迁移 `migrate_legacy_provider_keys` 在事务内调用 Keyring** | `db.rs:356-376` → 迁移循环内调用 `keyring_store::set_key`。若后续迁移步骤（如 `seed_default_tools`）失败导致 ROLLBACK，Keyring 中的 Key 不会被撤回 → 数据不一致 | 将不可回滚的副作用置于事务内 |
| P1-7 | **`fetch_provider_key` 释放锁再重获锁期间状态变化** | `commands.rs:294-312` → `drop(conn)` → Keyring IO → `state.db.lock()`。两次获得锁之间，其他线程可能修改/删除了该 provider | 没有用事务保护"读-判断-写"序列 |
| P1-8 | **`create_vendor` / `create_provider` Keyring 失败回滚不彻底** | `commands.rs:78-80` / `commands.rs:188-190` → DB 插入后 Keyring 失败尝试回删 DB。若回删也失败，DB 出现无 Key 的孤儿厂商 | 两阶段提交未实现 |
| P1-9 | **`delete_provider` 清理 Keyring 失败时只返回错误，不撤回 DB 操作** | `commands.rs:215-229` → DB 已删除但 Keyring 清理失败，错误被返回给前端需用户手动处理 | 未实现补偿事务 |
| P1-10 | **`apply_binding` 配置写入失败后无回滚** | `commands.rs:290` → config 写入失败直接 return Err，但 DB 已提交、Keyring 已写入、旧 Keyring 已删除。用户看到错误但状态已改变 | 操作顺序错误 |

#### 🟡 P2 — 一般级

| # | 问题 | 影响 |
|---|------|------|
| P2-1 | `formatModelStatus` 假阳性 — 有 Key 就显示"就绪"，不验证有效性 | 误导用户 |
| P2-2 | `load()` 闭包的 `selectedToolId` 引用是 Stale — `useCallback([],...)` | 非首次调用时条件判断用过期值 |
| P2-3 | `Number.parseInt(ctxLen, 10) || 128000` → 输入 `0` 静默回退为 `128000` | 不合理 |
| P2-4 | `dirs_home()` 在 6 个文件重复实现 6 次 | 维护成本，改一处遗漏五处 |
| P2-5 | `now_ts()` 在 `commands.rs` 和 `db.rs` 重复实现 | 同上 |
| P2-6 | `seed_default_tools` 中多个 CLI 工具的 `env_keys_json` 和 `launch_path` 为 None | 前端显示/启动时信息缺失 |
| P2-7 | 桌面路径硬编码中文目录 `Desktop\ai编程\` | 部分用户可能不同安装路径 |
| P2-8 | **`installer.rs` 通过 Desktop 拼接下载路径，非标准用户目录** | 若 Desktop 不存在或重定向到 OneDrive，下载到错误位置 |
| P2-9 | **`apply_config_for_tool` 内 `tool_id` 前 4 个 if 分支重复代码（Claude CLI 和 Claude Desktop 逻辑完全一致）** | 代码冗余，新增桌面版容易遗漏 |
| P2-10 | **`seed_default_tools` 中 `env_keys_json` 全为 None/空，但官方字段设计中该字段应存储环境变量映射关系** | 功能预留但从未实现，且旧代码无处使用此字段 |

#### 🔵 P3 — 优化级

| # | 问题 |
|---|------|
| P3-1 | `normalize_tool_configs` 与 `seed_default_tools` 重复设置数据 |
| P3-2 | `seed_default_providers` 空函数但仍被调用，增加困惑 |
| P3-3 | `handleChat`/`handleListModels` 中 HTTP 客户端 error 处理使用 `error_snippet` 截断 200 字符，但缺少详细日志 |
| P3-4 | `installer.rs` 的 `Manual` 分支返回 `Err` 而不是引导用户（但前端无处理逻辑） |
| P3-5 | 前端 `ProviderEditorModal` 中预设选择器的 `value` 绑定 `selectedPresetId` 但初始值可能不匹配选项 |
| P3-6 | 配置文件备份目录 `backups/` 没有 `.gitignore`，也不在 Tauri 打包排除列表中 |

### 2.3 架构性根本问题

| # | 问题 | 说明 |
|---|------|------|
| A-1 | **无统一的错误模型** | 后端使用 `Result<T, String>`，所有错误被转换为字符串。前端无法区分错误类型（网络/权限/数据不存在），统一弹出 `alert`。应定义枚举错误类型 + 前端错误码系统 |
| A-2 | **Config 写入层耦合过深** | `commands.rs` 的 `apply_config_for_tool` 直接硬编码 14 个工具的写入逻辑。新增工具需要改该函数。应改为策略模式或注册表模式 |
| A-3 | **缺少关注点分离** | `db.rs` 混用数据模型定义、CRUD、迁移、种子数据。`commands.rs` 混用参数校验、业务逻辑、Keyring 操作。应拆分为 repository / service / handler 三层 |
| A-4 | **无统一的配置原子写入封装** | `claude_config.rs` 有原子写入，`minimax_config.rs` 有独立实现，`tool_configs.rs` 又有自己的 `write_atomic`。应抽象为公共模块 |
| A-5 | **`AppState.config_path` 全局持有固定路径是架构错误** | `lib.rs:34-35` → config_path 固定为 `~/.minimax/config.yaml`，但 tools 表中每个工具有不同的配置路径。`apply_vendor` 还是用这个固定路径。导致：<br>1. 旧命令把 Claude 的配置写到了 MiniMax 配置文件<br>2. `AppState` 不应持有业务路径状态<br>3. config_path 应由业务层根据 tool_id 动态解析 |

---

## 3. 架构整改方案

### 3.1 目标架构

```
┌────────────────────────────────────────────────────────┐
│  Frontend (React + TypeScript)                         │
│  - 按功能拆分组件（非单文件 App.tsx）                    │
│  - 统一错误处理（ErrorBoundary + Toast + ErrorCode）     │
│  - 状态管理（React Query / Zustand 替代手动 useState）    │
└───────────────────────┬────────────────────────────────┘
                        │ Tauri IPC
┌───────────────────────▼────────────────────────────────┐
│  Backend (Rust)                                        │
│                                                        │
│  handlers/     ← 薄 Tauri Command 层，校验参数后委托     │
│    vendor_cmds.rs                                      │
│    provider_cmds.rs                                    │
│    binding_cmds.rs                                     │
│    tool_cmds.rs                                        │
│                                                        │
│  services/     ← 业务逻辑层，无锁无 IO 知识              │
│    provider_service.rs                                 │
│    binding_service.rs                                  │
│    detection_service.rs                                │
│                                                        │
│  repository/   ← 数据访问层（SQLite + Keyring）          │
│    provider_repo.rs                                    │
│    tool_repo.rs                                        │
│    binding_repo.rs                                     │
│                                                        │
│  config_writer/ ← 配置写入策略层（解耦工具类型）          │
│    writer_trait.rs      → ConfigWriter trait            │
│    writer_claude.rs     → Claude 格式                   │
│    writer_minimax.rs    → YAML 格式                     │
│    writer_codex.rs      → TOML 格式                     │
│    writer_registry.rs   → 注册表，根据 tool_id 分发       │
│                                                        │
│  common/       ← 共享工具                              │
│    errors.rs            → 统一的错误类型                │
│    atomic_io.rs         → 原子写入 + 备份               │
│    path_util.rs         → dirs_home() 等                │
│    time_util.rs         → now_ts() 等                   │
└────────────────────────────────────────────────────────┘
```

### 3.2 关键架构决策

#### 决策 1：错误模型

```rust
// common/errors.rs
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Keyring 错误: {0}")]
    Keyring(#[from] crate::keyring_store::KeyringError),
    #[error("配置写入错误: {0}")]
    ConfigWrite(#[from] crate::config_writer::ConfigError),
    #[error("工具未找到: {0}")]
    ToolNotFound(String),
    #[error("厂商未找到: {0}")]
    ProviderNotFound(String),
    #[error("厂商缺少 API Key: {0}")]
    MissingApiKey(String),
    #[error("不支持的厂商格式: {0}")]
    UnsupportedMode(String),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}
```

前端接收结构化错误对象，根据 `code` 字段显示不同 UI（Toast / Alert / 内联提示），而非全部 `alert()`。

#### 决策 2：配置写入策略模式

```rust
// config_writer/writer_trait.rs
#[async_trait]
pub trait ConfigWriter: Send + Sync {
    fn tool_id(&self) -> &'static str;
    fn apply(&self, params: WriteParams) -> Result<(), ConfigError>;
}

// config_writer/writer_registry.rs
pub struct WriterRegistry {
    writers: HashMap<&'static str, Box<dyn ConfigWriter>>,
}
impl WriterRegistry {
    pub fn get(&self, tool_id: &str) -> Option<&dyn ConfigWriter> { ... }
}
```

新增工具只需新增一个 `Writer` 实现 + 注册到 `Registry`，`commands.rs` 不需要修改。

#### 决策 3：锁层级

```rust
// 原则：DB Mutex 保护范围仅限于 SQLite 操作
// Keyring / 文件 I/O 必须在锁外执行

// ❌ 错误（当前代码）
let conn = state.db.lock()?;
keyring_store::set_key(...)?;       // ️ 持锁期间 IO
minimax_config::apply_provider(...)?; // ️ 持锁期间 IO

// ✅ 正确
let api_key = {
    let conn = state.db.lock()?;    // 持锁 ← 只读 DB
    get_key_from_db(&conn)?
};  // 尽早释放
keyring_store::set_key(...)?;        // 锁外 IO
minimax_config::apply_provider(...)?; // 锁外 IO
```

#### 决策 4：事务安全

不再手动 `conn.execute_batch("BEGIN")` 管理事务。统一使用事务闭包：

```rust
pub fn with_transaction<F, T>(conn: &Connection, f: F) -> Result<T, AppError>
where
    F: FnOnce(&Transaction) -> Result<T, AppError>,
{
    let tx = conn.transaction()?;    // 自动管理嵌套
    let result = f(&tx)?;
    tx.commit()?;
    Ok(result)
}
```

Rusqlite 的 `Transaction` 对象在 `DROP` 时自动 ROLLBACK，且由 SQLite 自身处理嵌套事务（SAVEPOINT）。

### 3.3 前端状态管理整改

**当前问题**：`App.tsx` 约 1258 行，所有逻辑耦合在一个文件中。`useState` + `useEffect` 散落 20+ 状态变量，数据流不清晰。

**目标**：

```
App.tsx → 仅路由/布局

ModelCenterPage.tsx → 模型中心（厂商 + 模型 CRUD）
ApplicationStudioPage.tsx → 应用管理（工具绑定）
ProviderEditorModal.tsx → 厂商编辑弹窗
ModelEditorModal.tsx → 模型编辑弹窗

hooks/
  useProviders.ts   → 厂商数据加载 + 操作
  useModels.ts      → 模型数据加载 + 操作
  useTools.ts       → 工具检测 + 绑定

components/
  layout/           → 侧边栏、顶栏
  common/           → Modal、Tag、SummaryCard 等复用组件
  studio/           → 应用管理专属组件
```

**状态管理**：引入 `@tanstack/react-query` 替代手动 `useState` + `useEffect` 的数据加载。

---

## 4. 阶段性执行计划

### 阶段 0：基础工程准备（预估 0.5 天）

**目标**：建立架构地基，不改动业务逻辑

| 任务 | 文件 | 说明 |
|------|------|------|
| 0.1 | `common/errors.rs` | 创建统一错误类型，Tauri Command 返回 `Result<T, AppError>` |
| 0.2 | `common/atomic_io.rs` | 抽取原子写入 + 备份逻辑（合并 `claude_config` / `minimax_config` / `tool_configs` 三处实现） |
| 0.3 | `common/path_util.rs` | 抽取 `dirs_home()`，删除其他 5 处的重复实现 |
| 0.4 | `common/time_util.rs` | 抽取 `now_ts()`，删除重复实现 |
| 0.5 | 项目结构重组 | 按 `handlers/` + `services/` + `repository/` + `config_writer/` 拆分模块 |
| 0.6 | `Cargo.toml` | 添加 `rustc-serialize` 等必要依赖，更新 `mod.rs` |

### 阶段 1：修复 P0 阻塞 Bug（预估 2 天）

**目标**：消除安全风险、数据损坏风险、异步死锁风险和双轨配置

**关键原则**：
- 所有 Tauri Command 改为 `async fn`，使用 `tokio::sync::Mutex` 替代 `std::sync::Mutex`
- 配置写入操作在 DB 操作之前执行（先写配置后写 DB，实现向前恢复）
- 工具级别的细粒度锁防止并发绑定冲突

| 任务 | 严重性 | 文件 | 改动 |
|------|--------|------|------|
| 1.1 | 🔴 P0-1 | `vendor.rs:13` | 改 `api.MiniMax.com` → `api.minimaxi.com` |
| 1.2 | 🔴 P0-2 | `installer.rs:97-113` | `Curl` 安装改为仅下载到临时目录 + 提示用户手动运行。**移除 `Invoke-Expression`** |
| 1.3 | 🔴 P0-3 | `claude_config.rs:78-84` | Windows 上用 `icacls` / `windows-acl` crate 设置权限仅当前用户可读 |
| 1.4 | 🔴 P0-4 + P1-10 | `commands.rs:233-292` | `apply_binding` 重构为新顺序：配置文件 → Keyring → DB 事务。DB 失败时回滚配置和 Keyring |
| 1.5 | 🔴 P0-5 | `AppState` 结构 + `commands.rs` | `std::sync::Mutex<Connection>` → `tokio::sync::Mutex<Connection>`。所有 Command 改为 `async fn` |
| 1.6 | 🔴 P0-6 | `commands.rs:apply_binding` | 引入 `tool_locks: HashMap<String, Arc<tokio::sync::Mutex<()>>>`，同一 tool_id 的绑定请求排队执行 |
| 1.7 | 🔴 P0-7 | `common/atomic_io.rs` | 文件写入全部使用"写临时文件 → `fs::rename` 原子替换"模式，消除并发写入文件损坏 |
| 1.8 | 🔴 P0-8 | `lib.rs:34-35` | **从 `AppState` 中移除 `config_path`**。旧 `apply_vendor` 改为通过 `tool_id` 动态解析路径，或标记为 Deprecated 直接删除 |

### 阶段 2：修复 P1 严重 Bug（预估 2 天）

| 任务 | 文件 | 改动 |
|------|------|------|
| 2.1 | `commands.rs` 全文件 | 审计所有持锁模式（阶段 1 进行中），将 Keyring / 文件 I/O 移到锁外 |
| 2.2 | `db.rs` + `commands.rs` | 用 `conn.transaction()` / `with_transaction()` 替换所有 `execute_batch("BEGIN/COMMIT")`，消除嵌套事务隐式提交 |
| 2.3 | `db.rs` | 添加 `get_model_by_id(conn, id)` 替代 `list_models().find()` |
| 2.4 | `App.tsx:544-553` | `getToolBinding` 无论有无 binding 都重置选择状态 |
| 2.5 | `commands.rs:89-107` | 重构 `update_vendor` 锁范围：先读 DB 释放锁，再更新 Keyring，最后再持锁写 DB |
| 2.6 | `db.rs:356-376` | `migrate_legacy_provider_keys` 移出迁移事务。改为：先遍历收集需要迁移的 (id, key)，提交事务，再逐个迁移 |
| 2.7 | `commands.rs:294-312` | `fetch_provider_key` 重构：使用原子性的"读DB → 写Keyring → 清DB"事务，或在重获锁后重新验证 provider 是否存在 |
| 2.8 | `commands.rs:78-80` + `188-190` | `create_vendor`/`create_provider`：先写 Keyring，成功后再写 DB。Keyring 失败时无需回滚 DB |
| 2.9 | `commands.rs:198-230` | `delete_provider`：Keyring 清理先于 DB 删除。Keyring 失败时 DB 操作不执行 |

### 阶段 3：配置写入层解耦（预估 2 天）

**目标**：消除 `apply_config_for_tool` 的 14 路硬编码分支

| 任务 | 文件 | 说明 |
|------|------|------|
| 3.1 | `config_writer/writer_trait.rs` | 定义 `ConfigWriter` trait |
| 3.2 | `config_writer/writer_registry.rs` | 注册表 |
| 3.3 | `config_writer/writer_claude.rs` | 从 `claude_config.rs` 迁移（保持原有实现） |
| 3.4 | `config_writer/writer_minimax.rs` | 从 `minimax_config.rs` 迁移 |
| 3.5 | `config_writer/writer_tool_configs.rs` | 从 `tool_configs.rs` 拆分（Codex / Qwen / Aider / Kimi / Grok / OpenCode） |
| 3.6 | `config_writer/writer_agent.rs` | 从 `agent_adapters.rs` 迁移 |
| 3.7 | `commands.rs:318-398` | 替换为 `registry.get(tool_id)?.apply(params)?` |

### 阶段 4：前端工程化（预估 2 天）

| 任务 | 说明 |
|------|------|
| 4.1 | 拆分 `App.tsx` 为独立页面文件 |
| 4.2 | 引入 `@tanstack/react-query`，替换所有手动 `useEffect` 数据加载 |
| 4.3 | 统一错误处理：`alert()` → Toast 通知 + 错误码识别 |
| 4.4 | 修复 Stale Closure（`useCallback` 依赖数组） |
| 4.5 | 修复 `formatModelStatus` 假阳性 |
| 4.6 | 修复 `Number.parseInt` 的 falsy 回退问题 |

### 阶段 5：P2 修复 + 测试覆盖（预估 2 天）

| 任务 | 说明 |
|------|------|
| 5.1 | `seed_default_tools` 补全 `env_keys_json` 和 `launch_path` 值 |
| 5.2 | 删除 `normalize_tool_configs` 多余更新 |
| 5.3 | 删除或实现 `seed_default_providers` |
| 5.4 | 为每个 Repository 模块编写单元测试（Mock 的 SQLite） |
| 5.5 | 为每个 ConfigWriter 编写集成测试（临时目录） |
| 5.6 | 前端组件测试（Vitest + Testing Library） |

### 阶段 6：文档与收尾（预估 0.5 天）

| 任务 | 说明 |
|------|------|
| 6.1 | 更新 README.md 匹配新定位 |
| 6.2 | 更新 DESIGN.md 匹配实际实现 |
| 6.3 | 更新 `tauri.conf.json` 描述等元数据 |
| 6.4 | 补充 `CHANGELOG.md` |

---

## 5. 测试策略

### 5.1 测试金字塔

```
        ╱╲
       ╱  ╲          E2E: 手动验证（首次发布前）
      ╱    ╲
     ╱ E2E  ╲
    ╱────────╲
   ╱          ╲       集成测试: ConfigWriter × 工具类型
  ╱ Integration ╲     Repository × 数据库操作
 ╱────────────────╲
╱                  ╲  单元测试: 工具函数、错误处理、
╱   Unit Tests      ╲ 路径解析、配置合并
╱────────────────────╲
```

### 5.2 测试框架

| 层 | 框架 |
|----|------|
| Rust 后端 | `cargo test` + `tempfile`（临时目录测试文件写入） |
| Rust 集成 | `rusqlite` in-memory database |
| 前端组件 | Vitest + Testing Library |
| 前端 E2E | 手动验证清单 |

### 5.3 关键测试用例

```rust
// ConfigWriter 测试 — 每个 writer 必须有
#[test]
fn writer_claude_merges_preserves_other_fields() { ... }
#[test]
fn writer_minimax_new_provider_creates_full_entry() { ... }
#[test]
fn writer_minimax_update_provider_only_updates_options() { ... }
#[test]
fn writer_codex_wires_api_by_mode_anthropic() { ... }
#[test]
fn writer_codex_wires_api_by_mode_openai() { ... }
#[test]
fn writer_opencode_merges_provider_and_model() { ... }

// 事务测试
#[test]
fn transaction_rollback_does_not_leak_keys() { ... }

// 安全测试
#[test]
fn installer_curl_does_not_invoke_expression() { ... }
```

### 5.4 手动验证清单

| # | 场景 | 预期 |
|---|------|------|
| 1 | 首次启动，无配置文件 | 自动创建空配置，无崩溃 |
| 2 | 添加厂商 + 保存 API Key | Key 存入 Keyring，SQLite 写入成功 |
| 3 | 编辑厂商，Key 留空 | Keyring 中 Key 不被覆盖 |
| 4 | 删除厂商 | Keyring 和 DB 同时清理 |
| 5 | 绑定工具 + 厂商 + 模型 | 配置文件正确写入目标路径 |
| 6 | 切换绑定 | 旧配置被覆盖，备份文件生成 |
| 7 | Keyring 损坏/不可用 | 优雅降级提示，不崩溃 |
| 8 | 配置文件被外部修改 | 检测 mtime 冲突，提示用户 |
| 9 | MiniMax Code 桌面版未安装 | 应用列表显示"未安装"，启动按钮禁用 |
| 10 | 连续快速切换绑定 | 无数据竞争，最后一次绑定有效 |

---

## 6. 预计工时

| 阶段 | 内容 | 预估（人天） |
|------|------|------------|
| 0 | 基础工程准备（错误模型 + 原子IO + 路径工具 + 模块结构） | 0.5 |
| 1 | P0 阻塞 Bug 修复（安全 + 异步锁 + 并发 + 双轨配置） | 2 |
| 2 | P1 严重 Bug 修复（事务 + 锁范围 + 回滚 + 竞态） | 2 |
| 3 | 配置写入层解耦（策略模式 + WriterRegistry + 14个适配器迁移） | 2.5 |
| 4 | 前端工程化（页面拆分 + React Query + 统一错误处理 + 状态修复） | 2 |
| 5 | P2 修复 + 测试覆盖（单元测试 + 集成测试 + 前端测试） | 2.5 |
| 6 | 文档与收尾 | 0.5 |
| **总计** | | **12 人天** |

> **注**：这是一个人全职开发的预估。如果使用子代理并行执行（例如阶段 1 中 4 个 P0 Bug 可以同时修复），实际日历时间可压缩到 **5-6 天**。

---

## 附录 A：文件变更清单

### 新增文件

```
src/common/
  mod.rs
  errors.rs
  atomic_io.rs
  path_util.rs
  time_util.rs

src/config_writer/
  mod.rs
  writer_trait.rs
  writer_registry.rs
  writer_claude.rs
  writer_minimax.rs
  writer_tool_configs.rs
  writer_agent.rs

src/repository/
  mod.rs
  provider_repo.rs
  tool_repo.rs
  binding_repo.rs
  model_repo.rs

src/services/
  mod.rs
  provider_service.rs
  binding_service.rs
  detection_service.rs

src/handlers/
  mod.rs
  provider_cmds.rs
  tool_cmds.rs
  binding_cmds.rs
  vendor_cmds.rs  // 旧命令兼容
  chat_cmds.rs
  install_cmds.rs

frontend/src/hooks/
  useProviders.ts
  useModels.ts
  useTools.ts
  useBindings.ts

frontend/src/pages/
  ModelCenterPage.tsx
  ApplicationStudioPage.tsx
  RepairPage.tsx
  PlaceholderPage.tsx

frontend/src/components/common/
  Modal.tsx
  Tag.tsx
  SummaryCard.tsx
  Field.tsx

frontend/src/components/studio/
  StudioToolCard.tsx
  StudioToolSection.tsx
  StudioEmpty.tsx
```

### 修改文件

```
src/lib.rs          → 模块注册重排
src/main.rs         → 不变（或微调）
src/db.rs           → 移除迁移种子重复逻辑，使用 repository 模式
src/tool_configs.rs  → 移入 config_writer 后删除
src/claude_config.rs → 移入 config_writer 后删除
src/minimax_config.rs → 移入 config_writer 后删除
src/agent_adapters.rs  → 移入 config_writer 后删除
src/commands.rs     → 按 handler 拆分后删除

frontend/src/App.tsx → 减肥为布局 + 路由
frontend/src/api.ts  → 添加错误码类型定义

src-tauri/Cargo.toml → 添加 windows-acl / 其他依赖
```

### 删除文件

```
src/tool_configs.rs     ← 迁移到 config_writer
src/claude_config.rs    ← 迁移到 config_writer
src/minimax_config.rs   ← 迁移到 config_writer
src/agent_adapters.rs   ← 迁移到 config_writer
src/commands.rs         ← 拆分为 handlers/
```
