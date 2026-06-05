# MiniMax Code Vendor Switcher

> 一键在 DeepSeek / Kimi / 智谱 GLM / Qwen 等多家 LLM 厂商之间切换，切换后启动 MiniMax Code 即可使用所选厂商的模型。

一个轻量级跨平台桌面工具，基于 **Tauri 2.0 + Rust + React**，通过修改 `~/.claude/settings.json` 的环境变量段来重定向 MiniMax Code CLI 的 API 端点。

## 功能

- **5 个内置厂商预设**：MiniMax、DeepSeek、Kimi（月之暗面）、智谱 GLM、Qwen（通义千问）
- **任意 OpenAI 兼容自定义端点**：在添加厂商时选"自定义 OpenAI 兼容端点"
- **一键切换**：写入 `~/.claude/settings.json` 的 `env` 字段，原文件自动备份
- **API Key 系统 Keyring 加密存储**：Windows Credential Manager / macOS Keychain / Linux Secret Service
- **原子写**：临时文件 + rename，避免半写状态
- **一键启动 MiniMax Code CLI**

## 快速开始

### 开发模式

```bash
npm install
npm run tauri dev
```

桌面窗口启动后可看到空列表，点击「+ 添加厂商」开始。

### 构建发布版

```bash
npm run tauri build
```

产物位置：

| 平台 | 路径 |
|------|------|
| Windows | `src-tauri/target/release/bundle/msi/MiniMax Code Vendor Switcher_0.1.0_x64_en-US.msi` |
| Windows (exe) | `src-tauri/target/release/bundle/nsis/MiniMax Code Vendor Switcher_0.1.0_x64-setup.exe` |
| macOS | `src-tauri/target/release/bundle/dmg/MiniMax Code Vendor Switcher_0.1.0_aarch64.dmg` |
| Linux | `src-tauri/target/release/bundle/{deb,appimage}/...` |

## 工作原理

工具本身**不存储** MiniMax Code 账号，也不修改 MiniMax Code 的安装文件。它只修改 `~/.claude/settings.json` 中的 `env` 段：

```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.deepseek.com",
    "ANTHROPIC_AUTH_TOKEN": "sk-...",
    "ANTHROPIC_MODEL": "deepseek-chat"
  }
}
```

启动 MiniMax Code 时，CLI 读取这个 env 配置作为后端 API 地址。

### 数据存储位置

| 数据 | 路径 |
|------|------|
| 厂商元数据（SQLite） | `%APPDATA%\com.MiniMax.vendor-switcher\vendors.db`（Windows）/ `~/Library/Application Support/com.MiniMax.vendor-switcher/vendors.db`（macOS）/ `~/.local/share/com.MiniMax.vendor-switcher/vendors.db`（Linux） |
| API Key | 系统 Keyring（service: `MiniMax-vendor-switcher`，account: `vendor:<uuid>`） |
| MiniMax Code 配置 | `~/.claude/settings.json`（原文件自动备份到 `~/.claude/backups/settings.<ms>.json`） |

## 端到端手动验证清单

实施完成后（每次升级或换机器时跑一遍）：

### 1. 启动应用

```bash
npm run tauri dev
```

预期：桌面窗口打开，看到空列表和「还没有厂商，点击右上角"添加厂商"开始。」

### 2. 添加一个测试厂商

点击「+ 添加厂商」→ 选「自定义 OpenAI 兼容端点」→ 填：

- 名称：`Test DeepSeek`
- API Base URL：`https://api.deepseek.com`
- 模型：`deepseek-chat`
- API Key：`sk-test-fake-key-do-not-use`

→ 保存

预期：列表显示 "Test DeepSeek"。

### 3. 应用并验证 settings.json

点击「应用」→ 预期：

- 该项前出现绿色圆点
- 验证 settings.json（Windows PowerShell）：

  ```powershell
  Get-Content ~/.claude/settings.json
  ```

  预期输出类似：

  ```json
  {
    "env": {
      "ANTHROPIC_BASE_URL": "https://api.deepseek.com",
      "ANTHROPIC_AUTH_TOKEN": "sk-test-fake-key-do-not-use",
      "ANTHROPIC_MODEL": "deepseek-chat"
    }
  }
  ```

