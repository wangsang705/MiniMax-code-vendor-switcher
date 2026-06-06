import { useCallback, useEffect, useState, useRef } from 'react';
import { api, DetectionResult, Tool, Provider, Model } from './api';

type TabId = 'tools' | 'providers' | 'chat';

// ===== 主应用 =====
export default function App() {
  const [activeTab, setActiveTab] = useState<TabId>('tools');

  const tabs: { id: TabId; label: string; icon: string }[] = [
    { id: 'tools', label: '工具中心', icon: '🛠' },
    { id: 'providers', label: '模型中心', icon: '🧠' },
    { id: 'chat', label: 'AI 助手', icon: '💬' },
  ];

  return (
    <div className="h-screen flex flex-col bg-gradient-to-br from-slate-50 to-slate-100">
      <header className="bg-white/80 backdrop-blur border-b border-slate-200 px-6 py-3 flex items-center justify-between shrink-0">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-blue-500 to-indigo-600 flex items-center justify-center text-white text-sm font-bold shadow-sm">A</div>
          <div>
            <h1 className="text-base font-semibold text-slate-900">AI Toolkit Hub</h1>
            <p className="text-[10px] text-slate-400 leading-none mt-0.5">多工具 AI 模型管理中心</p>
          </div>
        </div>
        <span className="text-[10px] px-2 py-1 bg-slate-100 text-slate-500 rounded-full font-medium">v2.0</span>
      </header>

      <div className="bg-white/50 backdrop-blur border-b border-slate-200 px-6 shrink-0">
        <nav className="flex gap-1 -mb-px">
          {tabs.map(tab => (
            <button key={tab.id} onClick={() => setActiveTab(tab.id)}
              className={`flex items-center gap-1.5 px-4 py-2.5 text-sm font-medium border-b-2 transition-all ${
                activeTab === tab.id
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-slate-500 hover:text-slate-700 hover:border-slate-300'
              }`}>
              <span className="text-base">{tab.icon}</span>
              <span>{tab.label}</span>
            </button>
          ))}
        </nav>
      </div>

      <main className="flex-1 p-6 overflow-auto">
        <div className="max-w-5xl mx-auto">
          {activeTab === 'tools' && <ToolHubPanel />}
          {activeTab === 'providers' && <ProviderModelPanel />}
          {activeTab === 'chat' && <AIChatPanel />}
        </div>
      </main>
    </div>
  );
}

// ===== 工具中心 =====
function ToolHubPanel() {
  const [detection, setDetection] = useState<DetectionResult[]>([]);
  const [tools, setTools] = useState<Tool[]>([]);
  const [loading, setLoading] = useState(true);
  const [bindTarget, setBindTarget] = useState<Tool | null>(null);
  const [providerCount, setProviderCount] = useState(0);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const [d, t, p] = await Promise.all([api.detectInstalledTools(), api.listTools(), api.listProviders()]);
      setDetection(d); setTools(t); setProviderCount(p.length);
    } catch (e) { console.error(e); }
    finally { setLoading(false); }
  }, []);

  useEffect(() => { load(); }, [load]);

  if (loading) return <LoadingState />;

  const installed = tools.filter(t => detection.find(d => d.tool_id === t.id)?.installed);
  const notInstalled = tools.filter(t => !detection.find(d => d.tool_id === t.id)?.installed);

  return (
    <div className="space-y-6">
      {/* 首次使用引导 */}
      {providerCount === 0 && (
        <div className="bg-gradient-to-r from-blue-50 to-indigo-50 border border-blue-200 rounded-xl p-5">
          <div className="flex items-start gap-3">
            <span className="text-xl">👋</span>
            <div>
              <p className="text-sm font-semibold text-blue-800">欢迎使用 AI Toolkit Hub！</p>
              <p className="text-xs text-blue-600 mt-1">
                请先在 <strong>模型中心</strong> 添加厂商和模型，然后回到这里为工具绑定模型。
              </p>
            </div>
          </div>
        </div>
      )}

      {/* 统计卡片 */}
      {installed.length > 0 && (
        <div className="grid grid-cols-3 gap-4">
          <StatCard icon="🔍" label="检测工具" value={`${tools.length} 个`} color="blue" />
          <StatCard icon="✅" label="已安装" value={`${installed.length} 个`} color="green" />
          <StatCard icon="📦" label="待安装" value={`${notInstalled.length} 个`} color="amber" />
        </div>
      )}

      {/* 已安装工具 */}
      {installed.length > 0 && (
        <Section title="已安装工具" count={installed.length} icon="✅">
          <div className="grid gap-3">
            {installed.map(t => {
              const det = detection.find(d => d.tool_id === t.id)!;
              return <ToolCard key={t.id} tool={t} det={det} installed onBind={() => setBindTarget(t)} />;
            })}
          </div>
        </Section>
      )}

      {/* 未安装工具 */}
      <Section title="未安装" count={notInstalled.length} icon="📦" muted={installed.length > 0}>
        {notInstalled.length === 0 ? <EmptyState text="全部已安装 🎉" /> : (
          <div className="grid gap-2">
            {notInstalled.map(t => (
              <ToolCard key={t.id} tool={t} installed={false} />
            ))}
          </div>
        )}
      </Section>

      {/* 绑定弹窗 */}
      {bindTarget && (
        <BindDialog tool={bindTarget} onClose={() => setBindTarget(null)} onDone={() => { setBindTarget(null); load(); }} />
      )}
    </div>
  );
}

