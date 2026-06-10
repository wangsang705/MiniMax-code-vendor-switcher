export type SectionId =
  | 'news'
  | 'featured'
  | 'courses'
  | 'models'
  | 'apps'
  | 'local-models'
  | 'repair'
  | 'feedback';

export type AppCategory = 'all' | 'desktop' | 'agents' | 'ide' | 'cli' | 'tools';

export const CUSTOM_PROVIDER_PRESET = '__custom__';

export const ANTHROPIC_PRESET_IDS = new Set(['minimax', 'deepseek']);

export const SUPPORTED_BINDING_TOOL_IDS = new Set([
  // CLI 编程工具
  'minimax-code-cli',
  'minimax-code-desktop',
  'claude-code-cli',
  'claude-desktop',
  'codex-cli',
  'codex-desktop',
  'opencode-cli',
  'qwen-code-cli',
  'aider-cli',
  'kimi-cli',
  'grok-build',
  'coffee-cli',
  // IDE 桌面端
  'cursor-desktop',
  'gemini-desktop',
  'windsurf-desktop',
  'trae-desktop',
  'zed-desktop',
  // Agents
  'openclaw',
  'hermes-agent',
  'nanobot',
]);