### 4. 验证备份目录

```bash
ls ~/.claude/backups/
```

预期：至少有一个 `settings.<timestamp>.json` 文件。

### 5. 验证 Keyring

**Windows**：打开「凭据管理器」→ 「Windows 凭据」→ 搜索 `MiniMax-vendor-switcher`
**macOS**：打开「钥匙串访问」→ 搜索 `MiniMax-vendor-switcher`
**Linux**：使用 `secret-tool search service MiniMax-vendor-switcher`（需安装 libsecret-tools）

预期：看到一条以 `vendor:<uuid>` 为用户名的凭据。

### 6. 测试预设厂商

添加几个真实厂商（DeepSeek/Kimi/智谱/Qwen），填入真实 API Key，依次切换并启动 MiniMax Code 验证对话功能。

### 7. 删除厂商，确认 Keyring 清理

回到应用 → 点击「删除」→ 确认。

预期：列表为空，凭据管理器中该条目消失。

### 8. 测试 MiniMax Code 启动按钮

- 状态 1：未安装 MiniMax Code → 按钮禁用 + 黄色提示
- 状态 2：已安装 → 按钮可点击，点击后启动 CLI

## 安全

- **API Key 永远不离开系统 Keyring + 切换瞬间的内存**
- **配置文件权限 0600**（Unix）
- **日志不打印完整 API Key**（代码中无 `dbg!`/`println!` 涉及 Key 值）
- **回滚保护**：每次切换前自动备份原 `settings.json`，可手动从 `~/.claude/backups/` 恢复

## 架构

```
┌─────────────────────────────────────────────┐
│   React 19 + TypeScript 前端                │
│   (App.tsx / VendorList / VendorDialog)    │
└──────────────────┬──────────────────────────┘
                   │ Tauri IPC (invoke)
┌──────────────────▼──────────────────────────┐
│   Rust 后端 (src-tauri/src/)                │
│   commands.rs (9 Tauri Commands)            │
│   ├─ db.rs           (SQLite CRUD)          │
│   ├─ keyring_store.rs (Keyring 封装)        │
│   ├─ claude_config.rs (原子写 + 备份)       │
│   ├─ vendor.rs        (5 预设)              │
│   └─ launcher.rs      (which + spawn)       │
└─────────────────────────────────────────────┘
```

## 文档

- [DESIGN.md](DESIGN.md) - 设计文档
- [docs/plans/2026-06-05-MiniMax-code-vendor-switcher-design.md](docs/plans/2026-06-05-MiniMax-code-vendor-switcher-design.md) - 详细设计
- [docs/plans/2026-06-05-MiniMax-code-vendor-switcher-implementation.md](docs/plans/2026-06-05-MiniMax-code-vendor-switcher-implementation.md) - 16 任务实施计划

## 开发与测试

### 运行后端单元测试

```bash
cd src-tauri
cargo test
```

9 个测试覆盖：
- `db` (2)：init + CRUD
- `claude_config` (2)：env 合并 + 备份创建
- `keyring` (1)：读写回环
- `launcher` (2)：路径格式 + 找不到不 panic
- `vendor` (2)：5 个预设存在 + 字段非空

### 类型检查与构建

```bash
npm run build
```

## 故障排查

| 现象 | 可能原因 | 修复 |
|------|---------|------|
| `tauri dev` 启动后立刻退出 | Windows 文件锁残留 | 关闭所有 node.exe / cargo 子进程，删除 `src-tauri/target/.rustc_info.json` 后重试 |
| Keyring 测试失败 | 无 GUI session 或 Credential Manager 不可用 | 在已登录用户的桌面会话下运行 |
| 切换后 MiniMax Code 仍连原厂商 | env 未生效 | 重启 MiniMax Code CLI（子进程启动时锁定 env） |
| 删除厂商后 Keyring 仍有条目 | Keyring 系统故障 | 凭据管理器中手动删除 `MiniMax-vendor-switcher` 下对应条目 |

## 路线图（v0.2+）

- 实时 API 调用测试
- 用量统计
- 暗色模式
- 多语言
- 配置导入/导出
- 备份目录自动轮转（保留最近 N 个）
- 单元测试覆盖 `commands.rs`

## 许可

MIT