// ===== 绑定弹窗（自动使用厂商已保存的 Key） =====
function BindDialog({ tool, onClose, onDone }: { tool: Tool; onClose: () => void; onDone: () => void }) {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [models, setModels] = useState<Model[]>([]);
  const [selProvider, setSelProvider] = useState('');
  const [selModel, setSelModel] = useState('');
  const [apiKey, setApiKey] = useState('');
  const [saving, setSaving] = useState(false);
  const [hasStoredKey, setHasStoredKey] = useState(false);
  const [firstLoad, setFirstLoad] = useState(true);

  useEffect(() => {
    Promise.all([api.listProviders(), api.listModels()]).then(([p, m]) => {
      setProviders(p); setModels(m);
      if (p.length > 0) {
        setSelProvider(p[0].id);
        const avail = m.filter(x => x.provider_id === p[0].id);
        if (avail.length > 0) setSelModel(avail[0].id);
        // 尝试获取第一个厂商的已保存 Key
        api.getProviderKey(p[0].id).then(k => { setApiKey(k); setHasStoredKey(true); }).catch(() => { setHasStoredKey(false); });
      }
      setFirstLoad(false);
    });
  }, []);

  const onProviderChange = async (pid: string) => {
    setSelProvider(pid);
    const avail = models.filter(m => m.provider_id === pid);
    if (avail.length > 0) setSelModel(avail[0].id);
    try { const k = await api.getProviderKey(pid); setApiKey(k); setHasStoredKey(true); }
    catch { setApiKey(''); setHasStoredKey(false); }
  };

  const handleSave = async () => {
    if (!selProvider || !selModel || firstLoad) return;
    if (!hasStoredKey && !apiKey) return;
    setSaving(true);
    try {
      await api.applyBinding(tool.id, selProvider, selModel, hasStoredKey ? undefined : apiKey);
      onDone();
    } catch (e) { alert('绑定失败: ' + e); }
    finally { setSaving(false); }
  };

  const availModels = models.filter(m => m.provider_id === selProvider);

  return (
    <div className="fixed inset-0 bg-black/40 backdrop-blur-sm flex items-center justify-center z-50" onClick={onClose}>
      <div className="bg-white rounded-2xl shadow-xl w-full max-w-lg p-6" onClick={e => e.stopPropagation()}>
        <h3 className="text-base font-semibold text-slate-800 mb-1">为 {tool.name} 绑定模型</h3>
        <p className="text-xs text-slate-400 mb-5">厂商和模型已在模型中心配置，选择后即可绑定。</p>
        <div className="space-y-4">
          <div>
            <label className="text-xs font-medium text-slate-500 mb-1 block">厂商（可在模型中心添加和管理）</label>
            <select value={selProvider} onChange={e => onProviderChange(e.target.value)}
              className="w-full border border-slate-200 rounded-lg px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-blue-500">
              <option value="">选择厂商...</option>
              {providers.map(p => <option key={p.id} value={p.id}>{p.name}</option>)}
            </select>
          </div>
          <div>
            <label className="text-xs font-medium text-slate-500 mb-1 block">模型</label>
            <select value={selModel} onChange={e => setSelModel(e.target.value)}
              className="w-full border border-slate-200 rounded-lg px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-blue-500" disabled={!selProvider}>
              <option value="">选择模型...</option>
              {availModels.map(m => <option key={m.id} value={m.id}>{m.name}</option>)}
            </select>
          </div>
          {!hasStoredKey ? (
            <div>
              <label className="text-xs font-medium text-slate-500 mb-1 block">API Key（首次绑定需输入，会自动保存）</label>
              <input type="password" value={apiKey} onChange={e => setApiKey(e.target.value)}
                placeholder="sk-..." className="w-full border border-slate-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500" />
            </div>
          ) : (
            <p className="text-xs text-emerald-600">✅ 已保存 API Key，可直接绑定</p>
          )}
        </div>
        <div className="flex justify-end gap-2 mt-6">
          <button onClick={onClose} className="px-4 py-2 text-sm text-slate-600 hover:text-slate-800">取消</button>
          <button onClick={handleSave} disabled={saving || !selProvider || !selModel || (!hasStoredKey && !apiKey)}
            className="px-5 py-2 bg-blue-500 text-white text-sm font-medium rounded-lg hover:bg-blue-600 disabled:bg-slate-300 transition-colors">
            {saving ? '绑定中...' : '确认绑定'}
          </button>
        </div>
      </div>
    </div>
  );
}

