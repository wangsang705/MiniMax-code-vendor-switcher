import { useCallback, useEffect, useMemo, useState, useDeferredValue } from 'react';
import { api, type Model, type Provider, type VendorPreset } from '../api';
import { ProviderEditorModal } from '../components/ProviderEditorModal';
import { ModelEditorModal } from '../components/ModelEditorModal';
import { Tag } from '../components/ui/Tag';
import { SummaryCard } from '../components/ui/SummaryCard';
import {
  duplicateProviderNames,
  formatContextLength,
  formatModelStatus,
  providerDisplayName,
  providerTone,
} from '../lib/app-utils';
import { Zap, Building2, Radio, Search, Loader2, Globe, CheckCircle2 } from 'lucide-react';

type ModelTab = 'vendors' | 'relay' | 'speed';

export default function ModelCenterPage() {
  const [activeTab, setActiveTab] = useState<ModelTab>('vendors');
  const [providers, setProviders] = useState<Provider[]>([]);
  const [models, setModels] = useState<Model[]>([]);
  const [presets, setPresets] = useState<VendorPreset[]>([]);
  const [showAddProvider, setShowAddProvider] = useState(false);
  const [showAddModel, setShowAddModel] = useState(false);
  const [editingProvider, setEditingProvider] = useState<Provider | null>(null);
  const [editingModel, setEditingModel] = useState<Model | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const deferredSearch = useDeferredValue(searchQuery);

  // 连接测试
  const [speedTesting, setSpeedTesting] = useState(false);
  const [speedResults, setSpeedResults] = useState<Record<string, number | 'error' | null>>({});

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

  // 搜索筛选后的模型（使用 deferredSearch 做防抖）
  const filteredModelCards = useMemo(() => {
    if (!deferredSearch.trim()) {
      return models
        .map(model => ({ model, provider: providerMap.get(model.provider_id) }))
        .filter((item): item is { model: Model; provider: Provider } => Boolean(item.provider));
    }
    const q = deferredSearch.toLowerCase();
    return models
      .map(model => ({ model, provider: providerMap.get(model.provider_id) }))
      .filter((item): item is { model: Model; provider: Provider } => {
        if (!item.provider) return false;
        return (
          item.model.name.toLowerCase().includes(q) ||
          item.model.model_id.toLowerCase().includes(q) ||
          item.provider.name.toLowerCase().includes(q) ||
          item.provider.id.toLowerCase().includes(q)
        );
      });
  }, [models, providerMap, deferredSearch]);

  // 搜索筛选后的厂商
  const filteredProviders = useMemo(() => {
    if (!deferredSearch.trim()) return providers;
    const q = deferredSearch.toLowerCase();
    return providers.filter(
      p =>
        p.name.toLowerCase().includes(q) ||
        p.id.toLowerCase().includes(q) ||
        p.api_base.toLowerCase().includes(q)
    );
  }, [providers, deferredSearch]);

  const readyProviderCount = providers.filter(provider => provider.has_api_key).length;
  const anthropicReadyCount = providers.filter(provider => provider.anthropic_mode).length;

  // 速度测试
  const runSpeedTest = async () => {
    setSpeedTesting(true);
    setSpeedResults({});
    const testProviders = providers.filter(p => p.has_api_key);
    if (testProviders.length === 0) {
      setSpeedTesting(false);
      return;
    }
    const results: Record<string, number | 'error'> = {};
    for (const p of testProviders) {
      const start = performance.now();
      try {
        const url = `${p.api_base.replace(/\/$/, '')}/models`;
        const controller = new AbortController();
        const timeout = setTimeout(() => controller.abort(), 5000);
        const resp = await fetch(url, {
          method: 'GET',
          signal: controller.signal,
          headers: p.anthropic_mode ? { 'x-api-key': 'test' } : { 'Authorization': 'Bearer test' },
        });
        clearTimeout(timeout);
        if (resp.ok || resp.status === 401) {
          // 401 表示端点可达但需要认证——这算连接成功
          results[p.id] = Math.round(performance.now() - start);
        } else {
          results[p.id] = 'error';
        }
      } catch {
        results[p.id] = 'error';
      }
      setSpeedResults({ ...results });
    }
    setSpeedTesting(false);
  };

  const tabs = [
    { key: 'vendors' as const, label: '大模型厂商', icon: Building2 },
    { key: 'relay' as const, label: '模型中转站', icon: Radio },
    { key: 'speed' as const, label: '测试速度', icon: Zap },
  ];

  return (
    <div className="space-y-6">
      {/* Tab 切换栏 */}
      <div className="flex flex-wrap items-center gap-3">
        {tabs.map(tab => {
          const Icon = tab.icon;
          const active = activeTab === tab.key;
          return (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key)}
              className={`flex items-center gap-2 rounded-xl px-6 py-3 text-lg font-semibold transition ${
                active
                  ? 'bg-[#d07347] text-white shadow-[0_8px_25px_rgba(208,115,71,0.25)]'
                  : 'bg-[#fbf7ef] text-[#6d6257] hover:bg-[#efe6da] border border-[#e3d7c8]'
              }`}
            >
              <Icon className="h-5 w-5" />
              {tab.label}
            </button>
          );
        })}
      </div>

      {/* ============ Tab: 大模型厂商 ============ */}
      {activeTab === 'vendors' && (
        <>
          {/* 搜索栏 */}
          <div className="relative">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-5 w-5 text-[#9b8a78]" />
            <input
              type="text"
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
              placeholder="搜索厂商或模型名称..."
              className="w-full rounded-2xl border border-[#e3d7c8] bg-[#fbf7ef] py-4 pl-12 pr-10 text-lg outline-none transition focus:border-[#d07347] focus:bg-white"
            />
            {searchQuery && (
              <button
                onClick={() => setSearchQuery('')}
                className="absolute right-4 top-1/2 -translate-y-1/2 text-sm text-[#9b8a78] hover:text-[#6d6257]"
              >
                清除
              </button>
            )}
          </div>

          {/* 统计卡片 */}
          <div className="grid gap-4 md:grid-cols-3">
            <SummaryCard
              title="已接入厂商"
              value={`${providers.length}`}
              detail={`${readyProviderCount} 个已保存 Key`}
            />
            <SummaryCard
              title="模型条目"
              value={`${models.length}`}
              detail="用于桌面端、Agents 与 CLI 绑定"
            />
            <SummaryCard
              title="Anthropic 兼容"
              value={`${anthropicReadyCount}`}
              detail="适合 Claude / MiniMax 一类工具"
            />
          </div>

          {/* 模型卡片网格 */}
          <div className="grid gap-6 md:grid-cols-2 xl:grid-cols-3">
            {filteredModelCards.map(({ model, provider }) => (
              <article
                key={model.id}
                className={`rounded-[28px] border bg-gradient-to-br p-6 shadow-[0_18px_40px_rgba(92,70,44,0.08)] ${providerTone(provider)}`}
              >
                <div className="mb-4 flex items-start justify-between gap-4">
                  <div>
                    <div className="text-sm tracking-[0.25em] text-[#9b8a78]">云端</div>
                    <h3 className="mt-3 text-2xl font-black text-[#2f241c]">
                      {providerDisplayName(provider, duplicateNames)}
                    </h3>
                  </div>
                  <div className="space-x-2 text-sm text-[#9d8f81]">
                    <button onClick={() => setEditingModel(model)} className="hover:text-[#6e5949]">
                      编辑模型
                    </button>
                    <button onClick={() => setEditingProvider(provider)} className="hover:text-[#6e5949]">
                      编辑
                    </button>
                    <button
                      onClick={async () => {
                        if (
                          !confirm(
                            `⚠️ 删除厂商"${provider.name}"会同时删除该厂商下的所有模型和工具绑定（包括已保存的 API Key），确定吗？`
                          )
                        )
                          return;
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
                  <div>
                    模型：<span className="font-semibold text-[#53483d]">{model.model_id}</span>
                  </div>
                  <div>
                    来源：
                    <span className="font-mono text-[#53483d]">
                      {provider.api_base.replace(/^https?:\/\//, '')}
                    </span>
                  </div>
                  <div>
                    上下文：
                    <span className="font-semibold text-[#53483d]">
                      {formatContextLength(model.context_length)}
                    </span>
                  </div>
                </div>

                <div className="mt-6 flex flex-wrap gap-2">
                  <Tag tone={provider.has_api_key ? 'green' : 'rose'}>
                    {formatModelStatus(provider, model)}
                  </Tag>
                  <Tag tone="slate">{provider.anthropic_mode ? 'Anthropic' : 'OpenAI'}</Tag>
                  {model.supports_reasoning && <Tag tone="amber">推理</Tag>}
                  {model.supports_tool_call && <Tag tone="blue">工具调用</Tag>}
                  {provider.has_api_key ? (
                    <Tag tone="green">已保存 Key</Tag>
                  ) : (
                    <Tag tone="rose">缺少 Key</Tag>
                  )}
                </div>
                <div className="mt-5 flex justify-end">
                  <button
                    onClick={async () => {
                      if (!confirm(`确定删除模型"${model.name}"吗？`)) return;
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

            {filteredModelCards.length === 0 && searchQuery && (
              <div className="col-span-full rounded-[28px] border border-dashed border-[#d9cdbd] bg-[#fbf8f1] px-8 py-12 text-center text-lg text-[#8f7d6a]">
                没有搜索到匹配的模型或厂商
              </div>
            )}

            {filteredModelCards.length === 0 && !searchQuery && (
              <div className="col-span-full rounded-[28px] border border-dashed border-[#d9cdbd] bg-[#fbf8f1] px-8 py-12 text-center text-lg text-[#8f7d6a]">
                还没有添加任何模型，点击下方按钮开始添加
              </div>
            )}

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

          {/* 厂商列表面板 */}
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
              {filteredProviders.map(provider => (
                <div
                  key={provider.id}
                  className="rounded-2xl border border-[#eadfce] bg-white px-4 py-4"
                >
                  <div className="flex items-start justify-between gap-3">
                    <div>
                      <div className="text-xl font-bold text-[#2f241c]">
                        {providerDisplayName(provider, duplicateNames)}
                      </div>
                      <div className="mt-1 text-sm text-[#9a8a79]">
                        {provider.api_base.replace(/^https?:\/\//, '')}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => setEditingProvider(provider)}
                        className="text-sm font-semibold text-[#6e5949] hover:text-[#3f3329]"
                      >
                        编辑
                      </button>
                      <button
                        onClick={async () => {
                          if (
                            !confirm(
                              `⚠️ 删除厂商"${provider.name}"会同时删除该厂商下的所有模型和工具绑定（包括已保存的 API Key），确定吗？`
                            )
                          )
                            return;
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
                    <Tag tone="slate">
                      {provider.anthropic_mode ? 'Anthropic 兼容' : 'OpenAI 格式'}
                    </Tag>
                    <Tag tone={provider.has_api_key ? 'green' : 'rose'}>
                      {provider.has_api_key ? '已保存 Key' : '缺少 Key'}
                    </Tag>
                    <Tag tone="amber">
                      {models.filter(model => model.provider_id === provider.id).length} 个模型
                    </Tag>
                  </div>
                </div>
              ))}
              {filteredProviders.length === 0 && searchQuery && (
                <div className="col-span-full py-8 text-center text-lg text-[#8f7d6a]">
                  没有搜索到匹配的厂商
                </div>
              )}
            </div>
          </section>
        </>
      )}

      {/* ============ Tab: 模型中转站 ============ */}
      {activeTab === 'relay' && (
        <div className="rounded-[28px] border border-[#e3d7c8] bg-[#fbf7ef] p-8 shadow-[0_18px_40px_rgba(92,70,44,0.08)]">
          <div className="flex items-center gap-4 mb-6">
            <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-br from-[#ff8b4d] to-[#d8642f] text-white">
              <Radio className="h-8 w-8" />
            </div>
            <div>
              <h3 className="text-3xl font-black text-[#2f241c]">模型中转站</h3>
              <p className="mt-2 text-lg text-[#8d7d6d]">
                配置 API 中转代理，统一管理多个厂商的访问入口。
              </p>
            </div>
          </div>

          <div className="grid gap-6 md:grid-cols-2">
            <div className="rounded-2xl border border-[#eadfce] bg-white p-6">
              <div className="flex items-center gap-2 text-lg font-bold text-[#2f241c] mb-3">
                <Globe className="h-5 w-5 text-[#d07347]" />
                直连模式（默认）
              </div>
              <p className="text-[#8d7d6d]">
                直接连接各厂商的官方 API 端点。无需额外配置，每个厂商独立管理。
              </p>
              <div className="mt-4 flex items-center gap-2 text-sm">
                <CheckCircle2 className="h-4 w-4 text-emerald-500" />
                <span className="text-[#6f6256]">当前使用{providers.length}个厂商端点</span>
              </div>
            </div>
            <div className="rounded-2xl border border-dashed border-[#d9cdbd] bg-[#fbf8f1] p-6 text-center">
              <div className="text-4xl mb-3">🔄</div>
              <div className="text-xl font-bold text-[#43382e]">自定义中转代理</div>
              <p className="mt-2 text-base text-[#9d8f81]">
                通过统一的反向代理地址访问所有厂商，集中管理 API Key 和流量。
              </p>
              <button
                onClick={() => setActiveTab('vendors')}
                className="mt-6 rounded-xl bg-[#d07347] px-6 py-3 text-lg font-semibold text-white transition hover:bg-[#c26438]"
              >
                先添加厂商
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ============ Tab: 测试速度 ============ */}
      {activeTab === 'speed' && (
        <div className="rounded-[28px] border border-[#e3d7c8] bg-[#fbf7ef] p-8 shadow-[0_18px_40px_rgba(92,70,44,0.08)]">
          <div className="flex items-center gap-4 mb-8">
            <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-br from-amber-500 to-orange-500 text-white">
              <Zap className="h-8 w-8" />
            </div>
            <div>
              <h3 className="text-3xl font-black text-[#2f241c]">连接速度测试</h3>
              <p className="mt-2 text-lg text-[#8d7d6d]">
                测试各厂商 API 端点的网络延迟，选择最快的接入方案。
              </p>
            </div>
          </div>

          {/* 需配置 Key 提示 */}
          {providers.filter(p => p.has_api_key).length === 0 && (
            <div className="mb-6 rounded-2xl border border-amber-200 bg-amber-50 px-5 py-4 text-amber-800">
              还没有厂商保存了 API Key。请先在「大模型厂商」Tab 中添加厂商并填写 Key。
            </div>
          )}

          <div className="space-y-4 mb-8">
            {providers.map(provider => {
              const result = speedResults[provider.id];
              return (
                <div
                  key={provider.id}
                  className="flex items-center gap-4 rounded-2xl border border-[#eadfce] bg-white px-5 py-4"
                >
                  <div
                    className={`flex h-10 w-10 items-center justify-center rounded-xl text-lg font-bold ${
                      provider.has_api_key
                        ? 'bg-gradient-to-br from-[#ff8b4d] to-[#d8642f] text-white'
                        : 'bg-[#ece5dc] text-[#9f9387]'
                    }`}
                  >
                    {provider.name.slice(0, 1).toUpperCase()}
                  </div>
                  <div className="min-w-0 flex-1">
                    <div className="font-bold text-[#2f241c]">{provider.name}</div>
                    <div className="text-sm text-[#9a8a79] truncate">
                      {provider.api_base.replace(/^https?:\/\//, '')}
                    </div>
                  </div>
                  <div className="text-right min-w-[80px]">
                    {result === undefined && !speedTesting && (
                      <span className="text-sm text-[#9a8a79]">待测试</span>
                    )}
                    {result === undefined && speedTesting && (
                      <Loader2 className="h-5 w-5 animate-spin text-[#d07347] ml-auto" />
                    )}
                    {result === 'error' && (
                      <span className="text-sm font-semibold text-red-500">连接失败</span>
                    )}
                    {result !== undefined && result !== 'error' && (
                      <div>
                        <span
                          className={`text-lg font-bold ${
                            (result as number) < 200
                              ? 'text-emerald-500'
                              : (result as number) < 500
                                ? 'text-amber-500'
                                : 'text-red-500'
                          }`}
                        >
                          {result as number}ms
                        </span>
                      </div>
                    )}
                  </div>
                  {!provider.has_api_key && (
                    <span className="text-xs rounded-full bg-rose-100 text-rose-700 px-2.5 py-1">
                      缺少 Key
                    </span>
                  )}
                </div>
              );
            })}
          </div>

          <button
            onClick={runSpeedTest}
            disabled={speedTesting || providers.filter(p => p.has_api_key).length === 0}
            className="flex items-center gap-2 rounded-2xl bg-[#d07347] px-8 py-4 text-xl font-bold text-white transition hover:bg-[#c26438] disabled:cursor-not-allowed disabled:bg-[#e5d0c3]"
          >
            {speedTesting ? (
              <>
                <Loader2 className="h-5 w-5 animate-spin" />
                测试中...
              </>
            ) : (
              <>
                <Zap className="h-5 w-5" />
                开始测试
              </>
            )}
          </button>
        </div>
      )}

      {/* 弹窗 */}
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
