import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  BookOpen,
  Boxes,
  BrainCircuit,
  Cpu,
  GraduationCap,
  MessageSquareText,
  PackageSearch,
  Sparkles,
  Star,
  Wrench,
} from 'lucide-react';
import { api, DetectionResult, Model, Provider, Tool, VendorPreset } from './api';

type SectionId =
  | 'news'
  | 'featured'
  | 'courses'
  | 'models'
  | 'apps'
  | 'local-models'
  | 'repair'
  | 'feedback';

type AppCategory = 'all' | 'desktop' | 'agents' | 'ide' | 'cli' | 'tools';

const CUSTOM_PROVIDER_PRESET = '__custom__';
const ANTHROPIC_PRESET_IDS = new Set(['minimax', 'deepseek']);
const SUPPORTED_BINDING_TOOL_IDS = new Set([
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
]);

function slugifyProviderId(input: string) {
  return input
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

function duplicateProviderNames(providers: Provider[]) {
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

function providerDisplayName(provider: Provider, duplicateNames: Set<string>) {
  if (duplicateNames.has(provider.name.toLowerCase())) {
    return `${provider.name} · ${provider.id}`;
  }
  return provider.name;
}

function providerTone(provider: Provider) {
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

function toolCategory(tool: Tool): AppCategory {
  if (tool.category === 'agent') return 'agents';
  if (tool.category === 'desktop') return 'desktop';
  if (tool.id.includes('codex') || tool.id.includes('qwen') || tool.id.includes('opencode') || tool.id.includes('kimi') || tool.id.includes('claude') || tool.id.includes('minimax')) {
    return 'cli';
  }
  return 'tools';
}

function toolDisplayMeta(tool: Tool) {
  const lookup: Record<string, { title: string; icon: string; category: AppCategory; installLabel?: string }> = {
    'claude-code-cli': { title: 'Claude 桌面端', icon: '✳', category: 'desktop' },
    'minimax-code-desktop': { title: 'MiniMax 桌面端', icon: 'M', category: 'desktop' },
    'codex-cli': { title: 'Codex 桌面端', icon: '☻', category: 'desktop' },
    'aider-cli': { title: 'Aider', icon: 'A', category: 'cli' },
    'openclaw': { title: 'OpenClaw', icon: '•', category: 'agents' },
    'hermes-agent': { title: 'Hermes Agent', icon: '女', category: 'agents', installLabel: 'AI 自动安装' },
    'nanobot': { title: 'NanoBot', icon: '猫', category: 'agents', installLabel: 'AI 自动安装' },
    'opencode-cli': { title: 'OpenCode', icon: '▣', category: 'cli' },
    'qwen-code-cli': { title: 'Qwen CLI', icon: '千', category: 'cli' },
    'kimi-cli': { title: 'Kimi CLI', icon: 'K', category: 'cli' },
    'minimax-code-cli': { title: 'MiniMax Code CLI', icon: 'M', category: 'cli' },
  };

  const fallback = {
    title: tool.name,
    icon: tool.name.slice(0, 1).toUpperCase(),
    category: toolCategory(tool),
  };

  return lookup[tool.id] ?? fallback;
}

function trimPath(value: string | null | undefined) {
  if (!value) return '-';
  if (value.length <= 26) return value;
  return `${value.slice(0, 23)}...`;
}

function toolStatus(detection: DetectionResult | undefined) {
  return detection?.installed ? '已安装' : '未安装';
}

function appCategoryLabel(category: AppCategory) {
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

function formatContextLength(value: number) {
  if (value >= 1000) return `${Math.round(value / 1000)}K`;
  return `${value}`;
}

function formatModelStatus(provider: Provider, model: Model) {
  if (!provider.has_api_key) return '待补全 Key';
  if (model.supports_tool_call && model.supports_reasoning) return '就绪';
  return '基础可用';
}

function sectionMeta(section: SectionId) {
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

const sidebarItems: Array<{ id: SectionId; label: string; icon: React.ComponentType<{ className?: string }> }> = [
  { id: 'news', label: 'AI 资讯', icon: BookOpen },
  { id: 'featured', label: '明星项目', icon: Star },
  { id: 'courses', label: 'AI 公开课', icon: GraduationCap },
  { id: 'models', label: '模型中心', icon: Boxes },
  { id: 'apps', label: '应用管理', icon: Cpu },
  { id: 'local-models', label: '本地大模型', icon: BrainCircuit },
  { id: 'repair', label: '安装与修复', icon: Wrench },
  { id: 'feedback', label: '问题反馈', icon: MessageSquareText },
];

export default function App() {
  const [activeSection, setActiveSection] = useState<SectionId>('models');
  const meta = sectionMeta(activeSection);
  const [compactLayout, setCompactLayout] = useState(false);

  useEffect(() => {
    const updateLayout = () => setCompactLayout(window.innerWidth < 1600);
    updateLayout();
    window.addEventListener('resize', updateLayout);
    return () => window.removeEventListener('resize', updateLayout);
  }, []);

  return (
    <div className="min-h-screen bg-[#f6f1e8] text-slate-900">
      <div className={`grid min-h-screen ${compactLayout ? 'grid-cols-[220px_minmax(0,1fr)]' : 'grid-cols-[280px_minmax(0,1fr)]'}`}>
        <aside className={`border-r border-[#e7ddd0] bg-[#f7f1e7] ${compactLayout ? 'px-6 py-6' : 'px-10 py-8'}`}>
          <div className="mb-10 flex items-center gap-4">
            <div className="grid h-14 w-14 place-items-center rounded-2xl bg-gradient-to-br from-[#ff8b4d] to-[#d8642f] text-white shadow-[0_18px_35px_rgba(216,100,47,0.22)]">
              <Sparkles className="h-7 w-7" />
            </div>
            <div>
              <div className="flex items-end gap-3">
                <h1 className={`${compactLayout ? 'text-3xl' : 'text-4xl'} font-black tracking-tight text-[#2f241c]`}>观景</h1>
                <span className="pb-1 text-sm tracking-[0.35em] text-[#cc6a43]">VISTA</span>
              </div>
              <p className="mt-1 text-sm text-[#8f7c6a]">中文模型与工具编排控制台</p>
            </div>
          </div>

          <nav className="space-y-2">
            {sidebarItems.map(item => {
              const Icon = item.icon;
              const active = item.id === activeSection;
              return (
                <button
                  key={item.id}
                  onClick={() => setActiveSection(item.id)}
                  className={`flex w-full items-center gap-4 rounded-2xl px-5 py-4 text-left transition-all ${
                    active
                      ? 'bg-[#ddd3c6] text-[#2f241c] shadow-[0_12px_30px_rgba(80,60,40,0.08)]'
                      : 'text-[#5f554d] hover:bg-[#ede5da]'
                  }`}
                >
                  <Icon className="h-6 w-6" />
                  <span className={`${compactLayout ? 'text-lg' : 'text-xl'} font-semibold`}>{item.label}</span>
                </button>
              );
            })}
          </nav>

          <div className="mt-auto pt-12 text-sm text-[#8f7c6a]">
            <div className="rounded-2xl border border-[#eadfce] bg-[#fbf7f0] px-5 py-4">
              <div className="font-semibold text-[#5a4b3d]">本地大模型</div>
              <div className="mt-1 text-[#9e8d7d]">离线</div>
            </div>
          </div>
        </aside>

        <main className={`${compactLayout ? 'px-6 py-6' : 'px-10 py-8'}`}>
          <header className="mb-8 flex items-start justify-between">
            <div>
              <div className="flex items-end gap-4">
                <h2 className={`${compactLayout ? 'text-4xl' : 'text-5xl'} font-black tracking-tight text-[#2d241b]`}>{meta.title}</h2>
                <span className="pb-2 text-sm font-semibold tracking-[0.35em] text-[#c96f46]">{meta.accent}</span>
              </div>
              <p className="mt-3 text-base text-[#8c7a69]">{meta.subtitle}</p>
            </div>
            <div className="rounded-full bg-[#efe7db] px-4 py-2 text-sm font-semibold text-[#786a5d]">中文界面</div>
          </header>

          {activeSection === 'models' && <ModelCenterPage />}
          {activeSection === 'apps' && <ApplicationStudioPage />}
          {activeSection === 'local-models' && <PlaceholderPanel title="本地大模型" body="这里下一步可以继续接 Ollama、LM Studio、vLLM 等本地推理服务。" />}
          {activeSection === 'repair' && <PlaceholderPanel title="安装与修复" body="这里适合集中处理缺失环境、诊断日志、重新检测与一键修复。" />}
          {activeSection === 'news' && <PlaceholderPanel title="AI 资讯" body="这里预留给资讯、公告与版本更新提醒。" />}
          {activeSection === 'featured' && <PlaceholderPanel title="明星项目" body="这里预留给推荐项目、模板工作流和热门组合。" />}
          {activeSection === 'courses' && <PlaceholderPanel title="AI 公开课" body="这里预留给教程、接入说明和上手指南。" />}
          {activeSection === 'feedback' && <PlaceholderPanel title="问题反馈" body="这里预留给日志导出、错误反馈和工单入口。" />}
        </main>
      </div>
    </div>
  );
}

function ModelCenterPage() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [models, setModels] = useState<Model[]>([]);
  const [presets, setPresets] = useState<VendorPreset[]>([]);
  const [showAddProvider, setShowAddProvider] = useState(false);
  const [showAddModel, setShowAddModel] = useState(false);
  const [editingProvider, setEditingProvider] = useState<Provider | null>(null);
  const [editingModel, setEditingModel] = useState<Model | null>(null);

  const load = useCallback(() => {
    Promise.all([api.listProviders(), api.listModels(), api.listPresets()])
      .then(([providerList, modelList, presetList]) => {
        setProviders(providerList);
        setModels(modelList);
        setPresets(presetList);
      })
      .catch(console.error);
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const duplicateNames = duplicateProviderNames(providers);
  const providerMap = useMemo(
    () => new Map(providers.map(provider => [provider.id, provider])),
    [providers]
  );

  const modelCards = useMemo(
    () =>
      models
        .map(model => ({ model, provider: providerMap.get(model.provider_id) }))
        .filter((item): item is { model: Model; provider: Provider } => Boolean(item.provider)),
    [models, providerMap]
  );

  const readyProviderCount = providers.filter(provider => provider.has_api_key).length;
  const anthropicReadyCount = providers.filter(provider => provider.anthropic_mode).length;

  return (
    <div className="space-y-6">
      <section className="space-y-6">
        <div className="flex flex-wrap items-center gap-4">
          <button className="rounded-xl border border-[#e3d7c8] bg-[#fbf7ef] px-6 py-3 text-xl font-semibold text-[#5b4d41] transition hover:bg-white">
            测试速度
          </button>
          <button className="rounded-xl bg-[#d8cfbf] px-6 py-3 text-xl font-semibold text-[#3a3128]">
            大模型厂商
          </button>
          <button className="rounded-xl px-6 py-3 text-xl font-semibold text-[#6d6257] transition hover:bg-[#efe6da]">
            模型中转站
          </button>
        </div>

        <div className="grid gap-4 md:grid-cols-3">
          <SummaryCard title="已接入厂商" value={`${providers.length}`} detail={`${readyProviderCount} 个已保存 Key`} />
          <SummaryCard title="模型条目" value={`${models.length}`} detail="用于桌面端、Agents 与 CLI 绑定" />
          <SummaryCard title="Anthropic 兼容" value={`${anthropicReadyCount}`} detail="适合 Claude / MiniMax 一类工具" />
        </div>

        <div className="grid gap-6 md:grid-cols-2 xl:grid-cols-3">
          {modelCards.map(({ model, provider }) => (
            <article
              key={model.id}
              className={`rounded-[28px] border bg-gradient-to-br p-6 shadow-[0_18px_40px_rgba(92,70,44,0.08)] ${providerTone(provider)}`}
            >
              <div className="mb-4 flex items-start justify-between gap-4">
                <div>
                  <div className="text-sm tracking-[0.25em] text-[#9b8a78]">云端</div>
                  <h3 className="mt-3 text-2xl font-black text-[#2f241c]">{providerDisplayName(provider, duplicateNames)}</h3>
                </div>
                <div className="space-x-2 text-sm text-[#9d8f81]">
                  <button onClick={() => setEditingModel(model)} className="hover:text-[#6e5949]">编辑模型</button>
                  <button onClick={() => setEditingProvider(provider)} className="hover:text-[#6e5949]">编辑</button>
                  <button
                    onClick={async () => {
                      if (!confirm(`确定删除厂商“${provider.name}”吗？`)) return;
                      await api.deleteProvider(provider.id);
                      load();
                    }}
                    className="hover:text-[#c35c44]"
                  >
                    删除
                  </button>
                </div>
              </div>

              <div className="space-y-3 text-[#6f6256]">
                <div>模型：<span className="font-semibold text-[#53483d]">{model.model_id}</span></div>
                <div>来源：<span className="font-mono text-[#53483d]">{provider.api_base.replace(/^https?:\/\//, '')}</span></div>
                <div>上下文：<span className="font-semibold text-[#53483d]">{formatContextLength(model.context_length)}</span></div>
              </div>

              <div className="mt-6 flex flex-wrap gap-2">
                <Tag tone={provider.has_api_key ? 'green' : 'rose'}>{formatModelStatus(provider, model)}</Tag>
                <Tag tone="slate">{provider.anthropic_mode ? 'Anthropic' : 'OpenAI'}</Tag>
                {model.supports_reasoning && <Tag tone="amber">推理</Tag>}
                {model.supports_tool_call && <Tag tone="blue">工具调用</Tag>}
                {provider.has_api_key ? <Tag tone="green">已保存 Key</Tag> : <Tag tone="rose">缺少 Key</Tag>}
              </div>
              <div className="mt-5 flex justify-end">
                <button
                  onClick={async () => {
                    if (!confirm(`确定删除模型“${model.name}”吗？`)) return;
                    await api.deleteModel(model.id);
                    load();
                  }}
                  className="text-sm font-semibold text-[#c35c44] hover:text-[#a94a34]"
                >
                  删除模型
                </button>
              </div>
            </article>
          ))}

          <button
            onClick={() => setShowAddModel(true)}
            className="grid min-h-[270px] place-items-center rounded-[28px] border border-dashed border-[#d9cdbd] bg-[#fbf8f1] text-center transition hover:bg-white"
          >
            <div>
              <div className="text-4xl font-black text-[#43382e]">添加模型</div>
              <div className="mt-3 text-2xl text-[#9d8f81]">OpenAI / Anthropic API</div>
            </div>
          </button>
        </div>
      </section>
      <section className="rounded-[28px] border border-[#e3d7c8] bg-[#fbf7ef] p-5 shadow-[0_18px_40px_rgba(92,70,44,0.08)]">
        <div className="mb-4 flex items-center justify-between">
          <h3 className="text-2xl font-black text-[#31261d]">厂商列表</h3>
          <button
            onClick={() => setShowAddProvider(true)}
            className="rounded-xl bg-[#d2cabd] px-4 py-2 text-lg font-semibold text-[#3a3027] transition hover:bg-[#c7bdaf]"
          >
            新增厂商
          </button>
        </div>

        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
          {providers.map(provider => (
            <div key={provider.id} className="rounded-2xl border border-[#eadfce] bg-white px-4 py-4">
              <div className="flex items-start justify-between gap-3">
                <div>
                  <div className="text-xl font-bold text-[#2f241c]">{providerDisplayName(provider, duplicateNames)}</div>
                  <div className="mt-1 text-sm text-[#9a8a79]">{provider.api_base.replace(/^https?:\/\//, '')}</div>
                </div>
                <div className="flex items-center gap-2">
                  <button onClick={() => setEditingProvider(provider)} className="text-sm font-semibold text-[#6e5949] hover:text-[#3f3329]">编辑</button>
                  <button
                    onClick={async () => {
                      if (!confirm(`确定删除厂商“${provider.name}”吗？`)) return;
                      await api.deleteProvider(provider.id);
                      load();
                    }}
                    className="text-sm font-semibold text-[#c35c44] hover:text-[#a94a34]"
                  >
                    删除
                  </button>
                </div>
              </div>
              <div className="mt-3 flex flex-wrap gap-2">
                <Tag tone="slate">{provider.anthropic_mode ? 'Anthropic 兼容' : 'OpenAI 格式'}</Tag>
                <Tag tone={provider.has_api_key ? 'green' : 'rose'}>{provider.has_api_key ? '已保存 Key' : '缺少 Key'}</Tag>
                <Tag tone="amber">{models.filter(model => model.provider_id === provider.id).length} 个模型</Tag>
              </div>
            </div>
          ))}
        </div>
      </section>

      {showAddProvider && (
        <ProviderEditorModal
          presets={presets}
          onClose={() => setShowAddProvider(false)}
          onSaved={() => {
            setShowAddProvider(false);
            load();
          }}
        />
      )}
      {editingProvider && (
        <ProviderEditorModal
          presets={presets}
          provider={editingProvider}
          onClose={() => setEditingProvider(null)}
          onSaved={() => {
            setEditingProvider(null);
            load();
          }}
        />
      )}
      {showAddModel && (
        <ModelEditorModal
          providers={providers}
          onClose={() => setShowAddModel(false)}
          onSaved={() => {
            setShowAddModel(false);
            load();
          }}
        />
      )}
      {editingModel && (
        <ModelEditorModal
          providers={providers}
          model={editingModel}
          onClose={() => setEditingModel(null)}
          onSaved={() => {
            setEditingModel(null);
            load();
          }}
        />
      )}
    </div>
  );
}

function ApplicationStudioPage() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [models, setModels] = useState<Model[]>([]);
  const [tools, setTools] = useState<Tool[]>([]);
  const [detection, setDetection] = useState<DetectionResult[]>([]);
  const [category, setCategory] = useState<AppCategory>('all');
  const [selectedToolId, setSelectedToolId] = useState<string>('');
  const [selectedProviderId, setSelectedProviderId] = useState<string>('');
  const [selectedModelId, setSelectedModelId] = useState<string>('');
  const [binding, setBinding] = useState<{ provider_name: string | null; model_name: string | null } | null>(null);
  const [saving, setSaving] = useState(false);
  const [notice, setNotice] = useState<string>('');

  const load = useCallback(async () => {
    const [providerList, modelList, toolList, detectionList] = await Promise.all([
      api.listProviders(),
      api.listModels(),
      api.listTools(),
      api.detectInstalledTools(),
    ]);
    setProviders(providerList);
    setModels(modelList);
    setTools(toolList);
    setDetection(detectionList);
    if (!selectedToolId) {
      const installed = toolList.find(tool => detectionList.find(item => item.tool_id === tool.id)?.installed);
      setSelectedToolId(installed?.id ?? toolList[0]?.id ?? '');
    }
  }, [selectedToolId]);

  useEffect(() => {
    load().catch(console.error);
  }, [load]);

  useEffect(() => {
    if (!selectedToolId) return;
    api.getToolBinding(selectedToolId).then(result => {
      setBinding(result ? { provider_name: result.provider_name, model_name: result.model_name } : null);
      if (result) {
        setSelectedProviderId(result.provider_id);
        setSelectedModelId(result.model_id);
      }
    }).catch(() => setBinding(null));
  }, [selectedToolId]);

  const selectedTool = tools.find(tool => tool.id === selectedToolId) ?? null;
  const selectedDetection = detection.find(item => item.tool_id === selectedToolId);
  const selectedProvider = providers.find(provider => provider.id === selectedProviderId) ?? null;
  const selectedMeta = selectedTool ? toolDisplayMeta(selectedTool) : null;
  const duplicateNames = duplicateProviderNames(providers);
  const installedCount = detection.filter(item => item.installed).length;
  const configurableCount = tools.filter(tool => SUPPORTED_BINDING_TOOL_IDS.has(tool.id)).length;
  const needsInstallCount = Math.max(tools.length - installedCount, 0);
  const compatibleProviders = useMemo(() => {
    if (!selectedTool) return providers;
    if (selectedTool.id === 'aider-cli') {
      return providers.filter(provider => !provider.anthropic_mode);
    }
    return providers;
  }, [providers, selectedTool]);
  const providerModels = models.filter(model => model.provider_id === selectedProviderId);

  const categories: Array<{ id: AppCategory; label: string }> = [
    { id: 'all', label: '全部' },
    { id: 'desktop', label: '桌面端' },
    { id: 'agents', label: 'Agents' },
    { id: 'ide', label: 'IDE' },
    { id: 'cli', label: 'CLI Code' },
    { id: 'tools', label: '工具' },
  ];

  const visibleTools = tools.filter(tool => {
    const meta = toolDisplayMeta(tool);
    return category === 'all' || meta.category === category;
  });
  const installedTools = visibleTools.filter(tool => detection.find(item => item.tool_id === tool.id)?.installed);
  const availableTools = visibleTools.filter(tool => !detection.find(item => item.tool_id === tool.id)?.installed);

  useEffect(() => {
    if (!selectedTool || !selectedProviderId) return;
    if (selectedTool.id === 'aider-cli') {
      const provider = providers.find(item => item.id === selectedProviderId);
      if (provider?.anthropic_mode) {
        setSelectedProviderId('');
        setSelectedModelId('');
      }
    }
  }, [providers, selectedProviderId, selectedTool]);

  const applyBinding = async () => {
    if (!selectedTool || !selectedProviderId || !selectedModelId) return;
    setSaving(true);
    setNotice('');
    try {
      await api.applyBinding(selectedTool.id, selectedProviderId, selectedModelId);
      const selectedProviderName = providers.find(provider => provider.id === selectedProviderId)?.name ?? null;
      const selectedModelName = models.find(model => model.id === selectedModelId)?.name ?? null;
      setBinding({ provider_name: selectedProviderName, model_name: selectedModelName });
      setNotice(`已将 ${selectedMeta?.title ?? selectedTool.name} 绑定到 ${selectedProviderName ?? '-'} / ${selectedModelName ?? '-'}`);
    } catch (error) {
      alert(`绑定失败: ${error}`);
    } finally {
      setSaving(false);
    }
  };

  const installSelectedTool = async () => {
    if (!selectedTool) return;
    try {
      const message = await api.installTool(selectedTool.id);
      alert(message);
      setNotice(`已触发 ${selectedMeta?.title ?? selectedTool.name} 的安装流程`);
      await load();
    } catch (error) {
      alert(`安装失败: ${error}`);
    }
  };

  return (
    <div className="grid gap-8 xl:grid-cols-[minmax(0,1fr)_420px]">
      <section>
        <div className="mb-6 grid gap-4 md:grid-cols-3">
          <SummaryCard title="已检测工具" value={`${tools.length}`} detail={`${installedCount} 个已安装`} />
          <SummaryCard title="可自动配置" value={`${configurableCount}`} detail="会优先接入桌面端、Agents 与 CLI" />
          <SummaryCard title="待安装" value={`${needsInstallCount}`} detail="缺失工具可在这里发起安装" />
        </div>

        <div className="mb-6 flex flex-wrap items-center gap-5 border-b border-[#e5dacc] pb-4">
          {categories.map(item => (
            <button
              key={item.id}
              onClick={() => setCategory(item.id)}
              className={`border-b-4 px-4 pb-4 text-2xl font-semibold transition ${
                category === item.id
                  ? 'border-[#d07347] text-[#2f241c]'
                  : 'border-transparent text-[#6e6258] hover:text-[#3f3329]'
              }`}
            >
              {item.label}
            </button>
          ))}
          <button
            onClick={() => load().catch(console.error)}
            className="ml-auto rounded-xl border border-[#e3d7c8] bg-[#fbf7ef] px-5 py-3 text-lg font-semibold text-[#5a4c40] transition hover:bg-white"
          >
            刷新
          </button>
        </div>

        <div className="space-y-8">
          <StudioToolSection
            title={`已安装 · ${installedTools.length}`}
            subtitle={`当前分类：${appCategoryLabel(category)}`}
          >
            {installedTools.map(tool => {
              const det = detection.find(item => item.tool_id === tool.id);
              const active = tool.id === selectedToolId;
              const meta = toolDisplayMeta(tool);
              return (
                <StudioToolCard
                  key={tool.id}
                  tool={tool}
                  detection={det}
                  meta={meta}
                  active={active}
                  selectedBindingLabel={binding && active ? binding.model_name ?? '-' : '-'}
                  onSelect={() => {
                    setSelectedToolId(tool.id);
                    setSelectedProviderId('');
                    setSelectedModelId('');
                  }}
                />
              );
            })}
            {installedTools.length === 0 && <StudioEmpty text="这个分类下还没有检测到已安装工具。" />}
          </StudioToolSection>

          <StudioToolSection
            title={`可安装 · ${availableTools.length}`}
            subtitle="这些工具还没有安装，可以作为下一步接入目标。"
          >
            {availableTools.map(tool => {
              const det = detection.find(item => item.tool_id === tool.id);
              const active = tool.id === selectedToolId;
              const meta = toolDisplayMeta(tool);
              return (
                <StudioToolCard
                  key={tool.id}
                  tool={tool}
                  detection={det}
                  meta={meta}
                  active={active}
                  selectedBindingLabel="-"
                  onSelect={() => {
                    setSelectedToolId(tool.id);
                    setSelectedProviderId('');
                    setSelectedModelId('');
                  }}
                />
              );
            })}
            {availableTools.length === 0 && <StudioEmpty text="这个分类下暂时没有新的可安装工具。" />}
          </StudioToolSection>
        </div>
      </section>

      <aside className="rounded-[32px] border border-[#e3d7c8] bg-[#fbf7ef] p-6 shadow-[0_18px_40px_rgba(92,70,44,0.08)]">
        {!selectedTool ? (
          <div className="grid min-h-[600px] place-items-center text-center text-[#8f7d6a]">
            <div>
              <PackageSearch className="mx-auto h-14 w-14 text-[#c6b6a4]" />
              <p className="mt-4 text-2xl font-semibold">选择要配置的工具</p>
            </div>
          </div>
        ) : (
          <div className="flex h-full flex-col">
            <div className="mb-6">
              <div className="text-sm tracking-[0.25em] text-[#9b8a78]">模型</div>
              <div className="mt-3 flex items-center justify-between gap-4">
                <div>
                  <h3 className="text-3xl font-black text-[#2f241c]">{selectedMeta?.title ?? selectedTool.name}</h3>
                  <p className="mt-2 text-base text-[#8d7d6d]">
                    当前绑定：{binding ? `${binding.provider_name ?? '-'} / ${binding.model_name ?? '-'}` : '尚未绑定'}
                  </p>
                </div>
                <div className={`rounded-full px-4 py-2 text-sm font-semibold ${selectedDetection?.installed ? 'bg-emerald-100 text-emerald-700' : 'bg-amber-100 text-amber-700'}`}>
                  {toolStatus(selectedDetection)}
                </div>
              </div>
              <div className="mt-4 flex flex-wrap gap-2">
                <Tag tone="slate">{selectedMeta ? appCategoryLabel(selectedMeta.category) : '工具'}</Tag>
                <Tag tone={SUPPORTED_BINDING_TOOL_IDS.has(selectedTool.id) ? 'green' : 'amber'}>
                  {SUPPORTED_BINDING_TOOL_IDS.has(selectedTool.id) ? '已接入自动配置' : '待接入自动配置'}
                </Tag>
              </div>
              {notice && (
                <div className="mt-4 rounded-[20px] border border-[#e5d8cb] bg-white px-4 py-3 text-base text-[#6d5a4b]">
                  {notice}
                </div>
              )}
            </div>

            {!SUPPORTED_BINDING_TOOL_IDS.has(selectedTool.id) ? (
              <div className="mt-10 rounded-3xl border border-[#e8d9c5] bg-white p-6 text-[#77695c]">
                <p className="text-xl font-semibold text-[#3a3028]">这个工具还没有接入自动配置。</p>
                <p className="mt-3 text-base">这次重构先把产品结构理顺，后续我们可以按工具逐个补适配。</p>
                {!selectedDetection?.installed && (
                  <button
                    onClick={installSelectedTool}
                    className="mt-6 rounded-2xl bg-[#d07347] px-6 py-3 text-lg font-semibold text-white transition hover:bg-[#c26438]"
                  >
                    AI 自动安装
                  </button>
                )}
              </div>
            ) : (
              <>
                <div className="mb-4 flex items-center justify-between">
                  <div className="text-lg font-semibold text-[#5a4c40]">可选厂商</div>
                  <div className="flex items-center gap-3 text-sm text-[#8d7d6d]">
                    <span>上游直连</span>
                    <span className="h-8 w-14 rounded-full bg-[#ebe1d2]" />
                  </div>
                </div>
                <div className="space-y-4 overflow-y-auto">
                  {compatibleProviders.map(provider => {
                    const active = provider.id === selectedProviderId;
                    return (
                      <button
                        key={provider.id}
                        onClick={() => {
                          setSelectedProviderId(provider.id);
                          const firstModel = models.find(model => model.provider_id === provider.id);
                          setSelectedModelId(firstModel?.id ?? '');
                        }}
                        className={`flex w-full items-start gap-4 rounded-[24px] border px-5 py-5 text-left transition ${
                          active
                            ? 'border-[#d07347] bg-white'
                            : 'border-[#eadfce] bg-white/70 hover:bg-white'
                        }`}
                      >
                        <div className={`mt-2 h-5 w-5 rounded-full border ${active ? 'border-[#d07347] bg-[#d07347]' : 'border-[#d5c6b2]'}`} />
                        <div className="min-w-0">
                          <div className="text-2xl font-bold text-[#2f241c]">{providerDisplayName(provider, duplicateNames)}</div>
                          <div className="mt-2 text-base text-[#8d7d6d]">{provider.api_base.replace(/^https?:\/\//, '')}</div>
                          <div className="mt-3 flex flex-wrap gap-2">
                            <Tag tone="slate">{provider.anthropic_mode ? 'Anthropic' : 'OpenAI'}</Tag>
                            <Tag tone={provider.has_api_key ? 'green' : 'rose'}>{provider.has_api_key ? '已保存 Key' : '缺少 Key'}</Tag>
                          </div>
                        </div>
                      </button>
                    );
                  })}
                </div>

                <div className="mt-6 rounded-[24px] border border-[#eadfce] bg-white p-5">
                  <div className="text-lg font-semibold text-[#5a4c40]">选择模型</div>
                  {selectedTool.id === 'aider-cli' && (
                    <p className="mt-2 text-sm text-[#9d8f81]">Aider 当前只支持 OpenAI 兼容厂商，因此这里会自动过滤 Anthropic 兼容源。</p>
                  )}
                  <select
                    value={selectedModelId}
                    onChange={event => setSelectedModelId(event.target.value)}
                    className="mt-4 w-full rounded-2xl border border-[#dfd2c1] bg-[#fcfaf6] px-4 py-4 text-lg outline-none focus:border-[#d07347]"
                  >
                    <option value="">请选择模型</option>
                    {providerModels.map(model => (
                      <option key={model.id} value={model.id}>{model.name}</option>
                    ))}
                  </select>
                  {selectedProvider && !selectedProvider.has_api_key && (
                    <p className="mt-4 text-base text-[#c15f44]">当前厂商还没有保存 API Key，请先去模型中心编辑厂商。</p>
                  )}
                  {selectedTool.config_path && (
                    <p className="mt-4 text-sm text-[#9d8f81]">配置将写入：{selectedTool.config_path}</p>
                  )}
                </div>

                <div className="mt-auto grid gap-4 pt-8">
                  <div className="rounded-[24px] border border-[#eadfce] bg-white px-5 py-4 text-[#7a6c5f]">
                    <div className="text-sm tracking-[0.2em] text-[#a18f7d]">当前策略</div>
                    <div className="mt-2 text-lg">
                      {selectedProvider
                        ? `${providerDisplayName(selectedProvider, duplicateNames)} / ${models.find(model => model.id === selectedModelId)?.name ?? '未选择模型'}`
                        : '请先选择厂商与模型'}
                    </div>
                  </div>
                  {!selectedDetection?.installed ? (
                    <button
                      onClick={installSelectedTool}
                      className="rounded-3xl bg-[#d07347] px-8 py-5 text-2xl font-bold text-white transition hover:bg-[#c26438]"
                    >
                      AI 自动安装
                    </button>
                  ) : (
                    <>
                      <button
                        onClick={applyBinding}
                        disabled={saving || !selectedProvider || !selectedModelId || !selectedProvider.has_api_key}
                        className="rounded-3xl bg-[#d07347] px-8 py-5 text-2xl font-bold text-white transition hover:bg-[#c26438] disabled:cursor-not-allowed disabled:bg-[#e5d0c3]"
                      >
                        {saving ? '保存配置中...' : '保存模型配置'}
                      </button>
                      <button
                        onClick={async () => {
                          try {
                            await api.launchTool(selectedTool.id);
                          } catch (error) {
                            alert(`启动失败: ${error}`);
                          }
                        }}
                        className="rounded-3xl border border-[#d9cdbd] bg-white px-8 py-4 text-xl font-semibold text-[#57483c] transition hover:bg-[#f7f2ea]"
                      >
                        启动应用
                      </button>
                    </>
                  )}
                  <div className="grid grid-cols-2 gap-3 text-sm text-[#8d7d6d]">
                    <label className="flex items-center gap-2">
                      <input type="checkbox" defaultChecked />
                      直接启动应用
                    </label>
                    <label className="flex items-center gap-2">
                      <input type="checkbox" defaultChecked />
                      修改模型配置
                    </label>
                  </div>
                </div>
              </>
            )}
          </div>
        )}
      </aside>
    </div>
  );
}

function ProviderEditorModal({
  presets,
  provider,
  onClose,
  onSaved,
}: {
  presets: VendorPreset[];
  provider?: Provider;
  onClose: () => void;
  onSaved: () => void;
}) {
  const [selectedPresetId, setSelectedPresetId] = useState<string>(provider ? CUSTOM_PROVIDER_PRESET : CUSTOM_PROVIDER_PRESET);
  const [form, setForm] = useState({
    id: provider?.id ?? '',
    name: provider?.name ?? '',
    api_base: provider?.api_base ?? '',
    anthropic_mode: provider?.anthropic_mode ?? true,
    api_key: '',
  });
  const [saving, setSaving] = useState(false);

  const applyPreset = (presetId: string) => {
    setSelectedPresetId(presetId);
    if (presetId === CUSTOM_PROVIDER_PRESET) return;
    const preset = presets.find(item => item.id === presetId);
    if (!preset) return;
    setForm(current => ({
      ...current,
      id: preset.id,
      name: preset.name,
      api_base: preset.api_base,
      anthropic_mode: ANTHROPIC_PRESET_IDS.has(preset.id) || preset.api_base.includes('/anthropic'),
    }));
  };

  const save = async () => {
    const generatedId = form.id.trim() || slugifyProviderId(form.name);
    if (!generatedId || !form.name.trim() || !form.api_base.trim()) return;
    setSaving(true);
    try {
      if (provider) {
        await api.updateProvider({
          id: provider.id,
          name: form.name.trim(),
          api_base: form.api_base.trim(),
          anthropic_mode: form.anthropic_mode,
          api_key: form.api_key || undefined,
        });
      } else {
        await api.createProvider({
          id: generatedId,
          name: form.name.trim(),
          api_base: form.api_base.trim(),
          anthropic_mode: form.anthropic_mode,
          api_key: form.api_key || undefined,
        });
      }
      onSaved();
    } catch (error) {
      alert(`保存失败: ${error}`);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Modal title={provider ? '编辑厂商' : '添加厂商'} onClose={onClose}>
      <div className="space-y-4">
        {!provider && (
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-500">使用预设</label>
            <select
              value={selectedPresetId}
              onChange={event => applyPreset(event.target.value)}
              className="w-full rounded-xl border border-slate-200 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-[#d07347]"
            >
              <option value={CUSTOM_PROVIDER_PRESET}>自定义厂商</option>
              {presets.map(preset => (
                <option key={preset.id} value={preset.id}>{preset.name}</option>
              ))}
            </select>
          </div>
        )}
        <Field label="厂商标识" value={form.id} onChange={value => setForm(current => ({ ...current, id: slugifyProviderId(value) }))} placeholder="留空时按名称自动生成" />
        <Field label="名称" value={form.name} onChange={value => setForm(current => ({ ...current, name: value }))} placeholder="例如：DeepSeek" />
        <Field label="API Base URL" value={form.api_base} onChange={value => setForm(current => ({ ...current, api_base: value }))} placeholder="https://api.deepseek.com/anthropic" />
        <div>
          <label className="mb-1 block text-xs font-medium text-slate-500">{provider ? 'API Key（留空不修改）' : 'API Key（建议直接填写）'}</label>
          <input
            type="password"
            value={form.api_key}
            onChange={event => setForm(current => ({ ...current, api_key: event.target.value }))}
            placeholder="sk-..."
            className="w-full rounded-xl border border-slate-200 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-[#d07347]"
          />
        </div>
        <label className="flex items-center gap-2 text-sm text-slate-700">
          <input
            type="checkbox"
            checked={form.anthropic_mode}
            onChange={event => setForm(current => ({ ...current, anthropic_mode: event.target.checked }))}
          />
          Anthropic 兼容模式
        </label>
      </div>
      <div className="mt-6 flex justify-end gap-2">
        <button onClick={onClose} className="px-4 py-2 text-sm text-slate-600 hover:text-slate-800">取消</button>
        <button
          onClick={save}
          disabled={saving}
          className="rounded-xl bg-[#d07347] px-5 py-2 text-sm font-semibold text-white hover:bg-[#c26438] disabled:bg-slate-300"
        >
          {saving ? '保存中...' : provider ? '保存' : '添加'}
        </button>
      </div>
    </Modal>
  );
}

function ModelEditorModal({
  providers,
  model,
  onClose,
  onSaved,
}: {
  providers: Provider[];
  model?: Model;
  onClose: () => void;
  onSaved: () => void;
}) {
  const [providerId, setProviderId] = useState(model?.provider_id ?? providers[0]?.id ?? '');
  const [name, setName] = useState(model?.name ?? '');
  const [modelId, setModelId] = useState(model?.model_id ?? '');
  const [ctxLen, setCtxLen] = useState(String(model?.context_length ?? 128000));
  const [maxOut, setMaxOut] = useState(String(model?.max_output ?? 8192));
  const [saving, setSaving] = useState(false);

  const save = async () => {
    if (!providerId || !name.trim() || !modelId.trim()) return;
    setSaving(true);
    try {
      if (model) {
        await api.updateModel({
          id: model.id,
          provider_id: providerId,
          name: name.trim(),
          model_id: modelId.trim(),
          context_length: Number.parseInt(ctxLen, 10) || 128000,
          max_output: Number.parseInt(maxOut, 10) || 8192,
        });
      } else {
        await api.createModel({
          provider_id: providerId,
          name: name.trim(),
          model_id: modelId.trim(),
          context_length: Number.parseInt(ctxLen, 10) || 128000,
          max_output: Number.parseInt(maxOut, 10) || 8192,
        });
      }
      onSaved();
    } catch (error) {
      alert(`保存失败: ${error}`);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Modal title={model ? '编辑模型' : '添加模型'} onClose={onClose}>
      <div className="space-y-4">
        <div>
          <label className="mb-1 block text-xs font-medium text-slate-500">所属厂商</label>
          <select
            value={providerId}
            onChange={event => setProviderId(event.target.value)}
            className="w-full rounded-xl border border-slate-200 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-[#d07347]"
          >
            {providers.map(provider => (
              <option key={provider.id} value={provider.id}>
                {provider.name} ({provider.id})
              </option>
            ))}
          </select>
        </div>
        <Field label="显示名称" value={name} onChange={setName} placeholder="例如：DeepSeek Chat" />
        <Field label="模型 ID" value={modelId} onChange={setModelId} placeholder="例如：deepseek-chat" />
        <div className="grid grid-cols-2 gap-3">
          <Field label="上下文长度" value={ctxLen} onChange={setCtxLen} placeholder="128000" />
          <Field label="最大输出" value={maxOut} onChange={setMaxOut} placeholder="8192" />
        </div>
      </div>
      <div className="mt-6 flex justify-end gap-2">
        <button onClick={onClose} className="px-4 py-2 text-sm text-slate-600 hover:text-slate-800">取消</button>
        <button
          onClick={save}
          disabled={saving}
          className="rounded-xl bg-[#d07347] px-5 py-2 text-sm font-semibold text-white hover:bg-[#c26438] disabled:bg-slate-300"
        >
          {saving ? '保存中...' : model ? '保存' : '添加'}
        </button>
      </div>
    </Modal>
  );
}

function PlaceholderPanel({ title, body }: { title: string; body: string }) {
  return (
    <div className="rounded-[32px] border border-[#e3d7c8] bg-[#fbf7ef] p-10 shadow-[0_18px_40px_rgba(92,70,44,0.08)]">
      <div className="max-w-3xl">
        <div className="text-3xl font-black text-[#2f241c]">{title}</div>
        <p className="mt-4 text-lg leading-8 text-[#7d6f63]">{body}</p>
      </div>
    </div>
  );
}

function Modal({ title, children, onClose }: { title: string; children: React.ReactNode; onClose: () => void }) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-[rgba(30,24,18,0.28)] p-6" onClick={onClose}>
      <div
        className="w-full max-w-xl rounded-[32px] border border-[#eadfce] bg-[#fffdfa] p-8 shadow-[0_28px_60px_rgba(65,45,25,0.18)]"
        onClick={event => event.stopPropagation()}
      >
        <h3 className="mb-6 text-3xl font-black text-[#2f241c]">{title}</h3>
        {children}
      </div>
    </div>
  );
}

function Field({
  label,
  value,
  onChange,
  placeholder,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}) {
  return (
    <div>
      <label className="mb-1 block text-xs font-medium text-slate-500">{label}</label>
      <input
        type="text"
        value={value}
        onChange={event => onChange(event.target.value)}
        placeholder={placeholder}
        className="w-full rounded-xl border border-slate-200 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-[#d07347]"
      />
    </div>
  );
}

function SummaryCard({ title, value, detail }: { title: string; value: string; detail: string }) {
  return (
    <div className="rounded-[24px] border border-[#e3d7c8] bg-[#fbf7ef] px-5 py-5 shadow-[0_18px_40px_rgba(92,70,44,0.06)]">
      <div className="text-sm tracking-[0.22em] text-[#9a8a79]">{title}</div>
      <div className="mt-3 text-4xl font-black text-[#2f241c]">{value}</div>
      <div className="mt-2 text-base text-[#7c6c5b]">{detail}</div>
    </div>
  );
}

function StudioToolSection({
  title,
  subtitle,
  children,
}: {
  title: string;
  subtitle: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <div className="mb-4 flex items-end justify-between gap-4">
        <div>
          <h4 className="text-2xl font-black text-[#2f241c]">{title}</h4>
          <p className="mt-1 text-base text-[#867867]">{subtitle}</p>
        </div>
      </div>
      <div className="grid gap-6 md:grid-cols-2 xl:grid-cols-3">{children}</div>
    </div>
  );
}

function StudioToolCard({
  tool,
  detection,
  meta,
  active,
  selectedBindingLabel,
  onSelect,
}: {
  tool: Tool;
  detection?: DetectionResult;
  meta: { title: string; icon: string; category: AppCategory; installLabel?: string };
  active: boolean;
  selectedBindingLabel: string;
  onSelect: () => void;
}) {
  const isInstalled = !!detection?.installed;

  return (
    <button
      onClick={onSelect}
      className={`min-h-[250px] rounded-[28px] border p-6 text-left shadow-[0_18px_40px_rgba(92,70,44,0.08)] transition ${
        active
          ? 'border-[#d07347] bg-[#fff8f1]'
          : 'border-[#e3d7c8] bg-[#fbf7ef] hover:bg-white'
      }`}
    >
      <div className="mb-6 flex items-start justify-between gap-4">
        <div>
          <div className={`mb-4 inline-flex h-14 w-14 items-center justify-center rounded-2xl text-3xl font-black ${
            isInstalled ? 'bg-[#f3e4d6] text-[#c86d46]' : 'bg-[#ece5dc] text-[#9f9387]'
          }`}>
            {meta.icon}
          </div>
          <div className="text-2xl font-black text-[#2f241c]">{meta.title}</div>
          <div className="mt-3 text-lg text-[#7b6c5d]">模型：{selectedBindingLabel}</div>
        </div>
        <div className={`rounded-2xl px-4 py-2 text-sm font-semibold ${isInstalled ? 'bg-emerald-100 text-emerald-700' : 'bg-amber-100 text-amber-700'}`}>
          {toolStatus(detection)}
        </div>
      </div>
      <div className="space-y-2 text-base text-[#8d7d6d]">
        <div>应用：{trimPath(tool.launch_path ?? tool.launch_command ?? '-')}</div>
        <div>配置：{trimPath(tool.config_path ?? '-')}</div>
        <div>版本：{detection?.versions[0]?.replace(/^cli:/, '') ?? '-'}</div>
      </div>
      <div className="mt-5 flex flex-wrap gap-2">
        <Tag tone="slate">{appCategoryLabel(meta.category)}</Tag>
        <Tag tone={SUPPORTED_BINDING_TOOL_IDS.has(tool.id) ? 'green' : 'amber'}>
          {SUPPORTED_BINDING_TOOL_IDS.has(tool.id) ? '支持配置' : '待接入'}
        </Tag>
      </div>
      {!isInstalled && (
        <div className="mt-8">
          <span className="inline-flex rounded-2xl bg-[#ddd2c5] px-5 py-3 text-lg font-semibold text-[#5e5248]">
            {meta.installLabel ?? 'AI 自动安装'}
          </span>
        </div>
      )}
    </button>
  );
}

function StudioEmpty({ text }: { text: string }) {
  return (
    <div className="col-span-full rounded-[28px] border border-dashed border-[#d9cdbd] bg-[#fbf8f1] px-8 py-10 text-center text-lg text-[#8f7d6a]">
      {text}
    </div>
  );
}

function Tag({ tone, children }: { tone: 'slate' | 'amber' | 'blue' | 'green' | 'rose'; children: React.ReactNode }) {
  const styles: Record<string, string> = {
    slate: 'bg-white/80 text-[#7f7265] border border-[#e5dacc]',
    amber: 'bg-[#fff1d6] text-[#b27016]',
    blue: 'bg-[#e5f0ff] text-[#3f72c8]',
    green: 'bg-[#def6e8] text-[#2f9157]',
    rose: 'bg-[#ffe5df] text-[#c35c44]',
  };

  return <span className={`rounded-full px-3 py-1 text-sm font-semibold ${styles[tone]}`}>{children}</span>;
}