// ===== 工具卡片 =====
function ToolCard({ tool, det, installed, onBind }: {
  tool: Tool; det?: DetectionResult; installed?: boolean; onBind?: () => void;
}) {
  const [binding, setBinding] = useState<{ id: string; provider_name: string | null; model_name: string | null; } | null>(null);
  const [result, setResult] = useState<string | null>(null);

  useEffect(() => {
    if (installed) {
      api.getToolBinding(tool.id).then(b => {
        if (b) setBinding({ id: b.id, provider_name: b.provider_name, model_name: b.model_name });
      }).catch(() => {});
    }
  }, [tool.id, installed]);

  const handleLaunch = async () => {
    try { const pid = await api.launchTool(tool.id); setResult(`✅ 已启动 (PID: ${pid})`); setTimeout(() => setResult(null), 3000); }
    catch (e) { setResult('❌ ' + e); }
  };

  const handleUnbind = async () => {
    if (!binding || !confirm('确定解绑此工具的模型？')) return;
    try { await api.unbindTool(binding.id); setBinding(null); setResult('✅ 已解绑'); setTimeout(() => setResult(null), 2000); }
    catch (e) { setResult('❌ 解绑失败'); }
  };

  const handleInstall = async () => {
    setResult(null);
    try { const msg = await api.installTool(tool.id); setResult('✅ ' + msg); }
    catch (e) { setResult('❌ ' + e); }
  };

  return (
    <div className={`rounded-xl border transition-all ${
      installed
        ? 'bg-white border-slate-200 shadow-sm hover:shadow-md hover:border-slate-300'
        : 'bg-white/70 border-slate-200 hover:border-slate-300'
    }`}>
      <div className="p-4 flex items-center justify-between">
        <div className="flex items-center gap-3 min-w-0 flex-1">
          <div className={`w-2.5 h-2.5 rounded-full shrink-0 ${installed ? 'bg-emerald-500 shadow-sm shadow-emerald-200' : 'bg-slate-300'}`} />
          <div className="min-w-0">
            <div className="font-medium text-sm text-slate-800">{tool.name}</div>
            <div className="text-xs text-slate-400 mt-0.5">
              {installed && det ? (
                <>{det.install_type === 'cli' ? '💻 命令行工具' : det.install_type === 'desktop' ? '🖥 桌面应用' : '💻+🖥'}
                {det.versions[0] && ` · ${det.versions[0].replace('cli:', '')}`}</>
              ) : (
                <>{tool.category === 'cli' ? '💻 命令行' : tool.category === 'agent' ? '🤖 AI Agent' : '🖥 桌面端'}</>
              )}
              {binding && <span className="ml-2 text-blue-500 font-medium">🔗 {binding.provider_name}/{binding.model_name}</span>}
            </div>
          </div>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          {installed ? (
            <>
              {binding ? (
                <ActionButton onClick={handleUnbind} label="解绑" color="red" />
              ) : (
                <ActionButton onClick={onBind!} label="绑定模型" color="blue" />
              )}
              <ActionButton onClick={handleLaunch} label="启动" color="emerald" />
            </>
          ) : (
            <ActionButton onClick={handleInstall} label="一键安装" color="blue" />
          )}
        </div>
      </div>
      {result && (
        <div className={`px-4 pb-3 text-xs ${result.startsWith('✅') ? 'text-emerald-600' : 'text-red-500'} whitespace-pre-wrap break-words`}>
          {result}
        </div>
      )}
    </div>
  );
}

