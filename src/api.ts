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
