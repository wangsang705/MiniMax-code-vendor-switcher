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
  'minimax-code-cli',
  'minimax-code-desktop',
  'claude-code-cli',
  'codex-cli',
  'codex-desktop',
  'opencode-cli',
  'qwen-code-cli',
  'aider-cli',
  'kimi-cli',
  'claude-desktop',
  'grok-build',
  'openclaw',
  'hermes-agent',
  'nanobot',
  // NOTE: gemini-desktop 暂缺 ConfigWriter 实现，待补全后取消注释
]);
