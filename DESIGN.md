# MiniMax Code Vendor Switcher - 设计文档

**日期**：2026-06-05
**状态**：待实现
**作者**：Brainstorming 协作产出

---

## 1. 项目概述与目标

**项目名称**：`MiniMax Code Vendor Switcher`（暂定名，可改）

**核心目标**：一个轻量级跨平台桌面工具，让用户无需修改环境变量或编辑配置文件，就能可视化地在多个 LLM 厂商之间切换，切换后启动 `MiniMax Code`（Claude Code CLI）即可使用所选厂商的模型。

**核心使用流程**：
1. 用户首次启动 → 选择"添加厂商"或"使用预设"
2. 输入厂商名称、API Base URL、API Key、模型名
3. 点击"应用此厂商" → 工具写入 `~/.claude/settings.json` 或对应配置文件
4. 工具自动启动 `MiniMax Code`（或提示用户手动启动）
5. 后续使用：在工具内点选"当前厂商"下拉框 → 一键切换

**关键设计原则**：
- **不存储用户的 MiniMax Code 账号**（MiniMax Code CLI 自身的认证流程不改变，工具只负责切换它连接的后端 API 端点）
- **完全离线本地运行**（除 API 调用外不联网）
- **不修改 MiniMax Code 安装文件**（仅修改用户级配置文件）

**对应 MiniMax Code 的机制**：通过环境变量 `ANTHROPIC_BASE_URL` 和 `ANTHROPIC_AUTH_TOKEN`（或 `ANTHROPIC_API_KEY`）即可重定向到任意 OpenAI 兼容端点。

---

## 2. 技术架构

### 2.1 整体架构

```
┌─────────────────────────────────────────────┐
│   前端层 (WebView - React + TypeScript)     │
│   - 厂商列表、添加/编辑表单、切换按钮       │
│   - 当前激活厂商高亮显示                    │
└──────────────────┬──────────────────────────┘
                   │ Tauri IPC (invoke/listen)
┌──────────────────▼──────────────────────────┐
│   Rust 后端层 (Tauri Commands)              │
│   - vendor_crud: 增删改查厂商               │
│   - apply_vendor: 写入配置文件              │
│   - launch_claude: 启动 MiniMax Code        │
│   - get_current: 读取当前激活厂商           │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│   存储层                                     │
│   - SQLite: 厂商元数据 (id/name/api_base)   │
│   - 系统 Keyring: API Key 加密存储          │
│   - 配置文件: ~/.claude/settings.json       │
└─────────────────────────────────────────────┘
```

### 2.2 存储方案

- **SQLite**：多厂商切换需要事务一致性；便于后续扩展（用量统计、配置导入导出）
- **Keyring**：API Key 加密存储；跨平台一致（Windows Credential Manager / macOS Keychain / Linux Secret Service）

### 2.3 技术栈

**前端**：React 18 + TypeScript + Vite + Tailwind CSS + shadcn/ui

**后端模块**（Rust）：

| 模块 | 职责 |
|------|------|
| `db.rs` | SQLite 连接池、迁移、CRUD |
| `vendor.rs` | 厂商模型定义、预设加载 |
| `keyring_store.rs` | Keyring 读写封装 |
| `claude_config.rs` | 读写 `~/.claude/settings.json` |
| `launcher.rs` | 跨进程启动 MiniMax Code CLI |
| `commands.rs` | Tauri Command 入口（前端调用） |

---

## 3. 数据模型与配置流程

### 3.1 核心数据模型

```rust
// 厂商预设（内置在代码中，不可编辑）
struct VendorPreset {
    id: &'static str,           // "deepseek"
    name: &'static str,         // "DeepSeek"
    api_base: &'static str,     // "https://api.deepseek.com/v1"
    default_model: &'static str,// "deepseek-chat"
    env_template: EnvTemplate,
}

// 用户厂商实例（存储在 SQLite）
struct VendorInstance {
    id: String,                 // UUID
    preset_id: Option<String>,  // 关联预设（自定义则为 None）
    name: String,               // 显示名
    api_base: String,
    model: String,
    keyring_key: String,        // Keyring 中 API Key 的标识
    created_at: i64,
    updated_at: i64,
}

enum EnvTemplate {
    AnthropicCompat,  // ANTHROPIC_BASE_URL + ANTHROPIC_AUTH_TOKEN
    OpenAICompat,     // OpenAI 兼容端点（需转换层）
}
```

### 3.2 配置文件

切换厂商时，工具修改 `~/.claude/settings.json` 的 `env` 字段：

```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.deepseek.com",
    "ANTHROPIC_AUTH_TOKEN": "<从Keyring读取注入>",
    "ANTHROPIC_MODEL": "deepseek-chat"
  }
}
```

### 3.3 切换流程

