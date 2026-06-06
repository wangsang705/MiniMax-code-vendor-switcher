import { invoke } from '@tauri-apps/api/core';

// ===== 旧接口（向后兼容） =====

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

// ===== 第二阶段新接口 =====

export interface Tool {
  id: string;
  name: string;
  category: string;     // 'cli' | 'desktop' | 'agent'
  config_path: string | null;
  config_format: string; // 'json' | 'yaml' | 'json5' | 'env'
  launch_command: string | null;
  launch_path: string | null;
  detection_path_cmds: string;
  detection_files: string;
  created_at: number;
  updated_at: number;
}

export interface Provider {
  id: string;
  name: string;
  api_base: string;
  anthropic_mode: boolean;
  created_at: number;
  updated_at: number;
}

export interface Model {
  id: string;
  provider_id: string;
  name: string;
  model_id: string;
  context_length: number;
  max_output: number;
  supports_attachment: boolean;
  supports_reasoning: boolean;
  supports_tool_call: boolean;
  supports_vision: boolean;
  created_at: number;
  updated_at: number;
}

export interface DetectionResult {
  tool_id: string;
  tool_name: string;
  installed: boolean;
  install_type: string;  // 'cli' | 'desktop' | 'both' | 'none'
  versions: string[];
}

export interface ToolBinding {
  id: string;
  tool_id: string;
  provider_id: string;
  model_id: string;
  keyring_key: string | null;
  is_active: boolean;
  created_at: number;
  updated_at: number;
}

// ===== API 调用 =====

export const api = {
  // -- 旧命令 --
  listVendors: () => invoke<VendorInstance[]>('list_vendors'),
  listPresets: () => invoke<VendorPreset[]>('list_presets'),
  createVendor: (input: { preset_id: string | null; name: string; api_base: string; model: string; api_key: string }) =>
    invoke<VendorInstance>('create_vendor', { input }),
  updateVendor: (input: { id: string; name: string; api_base: string; model: string; api_key?: string }) =>
    invoke<VendorInstance>('update_vendor', { input }),
  deleteVendor: (id: string) => invoke<void>('delete_vendor', { id }),
  applyVendor: (id: string) => invoke<void>('apply_vendor', { id }),
  getActiveVendor: () => invoke<string | null>('get_active_vendor'),
  launchClaude: () => invoke<number>('launch_claude_cmd'),
  isClaudeInstalled: () => invoke<boolean>('is_claude_installed'),

  // -- 新命令 --
  detectInstalledTools: () => invoke<DetectionResult[]>('detect_installed_tools'),
  listTools: () => invoke<Tool[]>('list_tools'),
  listProviders: () => invoke<Provider[]>('list_providers'),
  listModels: () => invoke<Model[]>('list_models'),
  createProvider: (input: { id: string; name: string; api_base: string; anthropic_mode: boolean }) =>
    invoke<Provider>('create_provider', { input }),
  deleteProvider: (id: string) => invoke<void>('delete_provider', { id }),
  applyBinding: (tool_id: string, provider_id: string, model_id: string, api_key: string) =>
    invoke<void>('apply_binding', { toolId: tool_id, providerId: provider_id, modelId: model_id, apiKey: api_key }),
  launchTool: (tool_id: string) => invoke<number>('launch_tool', { toolId: tool_id }),

  // -- AI 对话 --
  chatSend: (input: {
    messages: { role: string; content: string }[];
    api_base: string;
    api_key: string;
    model: string;
    anthropic_mode: boolean;
  }) => invoke<{ content: string; model: string }>('chat_send', { input }),
};