// ===== 模型中心 =====
function ProviderModelPanel() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [models, setModels] = useState<Model[]>([]);
  const [showAdd, setShowAdd] = useState(false);
  const [showAddModel, setShowAddModel] = useState(false);
  const [editProvider, setEditProvider] = useState<Provider | null>(null);
  const [newProvider, setNewProvider] = useState({ id: '', name: '', api_base: '', anthropic_mode: true, api_key: '' });

  const load = useCallback(() => {
    Promise.all([api.listProviders(), api.listModels()]).then(([p, m]) => { setProviders(p); setModels(m); }).catch(console.error);
  }, []);
  useEffect(() => { load(); }, [load]);

  const handleAddProvider = async () => {
    if (!newProvider.id || !newProvider.name || !newProvider.api_base) return;
    try {
      await api.createProvider({ ...newProvider, api_key: newProvider.api_key || undefined });
      setShowAdd(false); setNewProvider({ id: '', name: '', api_base: '', anthropic_mode: true, api_key: '' }); load();
    } catch (e) { alert('添加失败: ' + e); }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-base font-semibold text-slate-800">模型配置</h2>
        <button onClick={() => setShowAddModel(true)}
          className="flex items-center gap-1.5 px-4 py-2 bg-white border border-slate-200 text-sm font-medium rounded-lg hover:bg-slate-50 transition-colors mr-2">
          + 添加模型
        </button>
        <button onClick={() => setShowAdd(true)}
          className="flex items-center gap-1.5 px-4 py-2 bg-blue-500 text-white text-sm font-medium rounded-lg hover:bg-blue-600 transition-colors shadow-sm">
          + 添加厂商
        </button>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <div className="space-y-3">
          <h3 className="text-xs font-semibold text-slate-400 uppercase tracking-wider">厂商 · {providers.length}</h3>
          {providers.length === 0 ? <EmptyState text="暂无厂商，点击上方添加" /> : (
            <div className="space-y-2">
              {providers.map(p => (
                <div key={p.id} className="bg-white rounded-xl border border-slate-200 p-4 shadow-sm hover:shadow-md transition-shadow">
                  <div className="flex items-start justify-between">
                    <div className="min-w-0 flex-1">
                      <div className="font-medium text-sm text-slate-800">{p.name}</div>
                      <div className="text-[11px] text-slate-400 mt-1 font-mono break-all">{p.api_base}</div>
                    </div>
                    <div className="flex gap-1 shrink-0 ml-2">
                      <button onClick={() => setEditProvider(p)}
                        className="text-slate-300 hover:text-blue-400 transition-colors text-xs px-1.5 py-0.5 rounded hover:bg-blue-50">✎</button>
                      <button onClick={async () => { if (confirm('确定删除此厂商？')) { try { await api.deleteProvider(p.id); load(); } catch (e) { alert('删除失败'); } } }}
                        className="text-slate-300 hover:text-red-400 transition-colors text-xs px-1.5 py-0.5 rounded hover:bg-red-50">✕</button>
                    </div>
                  </div>
                  <div className="flex items-center gap-2 mt-3">
                    <span className={`text-[10px] px-2 py-0.5 rounded-full font-medium ${p.anthropic_mode ? 'bg-blue-50 text-blue-600' : 'bg-slate-100 text-slate-500'}`}>
                      {p.anthropic_mode ? 'Anthropic 兼容' : 'OpenAI 格式'}
                    </span>
                    <span className="text-[10px] px-2 py-0.5 rounded-full bg-purple-50 text-purple-600 font-medium">
                      {models.filter(m => m.provider_id === p.id).length} 个模型
                    </span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="space-y-3">
          <h3 className="text-xs font-semibold text-slate-400 uppercase tracking-wider">模型 · {models.length}</h3>
          <div className="space-y-2 max-h-[600px] overflow-y-auto">
            {providers.map(p => {
              const pModels = models.filter(m => m.provider_id === p.id);
              if (pModels.length === 0) return null;
              return (
                <div key={p.id}>
                  <div className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider mb-1.5 mt-3 first:mt-0">{p.name}</div>
                  {pModels.map(m => (
                    <div key={m.id} className="bg-white rounded-lg border border-slate-200 p-3 mb-1.5 hover:border-slate-300 transition-colors">
                      <div className="flex items-center justify-between">
                        <span className="font-medium text-sm text-slate-800">{m.name}</span>
                        <span className="text-[10px] text-slate-400 font-mono">{(m.context_length / 1000).toFixed(0)}K ctx</span>
                      </div>
                      <div className="flex flex-wrap gap-1 mt-1.5">
                        {m.supports_reasoning && <Badge color="purple">推理</Badge>}
                        {m.supports_tool_call && <Badge color="blue">工具调用</Badge>}
                        {m.supports_vision && <Badge color="emerald">视觉</Badge>}
                        {m.supports_attachment && <Badge color="amber">附件</Badge>}
                      </div>
                    </div>
                  ))}
                </div>
              );
            })}
            {models.length === 0 && <EmptyState text="暂未添加模型" />}
          </div>
        </div>
      </div>

      {/* 添加厂商弹窗 */}
      {showAdd && (
        <Modal title="添加厂商" onClose={() => setShowAdd(false)}>
          <div className="space-y-4">
            <Field label="厂商 ID" value={newProvider.id} onChange={v => setNewProvider(p => ({ ...p, id: v }))} placeholder="例: deepseek" />
            <Field label="名称" value={newProvider.name} onChange={v => setNewProvider(p => ({ ...p, name: v }))} placeholder="例: DeepSeek" />
            <Field label="API Base URL" value={newProvider.api_base} onChange={v => setNewProvider(p => ({ ...p, api_base: v }))} placeholder="https://api.deepseek.com/anthropic" />
            <div>
              <label className="text-xs font-medium text-slate-500 mb-1 block">API Key（选填，保存后绑定工具时自动使用）</label>
              <input type="password" value={newProvider.api_key} onChange={e => setNewProvider(p => ({ ...p, api_key: e.target.value }))}
                placeholder="sk-..." className="w-full border border-slate-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500" />
            </div>
            <label className="flex items-center gap-2 text-sm text-slate-600">
              <input type="checkbox" checked={newProvider.anthropic_mode} onChange={e => setNewProvider(p => ({ ...p, anthropic_mode: e.target.checked }))} className="rounded border-slate-300" />
              Anthropic 兼容模式
            </label>
          </div>
          <div className="flex justify-end gap-2 mt-6">
            <button onClick={() => setShowAdd(false)} className="px-4 py-2 text-sm text-slate-600 hover:text-slate-800">取消</button>
            <button onClick={handleAddProvider} className="px-5 py-2 bg-blue-500 text-white text-sm font-medium rounded-lg hover:bg-blue-600">添加</button>
          </div>
        </Modal>
      )}

      {/* 添加模型弹窗 */}
      {showAddModel && <AddModelDialog onClose={() => setShowAddModel(false)} onDone={() => { setShowAddModel(false); load(); }} providers={providers} />}
      {editProvider && <EditProviderDialog provider={editProvider} onClose={() => setEditProvider(null)} onDone={() => { setEditProvider(null); load(); }} />}
    </div>
  );
}

// ===== 添加模型弹窗 =====
function AddModelDialog({ onClose, onDone, providers }: { onClose: () => void; onDone: () => void; providers: Provider[] }) {
  const [providerId, setProviderId] = useState(providers[0]?.id || '');
  const [name, setName] = useState('');
  const [modelId, setModelId] = useState('');
  const [ctxLen, setCtxLen] = useState('128000');
  const [maxOut, setMaxOut] = useState('8192');
  const [saving, setSaving] = useState(false);

  const handleSave = async () => {
    if (!providerId || !name || !modelId) return;
    setSaving(true);
    try {
      await api.createModel({
        provider_id: providerId, name, model_id: modelId,
        context_length: parseInt(ctxLen) || 128000,
        max_output: parseInt(maxOut) || 8192,
      });
      onDone();
    } catch (e) { alert('添加失败: ' + e); }
    finally { setSaving(false); }
  };

  return (
    <Modal title="添加模型" onClose={onClose}>
      <div className="space-y-4">
        <div>
          <label className="text-xs font-medium text-slate-500 mb-1 block">所属厂商</label>
          <select value={providerId} onChange={e => setProviderId(e.target.value)}
            className="w-full border border-slate-200 rounded-lg px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-blue-500">
            {providers.map(p => <option key={p.id} value={p.id}>{p.name}</option>)}
          </select>
        </div>
        <Field label="显示名称" value={name} onChange={setName} placeholder="例: DeepSeek Chat" />
        <Field label="模型 ID" value={modelId} onChange={setModelId} placeholder="例: deepseek-chat" />
        <div className="grid grid-cols-2 gap-3">
          <Field label="上下文长度" value={ctxLen} onChange={setCtxLen} placeholder="128000" />
          <Field label="最大输出" value={maxOut} onChange={setMaxOut} placeholder="8192" />
        </div>
      </div>
      <div className="flex justify-end gap-2 mt-6">
        <button onClick={onClose} className="px-4 py-2 text-sm text-slate-600 hover:text-slate-800">取消</button>
        <button onClick={handleSave} disabled={saving}
          className="px-5 py-2 bg-blue-500 text-white text-sm font-medium rounded-lg hover:bg-blue-600 disabled:bg-slate-300">添加</button>
      </div>
    </Modal>
  );
}

// ===== 编辑厂商弹窗 =====
function EditProviderDialog({ provider, onClose, onDone }: { provider: Provider; onClose: () => void; onDone: () => void }) {
  const [name, setName] = useState(provider.name);
  const [apiBase, setApiBase] = useState(provider.api_base);
  const [anthropicMode, setAnthropicMode] = useState(provider.anthropic_mode);
  const [saving, setSaving] = useState(false);

  const handleSave = async () => {
    if (!name || !apiBase) return;
    setSaving(true);
    try {
      await api.updateProvider({ id: provider.id, name, api_base: apiBase, anthropic_mode: anthropicMode });
      onDone();
    } catch (e) { alert('编辑失败: ' + e); }
    finally { setSaving(false); }
  };

  return (
    <Modal title="编辑厂商" onClose={onClose}>
      <div className="space-y-4">
        <Field label="名称" value={name} onChange={setName} />
        <Field label="API Base URL" value={apiBase} onChange={setApiBase} />
        <label className="flex items-center gap-2 text-sm text-slate-600">
          <input type="checkbox" checked={anthropicMode} onChange={e => setAnthropicMode(e.target.checked)} className="rounded border-slate-300" />
          Anthropic 兼容模式
        </label>
      </div>
      <div className="flex justify-end gap-2 mt-6">
        <button onClick={onClose} className="px-4 py-2 text-sm text-slate-600 hover:text-slate-800">取消</button>
        <button onClick={handleSave} disabled={saving} className="px-5 py-2 bg-blue-500 text-white text-sm font-medium rounded-lg hover:bg-blue-600 disabled:bg-slate-300">保存</button>
      </div>
    </Modal>
  );
}

// ===== AI 助手 =====
const STORAGE_KEY = 'ai-toolkit-hub-chat-config';

function AIChatPanel() {
  const [messages, setMessages] = useState<{ role: string; content: string }[]>([]);
  const [input, setInput] = useState('');
  const [providers, setProviders] = useState<Provider[]>([]);
  const [models, setModels] = useState<Model[]>([]);
  const [selProvider, setSelProvider] = useState(() => localStorage.getItem(`${STORAGE_KEY}-provider`) || '');
  const [selModel, setSelModel] = useState(() => localStorage.getItem(`${STORAGE_KEY}-model`) || '');
  const [apiKey, setApiKey] = useState(() => localStorage.getItem(`${STORAGE_KEY}-apikey`) || '');
  const [sending, setSending] = useState(false);
  const msgEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    Promise.all([api.listProviders(), api.listModels()]).then(([p, m]) => {
      setProviders(p); setModels(m);
    }).catch(() => {});
  }, []);

  // 持久化配置
  useEffect(() => {
    localStorage.setItem(`${STORAGE_KEY}-provider`, selProvider);
    localStorage.setItem(`${STORAGE_KEY}-model`, selModel);
    localStorage.setItem(`${STORAGE_KEY}-apikey`, apiKey);
  }, [selProvider, selModel, apiKey]);

  useEffect(() => { msgEndRef.current?.scrollIntoView({ behavior: 'smooth' }); }, [messages, sending]);

  const onProviderChange = (pid: string) => {
    setSelProvider(pid);
    const avail = models.filter(m => m.provider_id === pid);
    if (avail.length > 0) setSelModel(avail[0].id);
  };

  const handleSend = async () => {
    if (!input.trim() || !selProvider || !selModel || !apiKey) return;
    const provider = providers.find(p => p.id === selProvider);
    const model = models.find(m => m.id === selModel);
    if (!provider || !model) return;
    const userMsg = { role: 'user', content: input };
    setMessages(prev => [...prev, userMsg]);
    setInput('');
    setSending(true);
    try {
      const resp = await api.chatSend({
        messages: [...messages, userMsg], api_base: provider.api_base,
        api_key: apiKey, model: model.model_id, anthropic_mode: provider.anthropic_mode,
      });
      setMessages(prev => [...prev, { role: 'assistant', content: resp.content }]);
    } catch (e) {
      setMessages(prev => [...prev, { role: 'assistant', content: '❌ 请求失败: ' + e }]);
    } finally { setSending(false); }
  };

  const availModels = models.filter(m => m.provider_id === selProvider);
  const ready = selProvider && selModel && apiKey;

  return (
    <div className="flex flex-col h-[calc(100vh-10rem)] bg-white rounded-2xl border border-slate-200 shadow-sm">
      <div className="flex gap-3 p-4 border-b border-slate-100 bg-slate-50/50 rounded-t-2xl">
        <select value={selProvider} onChange={e => onProviderChange(e.target.value)}
          className="flex-1 border border-slate-200 rounded-lg px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-blue-500">
          <option value="">选择厂商...</option>
          {providers.map(p => <option key={p.id} value={p.id}>{p.name}</option>)}
        </select>
        <select value={selModel} onChange={e => setSelModel(e.target.value)}
          className="flex-1 border border-slate-200 rounded-lg px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-blue-500"
          disabled={!selProvider}>
          <option value="">选择模型...</option>
          {availModels.map(m => <option key={m.id} value={m.id}>{m.name}</option>)}
        </select>
        <input type="password" value={apiKey} onChange={e => setApiKey(e.target.value)}
          placeholder="API Key"
          className="flex-1 border border-slate-200 rounded-lg px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 && (
          <div className="flex items-center justify-center h-full">
            <div className="text-center">
              <div className="w-14 h-14 rounded-2xl bg-gradient-to-br from-blue-500 to-indigo-600 flex items-center justify-center text-2xl mx-auto mb-3 shadow-lg shadow-blue-200">💬</div>
              <p className="text-sm font-medium text-slate-600">AI 助手</p>
              <p className="text-xs text-slate-400 mt-1">配置会自动保存，下次打开无需重新填写</p>
            </div>
          </div>
        )}
        {messages.map((msg, i) => (
          <div key={i} className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}>
            <div className={`max-w-[70%] rounded-2xl px-4 py-2.5 text-sm leading-relaxed ${
              msg.role === 'user' ? 'bg-blue-500 text-white rounded-br-md' : 'bg-slate-100 text-slate-800 rounded-bl-md'
            }`}>{msg.content}</div>
          </div>
        ))}
        {sending && (
          <div className="flex justify-start">
            <div className="bg-slate-100 rounded-2xl rounded-bl-md px-4 py-3 text-sm text-slate-400 flex items-center gap-2">
              <span className="w-1.5 h-1.5 bg-slate-400 rounded-full animate-bounce" style={{ animationDelay: '0ms' }} />
              <span className="w-1.5 h-1.5 bg-slate-400 rounded-full animate-bounce" style={{ animationDelay: '150ms' }} />
              <span className="w-1.5 h-1.5 bg-slate-400 rounded-full animate-bounce" style={{ animationDelay: '300ms' }} />
            </div>
          </div>
        )}
        <div ref={msgEndRef} />
      </div>

      <div className="p-4 border-t border-slate-100">
        <div className="flex gap-2">
          <input type="text" value={input} onChange={e => setInput(e.target.value)}
            onKeyDown={e => e.key === 'Enter' && !sending && handleSend()}
            disabled={!ready || sending}
            placeholder={!ready ? '请先选择厂商、模型并输入 API Key' : '输入问题，Enter 发送...'}
            className="flex-1 border border-slate-200 rounded-xl px-4 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-slate-50" />
          <button onClick={handleSend} disabled={!ready || sending}
            className="px-5 py-2.5 bg-blue-500 text-white rounded-xl text-sm font-medium hover:bg-blue-600 disabled:bg-slate-300 transition-colors shadow-sm">
            {sending ? '发送中...' : '发送'}
          </button>
        </div>
      </div>
    </div>
  );
}