1. 用户点击"应用 DeepSeek"
2. 前端 `invoke('apply_vendor', { vendorId: 'xxx' })`
3. Rust 后端：
   a. 从 SQLite 读取厂商实例
   b. 从 Keyring 取出对应 API Key
   c. 读取现有 `~/.claude/settings.json`
   d. 合并 `env` 字段，保留其他用户配置
   e. 原子写回（先写临时文件再 rename）
   f. 返回成功
4. 前端显示"已切换到 DeepSeek ✓"
5. 用户启动 MiniMax Code（工具可一键启动或在外部启动）

### 3.4 安全设计

- 切换时 API Key 只在内存中出现一瞬，写入配置文件后不保留副本
- 配置文件权限设为 `0600`（仅当前用户可读）
- 日志中永不打印 API Key 完整值
- 每次切换前自动备份到 `~/.claude/backups/`，文件名带时间戳

---

## 4. UI 设计、错误处理与测试

### 4.1 主界面布局

```
┌─────────────────────────────────────────────────┐
│  ⚡ MiniMax Code Vendor Switcher        [─][□][×]│
├─────────────────────────────────────────────────┤
│  当前激活：[DeepSeek ▼]   ● 状态：已连接         │
│  [🚀 启动 MiniMax Code]   [⚙ 设置]              │
├─────────────────────────────────────────────────┤
│  厂商列表                          [+ 添加厂商]  │
│  ┌───────────────────────────────────────────┐ │
│  │ ● DeepSeek                                │ │
│  │   https://api.deepseek.com                │ │
│  │   模型: deepseek-chat        [应用][编辑][删除] │
│  ├───────────────────────────────────────────┤ │
│  │ ○ Kimi (月之暗面)                          │ │
│  │   ...                          [应用][编辑][删除] │
│  └───────────────────────────────────────────┘ │
├─────────────────────────────────────────────────┤
│  v0.1.0 · 本地工具 · 不上传任何数据            │
└─────────────────────────────────────────────────┘
```

**设计要点**：
- 单窗口、无侧边栏（极简风格）
- 当前激活厂商用绿色圆点 ● 标记
- 添加厂商对话框：4 个字段（名称、API Base、API Key、模型名）
- 内置预设点击"+"时自动填入默认值，用户只需填 API Key

### 4.2 内置厂商预设

| 厂商 | API Base | 默认模型 |
|------|----------|----------|
| MiniMax（默认） | `https://api.MiniMax.com` | `MiniMax-M3` |
| DeepSeek | `https://api.deepseek.com` | `deepseek-chat` |
| Kimi（月之暗面） | `https://api.moonshot.cn/v1` | `moonshot-v1-128k` |
| 智谱 GLM | `https://open.bigmodel.cn/api/paas/v4` | `glm-4-plus` |
| Qwen（通义千问） | `https://dashscope.aliyuncs.com/compatible-mode/v1` | `qwen-plus` |

**自定义支持**：用户可手动添加任何 OpenAI 兼容 API 端点。

### 4.3 错误处理

| 错误场景 | 处理方式 |
|---------|---------|
| 配置文件不存在 | 自动创建空 JSON 后写入 |
| 配置文件被外部修改 | 读取时检测 mtime，提示用户确认是否覆盖 |
| Keyring 访问失败 | 降级到加密文件存储（提示用户） |
| 厂商 API 调用失败 | 工具内显示错误，但不阻塞切换 |
| MiniMax Code 未安装 | 提示"请先安装 MiniMax Code CLI"并附安装指引 |
| SQLite 损坏 | 自动从备份恢复，UI 红色提示 |

### 4.4 测试方案

**单元测试**（Rust 后端，cargo test）：
- `db.rs`：CRUD、迁移、并发读写
- `claude_config.rs`：JSON 合并、原子写入、权限设置
- `keyring_store.rs`：mock 测试读写
- `launcher.rs`：路径解析、参数拼接

**集成测试**：
- 端到端：创建厂商 → 切换 → 验证 `settings.json` 内容
- 切换冲突：连续两次切换，验证备份正确生成

**手动验证清单**（首次发布前）：
1. 在 DeepSeek/Kimi/智谱/Qwen 各创建一个厂商
2. 依次切换并启动 MiniMax Code
3. 验证对话功能正常
4. 测试自定义 OpenAI 兼容端点
5. 卸载/重装 MiniMax Code 后工具仍正常工作

### 4.5 MVP 范围外（明确不做）

- 多语言、暗色模式切换（先用浅色）
- 实时调用测试
- 用量统计
- 配置导入/导出（v0.2 考虑）

---

## 5. 下一步

进入实现阶段时：
1. 使用 `superpowers:using-git-worktrees` 创建隔离工作区
2. 使用 `superpowers:writing-plans` 创建详细实现计划
3. 按 TDD 流程逐模块实现（先 Rust 后端，再前端 UI）
