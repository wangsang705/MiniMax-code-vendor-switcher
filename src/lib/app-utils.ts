import type { DetectionResult, Model, Provider, Tool } from '../api';
import type { AppCategory, SectionId } from './app-types';

export function slugifyProviderId(input: string) {
  const slug = input
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
  if (!slug) return `provider-${Date.now()}`;
  return `${slug}-${Date.now().toString(36).slice(-4)}`;
}

export function duplicateProviderNames(providers: Provider[]) {
  return new Set(
    Object.entries(
      providers.reduce<Record<string, number>>((acc, provider) => {
        const key = provider.name.trim().toLowerCase();
        acc[key] = (acc[key] || 0) + 1;
        return acc;
      }, {})
    )
      .filter(([, count]) => count > 1)
      .map(([name]) => name)
  );
}

export function providerDisplayName(provider: Provider, duplicateNames: Set<string>) {
  if (duplicateNames.has(provider.name.toLowerCase())) {
    return `${provider.name} · ${provider.id}`;
  }
  return provider.name;
}

export function providerTone(provider: Provider) {
  if (provider.id.includes('deepseek') || provider.name.toLowerCase().includes('deepseek')) {
    return 'from-sky-50 to-blue-50 border-sky-200';
  }
  if (provider.id.includes('minimax') || provider.name.toLowerCase().includes('minimax')) {
    return 'from-rose-50 to-orange-50 border-rose-200';
  }
  if (provider.id.includes('kimi')) {
    return 'from-amber-50 to-orange-50 border-amber-200';
  }
  if (provider.id.includes('zhipu')) {
    return 'from-neutral-100 to-stone-100 border-stone-200';
  }
  if (provider.id.includes('qwen')) {
    return 'from-violet-50 to-indigo-50 border-violet-200';
  }
  return 'from-slate-50 to-white border-slate-200';
}

export function toolCategory(tool: Tool): AppCategory {
  if (tool.category === 'agent') return 'agents';
  if (tool.category === 'desktop') return 'desktop';
  if (
    tool.id.includes('codex') ||
    tool.id.includes('qwen') ||
    tool.id.includes('opencode') ||
    tool.id.includes('kimi') ||
    tool.id.includes('claude') ||
    tool.id.includes('minimax')
  ) {
    return 'cli';
  }
  return 'tools';
}

export interface ToolDisplayMeta {
  title: string;
  icon: string;
  category: AppCategory;
  installLabel?: string;
}

export function toolDisplayMeta(tool: Tool): ToolDisplayMeta {
  const lookup: Record<string, ToolDisplayMeta> = {
    'claude-code-cli': { title: 'Claude Code CLI', icon: '✳', category: 'cli' },
    'minimax-code-desktop': { title: 'MiniMax 桌面端', icon: 'M', category: 'desktop' },
    'minimax-code-cli': { title: 'MiniMax Code CLI', icon: 'M', category: 'cli' },
    'codex-cli': { title: 'Codex CLI', icon: '☻', category: 'cli' },
    'codex-desktop': { title: 'Codex 桌面端', icon: '☻', category: 'desktop' },
    'claude-desktop': { title: 'Claude 桌面端', icon: '✳', category: 'desktop' },
    'gemini-desktop': { title: 'Gemini 桌面端', icon: 'G', category: 'desktop' },
    'cursor-desktop': { title: 'Cursor', icon: 'C', category: 'desktop' },
    'windsurf-desktop': { title: 'Windsurf', icon: 'W', category: 'desktop' },
    'trae-desktop': { title: 'Trae', icon: 'T', category: 'desktop' },
    'zed-desktop': { title: 'Zed', icon: 'Z', category: 'desktop' },
    'coffee-cli': { title: 'Coffee CLI', icon: '☕', category: 'cli' },
    'aider-cli': { title: 'Aider', icon: 'A', category: 'cli' },
    'opencode-cli': { title: 'OpenCode', icon: '▣', category: 'cli' },
    'qwen-code-cli': { title: 'Qwen CLI', icon: '千', category: 'cli' },
    'kimi-cli': { title: 'Kimi CLI', icon: 'K', category: 'cli' },
    'grok-build': { title: 'Grok Build', icon: 'G', category: 'cli' },
    openclaw: { title: 'OpenClaw', icon: '•', category: 'agents' },
    'hermes-agent': { title: 'Hermes Agent', icon: '⚕', category: 'agents', installLabel: 'AI 自动安装' },
    nanobot: { title: 'NanoBot', icon: '猫', category: 'agents', installLabel: 'AI 自动安装' },
  };

  const fallback: ToolDisplayMeta = {
    title: tool.name,
    icon: tool.name.slice(0, 1).toUpperCase(),
    category: toolCategory(tool),
  };

  return lookup[tool.id] ?? fallback;
}

export function trimPath(value: string | null | undefined) {
  if (!value) return '-';
  if (value.length <= 26) return value;
  return `${value.slice(0, 23)}...`;
}

export function toolStatus(detection: DetectionResult | undefined) {
  return detection?.installed ? '已安装' : '未安装';
}

export function appCategoryLabel(category: AppCategory) {
  switch (category) {
    case 'desktop':
      return '桌面端';
    case 'agents':
      return 'Agents';
    case 'ide':
      return 'IDE';
    case 'cli':
      return 'CLI Code';
    case 'tools':
      return '工具';
    default:
      return '全部';
  }
}

export function formatContextLength(value: number) {
  if (value >= 1000) return `${Math.round(value / 1000)}K`;
  return `${value}`;
}

export function formatModelStatus(provider: Provider, model: Model) {
  if (!provider.has_api_key) return '待补全 Key';
  if (model.supports_tool_call && model.supports_reasoning) return '就绪';
  return '基础可用';
}

export function sectionMeta(section: SectionId) {
  switch (section) {
    case 'models':
      return { title: '模型中心', accent: 'ROSTER', subtitle: '管理模型厂商、模型条目与中转配置。' };
    case 'apps':
      return { title: '应用管理', accent: 'STUDIO', subtitle: '为桌面端、Agents 与 CLI 工具分配模型。' };
    case 'local-models':
      return { title: '本地大模型', accent: 'LOCAL', subtitle: '后续用于接入本地推理服务与离线模型。' };
    case 'repair':
      return { title: '安装与修复', accent: 'CARE', subtitle: '集中处理安装、环境检测和恢复操作。' };
    case 'news':
      return { title: 'AI 资讯', accent: 'NEWS', subtitle: '预留资讯流与公告位。' };
    case 'featured':
      return { title: '明星项目', accent: 'PICKS', subtitle: '预留精选项目与推荐场景。' };
    case 'courses':
      return { title: 'AI 公开课', accent: 'GUIDE', subtitle: '预留教程、文档与上手指南。' };
    case 'feedback':
      return { title: '问题反馈', accent: 'VOICE', subtitle: '预留反馈、日志与问题上报入口。' };
    default:
      return { title: '观景', accent: 'VIEW', subtitle: '统一查看模型、工具和状态。' };
  }
}