// ===== 通用组件 =====

function Modal({ title, children, onClose }: { title: string; children: React.ReactNode; onClose: () => void }) {
  return (
    <div className="fixed inset-0 bg-black/40 backdrop-blur-sm flex items-center justify-center z-50" onClick={onClose}>
      <div className="bg-white rounded-2xl shadow-xl w-full max-w-md p-6" onClick={e => e.stopPropagation()}>
        <h3 className="text-base font-semibold text-slate-800 mb-5">{title}</h3>
        {children}
      </div>
    </div>
  );
}

function StatCard({ icon, label, value, color }: { icon: string; label: string; value: string; color: string }) {
  const colors: Record<string, string> = {
    blue: 'from-blue-500 to-indigo-600 shadow-blue-200',
    green: 'from-emerald-500 to-green-600 shadow-emerald-200',
    amber: 'from-amber-500 to-orange-600 shadow-amber-200',
  };
  return (
    <div className="bg-white rounded-xl border border-slate-200 p-4 shadow-sm">
      <div className="flex items-center gap-3">
        <div className={`w-10 h-10 rounded-xl bg-gradient-to-br ${colors[color]} flex items-center justify-center text-lg shadow-sm`}>{icon}</div>
        <div>
          <div className="text-xs text-slate-400">{label}</div>
          <div className="text-lg font-bold text-slate-800">{value}</div>
        </div>
      </div>
    </div>
  );
}

