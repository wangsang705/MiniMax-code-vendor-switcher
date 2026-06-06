# AI Toolkit Hub - 第二阶段设计文档

**日期：** 2026-06-06
**状态：** 实施中
**工作区：** `.worktrees/feat-ai-toolkit-hub`

## 架构概览

```
┌─────────────────────────────────────────────────────────┐
│                    UI 层 (React + Tailwind)              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │
│  │ 工具面板   │ │ 厂商配置   │ │ 模型中心   │ │ AI 助手   │   │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘   │
└───────┼─────────────┼────────────┼────────────┼─────────┘
        │             │            │            │
┌───────▼─────────────▼────────────▼────────────▼─────────┐
│               Rust 后端层 (Tauri Commands)                │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │ 检测引擎      │  │ 工具适配器    │  │ LLM 客户端     │  │
│  │ · PATH 扫描   │  │ · Claude    │  │ · 流式对话     │  │
│  │ · 安装目录扫描 │  │ · MiniMax   │  │ · 模型调用     │  │
│  │ · 注册表检测   │  │ · Codex     │  │               │  │
│  │               │  │ · Qwen      │  │               │  │
│  │               │  │ · OpenClaw  │  │               │  │
│  │               │  │ · Hermes    │  │               │  │
│  │               │  │ · Nanobot   │  │               │  │
│  └──────────────┘  └──────────────┘  └───────────────┘  │
│                                                         │
│  ┌──────────────────────────────────────────────────┐   │
│  │  存储层：SQLite (tools + providers + bindings)   │   │
│  │          + Keyring (API Keys)                    │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## 数据模型

```sql
-- 工具平台
tools (id, name, category, config_path, config_format,
       launch_command, launch_path, env_keys_json,
       detection_path_cmds_json, detection_files_json,
       created_at, updated_at)

-- 厂商/供应商
providers (id, name, api_base, preset_id, anthropic_mode,
           created_at, updated_at)

-- 模型  
models (id, provider_id, name, model_id, context_length,
        max_output, supports_attachment, supports_reasoning,
        supports_tool_call, supports_vision, created_at, updated_at)

-- 工具-厂商绑定
tool_bindings (id, tool_id, provider_id, model_id,
               api_key_ring, is_active, created_at, updated_at)
```

## 实施步骤

| Step | 内容 | 状态 |
|------|------|------|
| 1 | 重命名项目 + 新数据库 schema | ⏳ |
| 2 | 检测引擎 + 工具适配器 | 📝 |
| 3 | UI 重构（三栏布局 + Tab 面板） | 📝 |
| 4 | AI 对话窗口 + 流式 API 调用 | 📝 |
| 5 | Agents 支持 + 一键安装 | 📝 |
