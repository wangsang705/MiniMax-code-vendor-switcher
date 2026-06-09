import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Loader2, PackageSearch } from 'lucide-react';
import { api, type DetectionResult, type Model, type Provider, type Tool } from '../api';
import { StudioToolCard } from '../components/StudioToolCard';
import { SummaryCard } from '../components/ui/SummaryCard';
import { Tag } from '../components/ui/Tag';
import { StudioToolSection } from '../components/ui/StudioToolSection';
import { StudioEmpty } from '../components/ui/StudioEmpty';
import { useToast } from '../hooks/use-toast';
import { SUPPORTED_BINDING_TOOL_IDS } from '../lib/app-types';
import {
  appCategoryLabel,
  duplicateProviderNames,
  providerDisplayName,
  toolDisplayMeta,
  toolStatus,
} from '../lib/app-utils';

type AppCategory = 'all' | 'desktop' | 'agents' | 'ide' | 'cli' | 'tools';

export default function ApplicationStudioPage() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [models, setModels] = useState<Model[]>([]);
  const [tools, setTools] = useState<Tool[]>([]);
  const [detection, setDetection] = useState<DetectionResult[]>([]);
  const [category, setCategory] = useState<AppCategory>('all');
  const [selectedToolId, setSelectedToolId] = useState<string>('');
  const [selectedProviderId, setSelectedProviderId] = useState<string>('');
  const [selectedModelId, setSelectedModelId] = useState<string>('');
  const [binding, setBinding] = useState<{
    provider_name: string | null;
    model_name: string | null;
  } | null>(null);
  const [saving, setSaving] = useState(false);
  const [loading, setLoading] = useState(true);
  const [notice, setNotice] = useState<string>('');
  const [autoLaunch, setAutoLaunch] = useState(false);
  const toast = useToast();
  const initialSelectedDone = useRef(false);

  const load = useCallback(async () => {
    setLoading(true);
    try {
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
      if (!initialSelectedDone.current && !selectedToolId) {
        initialSelectedDone.current = true;
        const installed = toolList.find(
          tool => detectionList.find(item => item.tool_id === tool.id)?.installed
        );
        if (installed?.id ?? toolList[0]?.id) {
          setSelectedToolId(installed?.id ?? toolList[0]?.id ?? '');
        }
      }
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load().catch(console.error);
  }, [load]);

  useEffect(() => {
    if (!selectedToolId) return;
    api
      .getToolBinding(selectedToolId)
      .then(result => {
        setBinding(
          result
            ? { provider_name: result.provider_name, model_name: result.model_name }
            : null
        );
        if (result) {
          setSelectedProviderId(result.provider_id);
          setSelectedModelId(result.model_id);
        }
      })
      .catch(() => setBinding(null));
  }, [selectedToolId]);

  const selectedTool = tools.find(tool => tool.id === selectedToolId) ?? null;
  const selectedDetection = detection.find(
    item => item.tool_id === selectedToolId
  );
  const selectedProvider = providers.find(
    provider => provider.id === selectedProviderId
  ) ?? null;
  const selectedMeta = selectedTool ? toolDisplayMeta(selectedTool) : null;
  const duplicateNames = duplicateProviderNames(providers);
  const installedCount = detection.filter(item => item.installed).length;
  const configurableCount = tools.filter(tool =>
    SUPPORTED_BINDING_TOOL_IDS.has(tool.id)
  ).length;
  const needsInstallCount = Math.max(tools.length - installedCount, 0);
  const compatibleProviders = useMemo(() => {
    if (!selectedTool) return providers;
    if (selectedTool.id === 'aider-cli') {
      return providers.filter(provider => !provider.anthropic_mode);
    }
    return providers;
  }, [providers, selectedTool]);
  const providerModels = models.filter(
    model => model.provider_id === selectedProviderId
  );

  const categories: Array<{ id: AppCategory; label: string }> = [
    { id: 'all', label: '全部' },
    { id: 'cli', label: 'CLI Code' },
    { id: 'desktop', label: '桌面端' },
    { id: 'agents', label: 'Agents' },
    { id: 'tools', label: '工具' },
  ];

  const visibleTools = tools.filter(tool => {
    const meta = toolDisplayMeta(tool);
    return category === 'all' || meta.category === category;
  });
  const installedTools = visibleTools.filter(tool =>
    detection.find(item => item.tool_id === tool.id)?.installed
  );
  const availableTools = visibleTools.filter(
    tool => !detection.find(item => item.tool_id === tool.id)?.installed
  );

  useEffect(() => {
    if (!selectedTool || !selectedProviderId) return;
    if (selectedTool.id === 'aider-cli') {
      const provider = providers.find(
        item => item.id === selectedProviderId
      );
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
      await api.applyBinding(
        selectedTool.id,
        selectedProviderId,
        selectedModelId
      );
      const selectedProviderName =
        providers.find(provider => provider.id === selectedProviderId)
          ?.name ?? null;
      const selectedModelName =
        models.find(model => model.id === selectedModelId)?.name ?? null;
      setBinding({
        provider_name: selectedProviderName,
        model_name: selectedModelName,
      });
      const msg = `已将 ${selectedMeta?.title ?? selectedTool.name} 绑定到 ${selectedProviderName ?? '-'} / ${selectedModelName ?? '-'}`;
      setNotice(msg);
      toast.success(msg);

      // 如果勾选了"直接启动应用"，绑定成功后自动启动
      if (autoLaunch && selectedDetection?.installed) {
        try {
          await api.launchTool(selectedTool.id);
          toast.success(`已启动 ${selectedMeta?.title ?? selectedTool.name}`);
        } catch (launchErr) {
          toast.error(`启动失败: ${launchErr}`);
        }
      }
    } catch (error) {
      toast.error(`绑定失败: ${error}`);
    } finally {
      setSaving(false);
    }
  };

  const installSelectedTool = async () => {
    if (!selectedTool) return;
    try {
      const message = await api.installTool(selectedTool.id);
      toast.success(message);
      setNotice(
        `已触发 ${selectedMeta?.title ?? selectedTool.name} 的安装流程`
      );
      await load();
    } catch (error) {
      toast.error(`安装失败: ${error}`);
    }
  };

  return (
    <div className="flex flex-col gap-8 xl:flex-row">
      <section className="min-w-0 flex-1">
        <div className="mb-6 grid gap-4 md:grid-cols-3">
          <SummaryCard
            title="工具总数"
            value={`${tools.length}`}
            detail={`${installedCount} 个已安装`}
          />
          <SummaryCard
            title="可自动配置"
            value={`${configurableCount}`}
            detail="会优先接入桌面端、Agents 与 CLI"
          />
          <SummaryCard
            title="待安装"
            value={`${needsInstallCount}`}
            detail="缺失工具可在这里发起安装"
          />
        </div>

        {loading ? (
          <div className="flex flex-col items-center justify-center py-24 text-[#8f7d6a]">
            <Loader2 className="mx-auto h-10 w-10 animate-spin text-[#d07347]" />
            <p className="mt-4 text-xl font-semibold">正在检测已安装的工具...</p>
          </div>
        ) : (
          <>
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
              const det = detection.find(
                item => item.tool_id === tool.id
              );
              const active = tool.id === selectedToolId;
              const meta = toolDisplayMeta(tool);
              return (
                <StudioToolCard
                  key={tool.id}
                  tool={tool}
                  detection={det}
                  meta={meta}
                  active={active}
                  selectedBindingLabel={
                    binding && active ? binding.model_name ?? '-' : '-'
                  }
                  onSelect={() => {
                    setSelectedToolId(tool.id);
                    setSelectedProviderId('');
                    setSelectedModelId('');
                  }}
                />
              );
            })}
            {installedTools.length === 0 && (
              <StudioEmpty text="这个分类下还没有检测到已安装工具。" />
            )}
          </StudioToolSection>

          <StudioToolSection
            title={`可安装 · ${availableTools.length}`}
            subtitle="这些工具还没有安装，可以作为下一步接入目标。"
          >
            {availableTools.map(tool => {
              const det = detection.find(
                item => item.tool_id === tool.id
              );
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
            {availableTools.length === 0 && (
              <StudioEmpty text="这个分类下暂时没有新的可安装工具。" />
            )}
          </StudioToolSection>
        </div>
        </>
        )}
      </section>

      <aside className="w-full shrink-0 rounded-[32px] border border-[#e3d7c8] bg-[#fbf7ef] p-6 shadow-[0_18px_40px_rgba(92,70,44,0.08)] xl:w-[380px] 2xl:w-[420px]">
        {!selectedTool ? (
          <div className="grid min-h-[400px] place-items-center text-center text-[#8f7d6a]">
            <div>
              <PackageSearch className="mx-auto h-14 w-14 text-[#c6b6a4]" />
              <p className="mt-4 text-2xl font-semibold">选择要配置的工具</p>
            </div>
          </div>
        ) : (
          <div className="flex h-full flex-col">
            <div className="mb-6">
              <div className="text-sm tracking-[0.25em] text-[#9b8a78]">
                模型
              </div>
              <div className="mt-3 flex items-center justify-between gap-4">
                <div>
                  <h3 className="text-3xl font-black text-[#2f241c]">
                    {selectedMeta?.title ?? selectedTool.name}
                  </h3>
                  <p className="mt-2 text-base text-[#8d7d6d]">
                    当前绑定：
                    {binding
                      ? `${binding.provider_name ?? '-'} / ${
                          binding.model_name ?? '-'
                        }`
                      : '尚未绑定'}
                  </p>
                </div>
                <div
                  className={`rounded-full px-4 py-2 text-sm font-semibold ${
                    selectedDetection?.installed
                      ? 'bg-emerald-100 text-emerald-700'
                      : 'bg-amber-100 text-amber-700'
                  }`}
                >
                  {toolStatus(selectedDetection)}
                </div>
              </div>
              <div className="mt-4 flex flex-wrap gap-2">
                <Tag tone="slate">
                  {selectedMeta
                    ? appCategoryLabel(selectedMeta.category)
                    : '工具'}
                </Tag>
                <Tag
                  tone={
                    SUPPORTED_BINDING_TOOL_IDS.has(selectedTool.id)
                      ? 'green'
                      : 'amber'
                  }
                >
                  {SUPPORTED_BINDING_TOOL_IDS.has(selectedTool.id)
                    ? '已接入自动配置'
                    : '待接入自动配置'}
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
                <p className="text-xl font-semibold text-[#3a3028]">
                  这个工具还没有接入自动配置。
                </p>
                <p className="mt-3 text-base">
                  这次重构先把产品结构理顺，后续我们可以按工具逐个补适配。
                </p>
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
                  <div className="text-lg font-semibold text-[#5a4c40]">
                    可选厂商
                  </div>
                  <div className="flex items-center gap-3 text-sm text-[#8d7d6d]">
                    <span>上游直连</span>
                    <span className="h-8 w-14 rounded-full bg-[#ebe1d2]" />
                  </div>
                </div>
                <div className="space-y-4 overflow-y-auto">
                  {compatibleProviders.length > 0 ? (
                    compatibleProviders.map(provider => {
                    const active = provider.id === selectedProviderId;
                    return (
                      <button
                        key={provider.id}
                        onClick={() => {
                          setSelectedProviderId(provider.id);
                          const firstModel = models.find(
                            model =>
                              model.provider_id === provider.id
                          );
                          setSelectedModelId(firstModel?.id ?? '');
                        }}
                        className={`flex w-full items-start gap-4 rounded-[24px] border px-5 py-5 text-left transition ${
                          active
                            ? 'border-[#d07347] bg-white'
                            : 'border-[#eadfce] bg-white/70 hover:bg-white'
                        }`}
                      >
                        <div
                          className={`mt-2 h-5 w-5 rounded-full border ${
                            active
                              ? 'border-[#d07347] bg-[#d07347]'
                              : 'border-[#d5c6b2]'
                          }`}
                        />
                        <div className="min-w-0">
                          <div className="text-2xl font-bold text-[#2f241c]">
                            {providerDisplayName(
                              provider,
                              duplicateNames
                            )}
                          </div>
                          <div className="mt-2 text-base text-[#8d7d6d]">
                            {provider.api_base.replace(
                              /^https?:\/\//,
                              ''
                            )}
                          </div>
                          <div className="mt-3 flex flex-wrap gap-2">
                            <Tag tone="slate">
                              {provider.anthropic_mode
                                ? 'Anthropic'
                                : 'OpenAI'}
                            </Tag>
                            <Tag
                              tone={
                                provider.has_api_key
                                  ? 'green'
                                  : 'rose'
                              }
                            >
                              {provider.has_api_key
                                ? '已保存 Key'
                                : '缺少 Key'}
                            </Tag>
                          </div>
                        </div>
                      </button>
                    );
                  })) : (
                    <div className="rounded-2xl border border-dashed border-[#d9cdbd] bg-[#fbf8f1] px-5 py-8 text-center">
                      <div className="text-xl font-bold text-[#43382e] mb-2">还没有厂商</div>
                      <p className="text-base text-[#9d8f81] mb-4">
                        请先在「模型中心」添加厂商并配置 API Key
                      </p>
                      <button
                        onClick={async () => {
                          // 通过 Tauri 事件通知父组件切换到模型中心页
                          try {
                            const { emit } = await import('@tauri-apps/api/event');
                            await emit('navigate', { section: 'models' });
                          } catch {
                            // fallback: 如果事件系统不可用，跳转到 hash 路由
                            window.location.hash = '#/models';
                          }
                        }}
                        className="rounded-xl bg-[#d07347] px-5 py-2.5 text-base font-semibold text-white transition hover:bg-[#c26438]"
                      >
                        前往模型中心
                      </button>
                    </div>
                  )}
                </div>

                <div className="mt-6 rounded-[24px] border border-[#eadfce] bg-white p-5">
                  <div className="text-lg font-semibold text-[#5a4c40]">
                    选择模型
                  </div>
                  {selectedTool.id === 'aider-cli' && (
                    <p className="mt-2 text-sm text-[#9d8f81]">
                      Aider
                      当前只支持OpenAI兼容厂商，因此这里会自动过滤Anthropic兼容源。
                    </p>
                  )}
                  <select
                    value={selectedModelId}
                    onChange={event =>
                      setSelectedModelId(event.target.value)
                    }
                    className="mt-4 w-full rounded-2xl border border-[#dfd2c1] bg-[#fcfaf6] px-4 py-4 text-lg outline-none focus:border-[#d07347]"
                  >
                    <option value="">请选择模型</option>
                    {providerModels.map(model => (
                      <option key={model.id} value={model.id}>
                        {model.name}
                      </option>
                    ))}
                  </select>
                  {selectedProvider &&
                    !selectedProvider.has_api_key && (
                      <p className="mt-4 text-base text-[#c15f44]">
                        当前厂商还没有保存 API
                        Key，请先去模型中心编辑厂商。
                      </p>
                    )}
                  {selectedTool.config_path && (
                    <p className="mt-4 text-sm text-[#9d8f81]">
                      配置将写入：{selectedTool.config_path}
                    </p>
                  )}
                </div>

                <div className="mt-auto grid gap-4 pt-8">
                  <div className="rounded-[24px] border border-[#eadfce] bg-white px-5 py-4 text-[#7a6c5f]">
                    <div className="text-sm tracking-[0.2em] text-[#a18f7d]">
                      当前策略
                    </div>
                    <div className="mt-2 text-lg">
                      {selectedProvider
                        ? `${
                            providerDisplayName(
                              selectedProvider,
                              duplicateNames
                            )
                          } / ${
                            models.find(
                              model =>
                                model.id === selectedModelId
                            )?.name ?? '未选择模型'
                          }`
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
                      <div className="rounded-2xl border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
                        ⚠️ 保存配置后，该工具将锁定到此厂商/模型。如需切换，请回到此页面重新绑定。
                      </div>
                      <button
                        onClick={applyBinding}
                        disabled={
                          saving ||
                          !selectedProvider ||
                          !selectedModelId ||
                          !selectedProvider.has_api_key
                        }
                        className="rounded-3xl bg-[#d07347] px-8 py-5 text-2xl font-bold text-white transition hover:bg-[#c26438] disabled:cursor-not-allowed disabled:bg-[#e5d0c3]"
                      >
                        {saving ? '保存配置中...' : '保存模型配置'}
                      </button>
                      <button
                        onClick={async () => {
                          try {
                            await api.launchTool(selectedTool.id);
                          } catch (error) {
                            toast.error(`启动失败: ${error}`);
                          }
                        }}
                        className="rounded-3xl border border-[#d9cdbd] bg-white px-8 py-4 text-xl font-semibold text-[#57483c] transition hover:bg-[#f7f2ea]"
                      >
                        启动应用
                      </button>
                    </>
                  )}
                  <div className="grid grid-cols-2 gap-3 text-sm text-[#8d7d6d]">
                    <label className="flex items-center gap-2 cursor-pointer hover:text-[#6d5a4b]">
                      <input
                        type="checkbox"
                        checked={autoLaunch}
                        onChange={e => setAutoLaunch(e.target.checked)}
                        className="accent-[#d07347]"
                      />
                      绑定后自动启动
                    </label>
                    <label className="flex items-center gap-2 text-[#b0a296] cursor-not-allowed">
                      <input type="checkbox" disabled className="accent-[#d07347]" />
                      高级配置（即将推出）
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