function Section({ title, count, icon, muted, children }: { title: string; count: number; icon: string; muted?: boolean; children: React.ReactNode }) {
  return (
    <div>
      <div className="flex items-center gap-2 mb-3">
        <span className="text-sm">{icon}</span>
        <h3 className={`text-sm font-semibold ${muted ? 'text-slate-400' : 'text-slate-700'}`}>{title}</h3>
        <span className={`text-[10px] px-1.5 py-0.5 rounded-full font-medium ${muted ? 'bg-slate-100 text-slate-400' : 'bg-slate-100 text-slate-500'}`}>{count}</span>
      </div>
      {children}
    </div>
  );
}

function ActionButton({ onClick, label, color, disabled }: { onClick: () => void; label: string; color: string; disabled?: boolean }) {
  const colors: Record<string, string> = {
    blue: 'bg-blue-50 text-blue-700 hover:bg-blue-100',
    emerald: 'bg-emerald-50 text-emerald-700 hover:bg-emerald-100',
    red: 'bg-red-50 text-red-700 hover:bg-red-100',
  };
  return (
    <button onClick={onClick} disabled={disabled}
      className={`text-xs px-3 py-1.5 rounded-lg font-medium transition-colors disabled:opacity-50 ${colors[color] || colors.blue}`}>
      {label}
    </button>
  );
}

function Badge({ color, children }: { color: string; children: string }) {
  const colors: Record<string, string> = {
    purple: 'bg-purple-50 text-purple-600',
    blue: 'bg-blue-50 text-blue-600',
    emerald: 'bg-emerald-50 text-emerald-600',
    amber: 'bg-amber-50 text-amber-600',
  };
  return <span className={`text-[10px] px-1.5 py-0.5 rounded-full font-medium ${colors[color]}`}>{children}</span>;
}

function Field({ label, value, onChange, placeholder }: { label: string; value: string; onChange: (v: string) => void; placeholder?: string }) {
  return (
    <div>
      <label className="text-xs font-medium text-slate-500 mb-1 block">{label}</label>
      <input type="text" value={value} onChange={e => onChange(e.target.value)} placeholder={placeholder}
        className="w-full border border-slate-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500" />
    </div>
  );
}

function LoadingState() {
  return (
    <div className="flex items-center justify-center py-24">
      <div className="text-center">
        <div className="w-10 h-10 border-2 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-3" />
        <p className="text-sm text-slate-400">扫描已安装工具...</p>
      </div>
    </div>
  );
}

function EmptyState({ text }: { text: string }) {
  return <p className="text-sm text-slate-400 py-8 text-center">{text}</p>;
}
