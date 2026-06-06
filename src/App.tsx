import { useCallback, useEffect, useState, useRef } from 'react';
import { VendorList } from './components/VendorList';
import { VendorDialog } from './components/VendorDialog';
import { api, VendorInstance, DetectionResult, Tool, Provider, Model } from './api';

type TabId = 'tools' | 'vendors' | 'providers' | 'chat';

// ===== 主应用 =====

export default function App() {
  const [activeTab, setActiveTab] = useState<TabId>('tools');

  // 旧版状态
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editing, setEditing] = useState<VendorInstance | null>(null);
  const [listKey, setListKey] = useState(0);
  const handleSaved = useCallback(() => {
    setDialogOpen(false);
    setListKey(k => k + 1);
  }, []);

  const tabs: { id: TabId; label: string; icon: string }[] = [
    { id: 'tools', label: '工具中心', icon: '🛠' },
    { id: 'vendors', label: '厂商管理', icon: '📋' },
    { id: 'providers', label: '模型中心', icon: '🧠' },
    { id: 'chat', label: 'AI 助手', icon: '💬' },
  ];

  return (
    <div className="h-screen flex flex-col bg-gradient-to-br from-slate-50 to-slate-100">
      {/* 顶栏 */}
      <header className="bg-white/80 backdrop-blur border-b border-slate-200 px-6 py-3 flex items-center justify-between shrink-0">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-blue-500 to-indigo-600 flex items-center justify-center text-white text-sm font-bold shadow-sm">
            A
          </div>
          <div>
            <h1 className="text-base font-semibold text-slate-900">AI Toolkit Hub</h1>
            <p className="text-[10px] text-slate-400 leading-none mt-0.5">多工具 AI 模型管理中心</p>
          </div>
        </div>
        <span className="text-[10px] px-2 py-1 bg-slate-100 text-slate-500 rounded-full font-medium">v2.0</span>
      </header>

      {/* Tab 导航 */}
      <div className="bg-white/50 backdrop-blur border-b border-slate-200 px-6 shrink-0">
        <nav className="flex gap-1 -mb-px">
          {tabs.map(tab => (
            <button key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center gap-1.5 px-4 py-2.5 text-sm font-medium border-b-2 transition-all ${
                activeTab === tab.id
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-slate-500 hover:text-slate-700 hover:border-slate-300'
              }`}
            >
              <span className="text-base">{tab.icon}</span>
              <span>{tab.label}</span>
            </button>
          ))}
        </nav>
      </div>

      {/* 内容区 */}
      <main className="flex-1 p-6 overflow-auto">
        <div className="max-w-5xl mx-auto">
          {activeTab === 'tools' && <ToolHubPanel />}
          {activeTab === 'vendors' && (
            <>
              <VendorList refreshKey={listKey}
                onAdd={() => { setEditing(null); setDialogOpen(true); }}
                onEdit={v => { setEditing(v); setDialogOpen(true); }}
                onChanged={handleSaved}
              />
              {dialogOpen && (
                <VendorDialog editing={editing}
                  onClose={() => setDialogOpen(false)} onSaved={handleSaved}
                />
              )}
            </>
          )}
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

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const [d, t] = await Promise.all([api.detectInstalledTools(), api.listTools()]);
      setDetection(d);
      setTools(t);
    } catch (e) { console.error(e); }
    finally { setLoading(false); }
  }, []);

  useEffect(() => { load(); }, [load]);

  if (loading) return <LoadingState />;

  const installed = tools.filter(t => detection.find(d => d.tool_id === t.id)?.installed);
  const notInstalled = tools.filter(t => !detection.find(d => d.tool_id === t.id)?.installed);

  return (
    <div className="space-y-6">
      {/* 统计卡片 */}
      <div className="grid grid-cols-3 gap-4">
        <StatCard icon="🔍" label="检测工具" value={`${tools.length} 个`} color="blue" />
        <StatCard icon="✅" label="已安装" value={`${installed.length} 个`} color="green" />
        <StatCard icon="📦" label="待安装" value={`${notInstalled.length} 个`} color="amber" />
      </div>

      {/* 已安装工具 */}
      <Section title="已安装工具" count={installed.length} icon="✅">
        {installed.length === 0 ? <EmptyState text="未检测到已安装的工具" /> : (
          <div className="grid gap-3">
            {installed.map(t => {
              const det = detection.find(d => d.tool_id === t.id)!;
              return <ToolCard key={t.id} tool={t} det={det} installed />;
            })}
          </div>
        )}
      </Section>

      {/* 未安装工具 */}
      <Section title="未安装" count={notInstalled.length} icon="📦" muted>
        {notInstalled.length === 0 ? <EmptyState text="全部已安装 🎉" /> : (
          <div className="grid gap-2">
            {notInstalled.map(t => (
              <ToolCard key={t.id} tool={t} installed={false} />
            ))}
          </div>
        )}
      </Section>
    </div>
  );
}

// ===== 工具卡片 =====

function ToolCard({ tool, det, installed }: {
  tool: Tool; det?: DetectionResult; installed: boolean;
}) {
  const [binding, setBinding] = useState(false);

  const handleBind = async () => {
    setBinding(true);
    try {
      const [providers, models] = await Promise.all([api.listProviders(), api.listModels()]);
      const pList = providers.map((p, i) => `${i}: ${p.name}`).join('\n');
      const pIdx = prompt(`选择厂商:\n${pList}`);
      if (pIdx === null) { setBinding(false); return; }
      const provider = providers[parseInt(pIdx)];
      if (!provider) { setBinding(false); return alert('无效选择'); }
      const availModels = models.filter(m => m.provider_id === provider.id);
      const mList = availModels.map((m, i) => `${i}: ${m.name}`).join('\n');
      const mIdx = prompt(`选择模型:\n${mList}`);
      if (mIdx === null) { setBinding(false); return; }
      const model = availModels[parseInt(mIdx)];
      if (!model) { setBinding(false); return alert('无效选择'); }
      const key = prompt(`输入 ${provider.name} 的 API Key:`);
      if (!key) { setBinding(false); return; }
      await api.applyBinding(tool.id, provider.id, model.id, key);
    } catch (e) { alert('绑定失败: ' + e); }
    finally { setBinding(false); }
  };

  const handleLaunch = async () => {
    try { const pid = await api.launchTool(tool.id); alert(`✅ 已启动 (PID: ${pid})`); }
    catch (e) { alert('启动失败: ' + e); }
  };

  return (
    <div className={`rounded-xl border transition-all ${
      installed
        ? 'bg-white border-slate-200 shadow-sm hover:shadow-md hover:border-slate-300'
        : 'bg-white/50 border-dashed border-slate-200 opacity-60'
    }`}>
      <div className="p-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className={`w-2.5 h-2.5 rounded-full ${installed ? 'bg-emerald-500 shadow-sm shadow-emerald-200' : 'bg-slate-300'}`} />
          <div>
            <div className="font-medium text-sm text-slate-800">{tool.name}</div>
            {installed && det ? (
              <div className="text-xs text-slate-400 mt-0.5">
                {det.install_type === 'cli' ? '💻 命令行工具' : det.install_type === 'desktop' ? '🖥 桌面应用' : '💻+🖥'}
                {det.versions[0] && ` · ${det.versions[0].replace('cli:', '')}`}
              </div>
            ) : (
              <div className="text-xs text-slate-400 mt-0.5">
                {tool.category === 'cli' ? '💻 命令行' : tool.category === 'agent' ? '🤖 AI Agent' : '🖥 桌面端'}
              </div>
            )}
          </div>
        </div>
        {installed ? (
          <div className="flex gap-2">
            <ActionButton onClick={handleBind} disabled={binding} label="绑定模型" color="blue" />
            <ActionButton onClick={handleLaunch} label="启动" color="emerald" />
          </div>
        ) : (
          <span className="text-xs text-slate-300 px-3">—</span>
        )}
      </div>
    </div>
  );
}

// ===== 模型中心 =====

function ProviderModelPanel() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [models, setModels] = useState<Model[]>([]);
  const [showAdd, setShowAdd] = useState(false);
  const [newProvider, setNewProvider] = useState({ id: '', name: '', api_base: '', anthropic_mode: true });

  const load = useCallback(() => {
    Promise.all([api.listProviders(), api.listModels()]).then(([p, m]) => {
      setProviders(p); setModels(m);
    }).catch(console.error);
  }, []);

  useEffect(() => { load(); }, [load]);

  const handleAddProvider = async () => {
    if (!newProvider.id || !newProvider.name || !newProvider.api_base) return;
    try {
      await api.createProvider(newProvider);
      setShowAdd(false);
      setNewProvider({ id: '', name: '', api_base: '', anthropic_mode: true });
      load();
    } catch (e) { alert('添加失败: ' + e); }
  };

  const handleDeleteProvider = async (id: string) => {
    if (!confirm('确定删除此厂商？')) return;
    try { await api.deleteProvider(id); load(); }
    catch (e) { alert('删除失败: ' + e); }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-base font-semibold text-slate-800">模型配置</h2>
        <button onClick={() => setShowAdd(true)}
          className="flex items-center gap-1.5 px-4 py-2 bg-blue-500 text-white text-sm font-medium rounded-lg hover:bg-blue-600 transition-colors shadow-sm">
          <span className="text-base">+</span> 添加厂商
        </button>
      </div>

      <div className="grid grid-cols-2 gap-6">
        {/* 厂商列表 */}
        <div className="space-y-3">
          <h3 className="text-xs font-semibold text-slate-400 uppercase tracking-wider">厂商 · {providers.length}</h3>
          <div className="space-y-2">
            {providers.map(p => (
              <div key={p.id} className="bg-white rounded-xl border border-slate-200 p-4 shadow-sm hover:shadow-md transition-shadow">
                <div className="flex items-start justify-between">
                  <div>
                    <div className="font-medium text-sm text-slate-800">{p.name}</div>
                    <div className="text-[11px] text-slate-400 mt-1 font-mono break-all">{p.api_base}</div>
                  </div>
                  <button onClick={() => handleDeleteProvider(p.id)}
                    className="text-slate-300 hover:text-red-400 transition-colors text-xs px-1.5 py-0.5 rounded hover:bg-red-50">
                    ✕
                  </button>
                </div>
                <div className="flex items-center gap-2 mt-3">
                  <span className={`text-[10px] px-2 py-0.5 rounded-full font-medium ${
                    p.anthropic_mode
                      ? 'bg-blue-50 text-blue-600'
                      : 'bg-slate-100 text-slate-500'
                  }`}>
                    {p.anthropic_mode ? 'Anthropic 兼容' : 'OpenAI 格式'}
                  </span>
                  <span className="text-[10px] px-2 py-0.5 rounded-full bg-purple-50 text-purple-600 font-medium">
                    {models.filter(m => m.provider_id === p.id).length} 个模型
                  </span>
                </div>
              </div>
            ))}
            {providers.length === 0 && <EmptyState text="暂无厂商，点击上方添加" />}
          </div>
        </div>

        {/* 模型列表 */}
        <div className="space-y-3">
          <h3 className="text-xs font-semibold text-slate-400 uppercase tracking-wider">模型 · {models.length}</h3>
          <div className="space-y-2 max-h-[600px] overflow-y-auto">
            {providers.map(p => {
              const pModels = models.filter(m => m.provider_id === p.id);
              if (pModels.length === 0) return null;
              return (
                <div key={p.id}>
                  <div className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider mb-1.5 mt-3 first:mt-0">
                    {p.name}
                  </div>
                  {pModels.map(m => (
                    <div key={m.id} className="bg-white rounded-lg border border-slate-200 p-3 mb-1.5 hover:border-slate-300 transition-colors">
                      <div className="flex items-center justify-between">
                        <span className="font-medium text-sm text-slate-800">{m.name}</span>
                        <span className="text-[10px] text-slate-400 font-mono">
                          {(m.context_length / 1000).toFixed(0)}K ctx
                        </span>
                      </div>
                      <div className="flex flex-wrap gap-1 mt-1.5">
                        {m.supports_reasoning && <ModelBadge color="purple">推理</ModelBadge>}
                        {m.supports_tool_call && <ModelBadge color="blue">工具调用</ModelBadge>}
                        {m.supports_vision && <ModelBadge color="emerald">视觉</ModelBadge>}
                        {m.supports_attachment && <ModelBadge color="amber">附件</ModelBadge>}
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

      {/* 添加厂商对话框 */}
      {showAdd && (
        <div className="fixed inset-0 bg-black/40 backdrop-blur-sm flex items-center justify-center z-50" onClick={() => setShowAdd(false)}>
          <div className="bg-white rounded-2xl shadow-xl w-full max-w-md p-6" onClick={e => e.stopPropagation()}>
            <h3 className="text-base font-semibold text-slate-800 mb-5">添加厂商</h3>
            <div className="space-y-4">
              <Field label="厂商 ID" value={newProvider.id} onChange={v => setNewProvider(p => ({ ...p, id: v }))} placeholder="例: deepseek" />
              <Field label="名称" value={newProvider.name} onChange={v => setNewProvider(p => ({ ...p, name: v }))} placeholder="例: DeepSeek" />
              <Field label="API Base URL" value={newProvider.api_base} onChange={v => setNewProvider(p => ({ ...p, api_base: v }))} placeholder="https://api.deepseek.com/anthropic" />
              <div className="flex items-center gap-2">
                <input type="checkbox" id="am" checked={newProvider.anthropic_mode}
                  onChange={e => setNewProvider(p => ({ ...p, anthropic_mode: e.target.checked }))}
                  className="rounded border-slate-300" />
                <label htmlFor="am" className="text-sm text-slate-600">Anthropic 兼容模式（使用 @ai-sdk/anthropic）</label>
              </div>
            </div>
            <div className="flex justify-end gap-2 mt-6">
              <button onClick={() => setShowAdd(false)}
                className="px-4 py-2 text-sm text-slate-600 hover:text-slate-800 transition-colors">取消</button>
              <button onClick={handleAddProvider}
                className="px-5 py-2 bg-blue-500 text-white text-sm font-medium rounded-lg hover:bg-blue-600 transition-colors">添加</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// ===== AI 助手 =====

function AIChatPanel() {
  const [messages, setMessages] = useState<{ role: string; content: string }[]>([]);
  const [input, setInput] = useState('');
  const [providers, setProviders] = useState<Provider[]>([]);
  const [models, setModels] = useState<Model[]>([]);
  const [selectedProvider, setSelectedProvider] = useState('');
  const [selectedModel, setSelectedModel] = useState('');
  const [apiKey, setApiKey] = useState('');
  const [sending, setSending] = useState(false);
  const msgEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    Promise.all([api.listProviders(), api.listModels()]).then(([p, m]) => {
      setProviders(p); setModels(m);
    }).catch(() => {});
  }, []);

  useEffect(() => { msgEndRef.current?.scrollIntoView({ behavior: 'smooth' }); }, [messages, sending]);

  const onProviderChange = (pid: string) => {
    setSelectedProvider(pid);
    const m = models.filter(x => x.provider_id === pid);
    if (m.length > 0) setSelectedModel(m[0].id);
  };

  const handleSend = async () => {
    if (!input.trim() || !selectedProvider || !selectedModel || !apiKey) return;
    const provider = providers.find(p => p.id === selectedProvider);
    const model = models.find(m => m.id === selectedModel);
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

  const availModels = models.filter(m => m.provider_id === selectedProvider);
  const ready = selectedProvider && selectedModel && apiKey;

  return (
    <div className="flex flex-col h-[calc(100vh-10rem)] bg-white rounded-2xl border border-slate-200 shadow-sm">
      {/* 配置栏 */}
      <div className="flex gap-3 p-4 border-b border-slate-100 bg-slate-50/50 rounded-t-2xl">
        <select value={selectedProvider} onChange={e => onProviderChange(e.target.value)}
          className="flex-1 border border-slate-200 rounded-lg px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-blue-500">
          <option value="">选择厂商...</option>
          {providers.map(p => <option key={p.id} value={p.id}>{p.name}</option>)}
        </select>
        <select value={selectedModel} onChange={e => setSelectedModel(e.target.value)}
          className="flex-1 border border-slate-200 rounded-lg px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-blue-500"
          disabled={!selectedProvider}>
          <option value="">选择模型...</option>
          {availModels.map(m => <option key={m.id} value={m.id}>{m.name}</option>)}
        </select>
        <input type="password" value={apiKey} onChange={e => setApiKey(e.target.value)}
          placeholder="API Key"
          className="flex-1 border border-slate-200 rounded-lg px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
      </div>

      {/* 消息区 */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 && (
          <div className="flex items-center justify-center h-full">
            <div className="text-center">
              <div className="w-14 h-14 rounded-2xl bg-gradient-to-br from-blue-500 to-indigo-600 flex items-center justify-center text-2xl mx-auto mb-3 shadow-lg shadow-blue-200">
                💬
              </div>
              <p className="text-sm font-medium text-slate-600">AI 助手已就绪</p>
              <p className="text-xs text-slate-400 mt-1 max-w-xs mx-auto">选择上方的厂商和模型，输入 API Key 后即可开始对话</p>
            </div>
          </div>
        )}
        {messages.map((msg, i) => (
          <div key={i} className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}>
            <div className={`max-w-[70%] rounded-2xl px-4 py-2.5 text-sm leading-relaxed ${
              msg.role === 'user'
                ? 'bg-blue-500 text-white rounded-br-md'
                : 'bg-slate-100 text-slate-800 rounded-bl-md'
            }`}>
              {msg.content}
            </div>
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

      {/* 输入区 */}
      <div className="p-4 border-t border-slate-100">
        <div className="flex gap-2">
          <input type="text" value={input} onChange={e => setInput(e.target.value)}
            onKeyDown={e => e.key === 'Enter' && !sending && handleSend()}
            disabled={!ready || sending}
            placeholder={!ready ? '请先选择厂商、模型并输入 API Key' : '输入问题，Enter 发送...'}
            className="flex-1 border border-slate-200 rounded-xl px-4 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-slate-50 disabled:cursor-not-allowed"
          />
          <button onClick={handleSend} disabled={!ready || sending}
            className="px-5 py-2.5 bg-blue-500 text-white rounded-xl text-sm font-medium hover:bg-blue-600 disabled:bg-slate-300 disabled:cursor-not-allowed transition-colors shadow-sm">
            {sending ? '发送中...' : '发送'}
          </button>
        </div>
      </div>
    </div>
  );
}

// ===== 可复用小组件 =====

function StatCard({ icon, label, value, color }: { icon: string; label: string; value: string; color: string }) {
  const colors: Record<string, string> = {
    blue: 'from-blue-500 to-indigo-600 shadow-blue-200',
    green: 'from-emerald-500 to-green-600 shadow-emerald-200',
    amber: 'from-amber-500 to-orange-600 shadow-amber-200',
  };
  return (
    <div className="bg-white rounded-xl border border-slate-200 p-4 shadow-sm">
      <div className="flex items-center gap-3">
        <div className={`w-10 h-10 rounded-xl bg-gradient-to-br ${colors[color]} flex items-center justify-center text-lg shadow-sm`}>
          {icon}
        </div>
        <div>
          <div className="text-xs text-slate-400">{label}</div>
          <div className="text-lg font-bold text-slate-800">{value}</div>
        </div>
      </div>
    </div>
  );
}

function Section({ title, count, icon, muted, children }: {
  title: string; count: number; icon: string; muted?: boolean; children: React.ReactNode;
}) {
  return (
    <div>
      <div className="flex items-center gap-2 mb-3">
        <span className="text-sm">{icon}</span>
        <h3 className={`text-sm font-semibold ${muted ? 'text-slate-400' : 'text-slate-700'}`}>{title}</h3>
        <span className={`text-[10px] px-1.5 py-0.5 rounded-full font-medium ${
          muted ? 'bg-slate-100 text-slate-400' : 'bg-slate-100 text-slate-500'
        }`}>{count}</span>
      </div>
      {children}
    </div>
  );
}

function ActionButton({ onClick, label, color, disabled }: {
  onClick: () => void; label: string; color: string; disabled?: boolean;
}) {
  const colors: Record<string, string> = {
    blue: 'bg-blue-50 text-blue-700 hover:bg-blue-100',
    emerald: 'bg-emerald-50 text-emerald-700 hover:bg-emerald-100',
  };
  return (
    <button onClick={onClick} disabled={disabled}
      className={`text-xs px-3 py-1.5 rounded-lg font-medium transition-colors disabled:opacity-50 ${colors[color]}`}>
      {label}
    </button>
  );
}

function ModelBadge({ color, children }: { color: string; children: string }) {
  const colors: Record<string, string> = {
    purple: 'bg-purple-50 text-purple-600',
    blue: 'bg-blue-50 text-blue-600',
    emerald: 'bg-emerald-50 text-emerald-600',
    amber: 'bg-amber-50 text-amber-600',
  };
  return <span className={`text-[10px] px-1.5 py-0.5 rounded-full font-medium ${colors[color]}`}>{children}</span>;
}

function Field({ label, value, onChange, placeholder }: {
  label: string; value: string; onChange: (v: string) => void; placeholder?: string;
}) {
  return (
    <div>
      <label className="text-xs font-medium text-slate-500 mb-1 block">{label}</label>
      <input type="text" value={value} onChange={e => onChange(e.target.value)}
        placeholder={placeholder}
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
