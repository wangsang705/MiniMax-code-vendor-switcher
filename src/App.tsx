import { useCallback, useEffect, useState } from 'react';
import { VendorList } from './components/VendorList';
import { VendorDialog } from './components/VendorDialog';
import { api, VendorInstance, DetectionResult, Tool, Provider, Model } from './api';

type TabId = 'vendors' | 'tools' | 'providers' | 'chat';

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
    { id: 'tools', label: '🛠 工具中心', icon: '' },
    { id: 'vendors', label: '📋 厂商管理（旧）', icon: '' },
    { id: 'providers', label: '🤖 模型中心', icon: '' },
    { id: 'chat', label: '💬 AI 助手', icon: '' },
  ];

  return (
    <div className="min-h-screen bg-gray-50 flex flex-col">
      {/* 顶栏 */}
      <header className="bg-white border-b border-gray-200 px-6 py-3 flex items-center justify-between">
        <h1 className="text-lg font-bold text-gray-900">⚡ AI Toolkit Hub</h1>
        <span className="text-xs text-gray-400">v2.0</span>
      </header>

      {/* Tab 导航 */}
      <div className="bg-white border-b border-gray-200 px-6">
        <nav className="flex gap-1 -mb-px">
          {tabs.map(tab => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`px-4 py-2.5 text-sm font-medium border-b-2 transition-colors ${
                activeTab === tab.id
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700'
              }`}
            >
              {tab.icon || tab.label}
              {tab.icon && <span className="ml-1">{tab.label}</span>}
            </button>
          ))}
        </nav>
      </div>

      {/* 内容区 */}
      <main className="flex-1 p-6 overflow-auto">
        {activeTab === 'tools' && <ToolHubPanel />}
        {activeTab === 'vendors' && (
          <div className="max-w-3xl mx-auto">
            <VendorList
              refreshKey={listKey}
              onAdd={() => { setEditing(null); setDialogOpen(true); }}
              onEdit={v => { setEditing(v); setDialogOpen(true); }}
              onChanged={handleSaved}
            />
            {dialogOpen && (
              <VendorDialog
                editing={editing}
                onClose={() => setDialogOpen(false)}
                onSaved={handleSaved}
              />
            )}
          </div>
        )}
        {activeTab === 'providers' && <ProviderModelPanel />}
        {activeTab === 'chat' && <AIChatPanel />}
      </main>
    </div>
  );
}

// ===== 工具中心面板 =====
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

  const handleLaunch = async (toolId: string) => {
    try {
      const pid = await api.launchTool(toolId);
      alert(`✅ 已启动 (PID: ${pid})`);
    } catch (e) { alert('启动失败: ' + e); }
  };

  const handleBind = async (toolId: string) => {
    try {
      const [providers, models] = await Promise.all([api.listProviders(), api.listModels()]);
      const pList = providers.map((p, i) => `${i}: ${p.name}`).join('\n');
      const pIdx = prompt(`选择厂商:\n${pList}`);
      if (pIdx === null) return;
      const provider = providers[parseInt(pIdx)];
      if (!provider) return alert('无效选择');

      const availModels = models.filter(m => m.provider_id === provider.id);
      const mList = availModels.map((m, i) => `${i}: ${m.name}`).join('\n');
      const mIdx = prompt(`选择模型:\n${mList}`);
      if (mIdx === null) return;
      const model = availModels[parseInt(mIdx)];
      if (!model) return alert('无效选择');

      const key = prompt(`输入 ${provider.name} 的 API Key:`);
      if (!key) return;

      await api.applyBinding(toolId, provider.id, model.id, key);
      alert(`✅ 已绑定 ${tools.find(t => t.id === toolId)?.name} → ${provider.name}/${model.name}`);
    } catch (e) { alert('绑定失败: ' + e); }
  };

  if (loading) return <div className="text-center py-12 text-gray-400">⏳ 检测中...</div>;

  const installed = tools.filter(t => detection.find(d => d.tool_id === t.id)?.installed);
  const notInstalled = tools.filter(t => !detection.find(d => d.tool_id === t.id)?.installed);

  return (
    <div className="max-w-4xl mx-auto">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-base font-semibold text-gray-800">已检测到 {installed.length}/{tools.length} 个工具</h2>
        <button onClick={load} className="text-xs px-3 py-1.5 border rounded hover:bg-gray-50">
          🔄 刷新
        </button>
      </div>

      <div className="space-y-2 mb-8">
        {installed.map(t => {
          const det = detection.find(d => d.tool_id === t.id)!;
          return (
            <div key={t.id} className="bg-white border rounded-lg p-4 flex items-center justify-between shadow-sm">
              <div className="flex items-center gap-3">
                <span className="w-2.5 h-2.5 rounded-full bg-green-500" />
                <div>
                  <div className="font-medium text-sm">{t.name}</div>
                  <div className="text-xs text-gray-400">
                    {det.install_type === 'cli' ? '💻 CLI' :
                     det.install_type === 'desktop' ? '🖥 桌面端' : '💻+🖥'}
                    {det.versions[0] && ` · ${det.versions[0]}`}
                  </div>
                </div>
              </div>
              <div className="flex gap-2">
                <button onClick={() => handleBind(t.id)}
                  className="text-xs px-3 py-1.5 bg-blue-50 text-blue-700 rounded hover:bg-blue-100">
                  🔗 绑定模型
                </button>
                <button onClick={() => handleLaunch(t.id)}
                  className="text-xs px-3 py-1.5 bg-green-50 text-green-700 rounded hover:bg-green-100">
                  🚀 启动
                </button>
              </div>
            </div>
          );
        })}
        {installed.length === 0 && <p className="text-sm text-gray-400 text-center py-8">未检测到已安装的工具</p>}
      </div>

      <h3 className="text-sm font-semibold text-gray-500 mb-2">未安装 ({notInstalled.length})</h3>
      <div className="space-y-2">
        {notInstalled.map(t => (
          <div key={t.id} className="bg-white border border-dashed rounded-lg p-4 flex items-center justify-between opacity-50">
            <div className="flex items-center gap-3">
              <span className="w-2.5 h-2.5 rounded-full bg-gray-300" />
              <div>
                <div className="font-medium text-sm text-gray-400">{t.name}</div>
                <div className="text-xs text-gray-400">
                  {t.category === 'cli' ? '命令行' : t.category === 'agent' ? 'AI Agent' : '桌面端'}
                </div>
              </div>
            </div>
            <span className="text-xs text-gray-300">—</span>
          </div>
        ))}
      </div>
    </div>
  );
}

// ===== 模型中心面板 =====
function ProviderModelPanel() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [models, setModels] = useState<Model[]>([]);

  useEffect(() => {
    Promise.all([api.listProviders(), api.listModels()]).then(([p, m]) => {
      setProviders(p);
      setModels(m);
    });
  }, []);

  return (
    <div className="max-w-4xl mx-auto">
      <div className="grid grid-cols-2 gap-6">
        <div>
          <h2 className="text-base font-semibold text-gray-800 mb-3">🤖 厂商</h2>
          <div className="space-y-2">
            {providers.map(p => (
              <div key={p.id} className="bg-white border rounded-lg p-4">
                <div className="font-medium text-sm">{p.name}</div>
                <div className="text-xs text-gray-500 mt-1 break-all">{p.api_base}</div>
                <div className="flex gap-2 mt-2">
                  <span className={`text-xs px-2 py-0.5 rounded ${p.anthropic_mode ? 'bg-blue-50 text-blue-700' : 'bg-gray-50 text-gray-500'}`}>
                    {p.anthropic_mode ? 'Anthropic 兼容' : 'OpenAI 格式'}
                  </span>
                  <span className="text-xs bg-gray-50 text-gray-500 px-2 py-0.5 rounded">
                    {models.filter(m => m.provider_id === p.id).length} 模型
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div>
          <h2 className="text-base font-semibold text-gray-800 mb-3">🧠 模型</h2>
          <div className="space-y-2 max-h-[600px] overflow-y-auto">
            {providers.map(p => (
              <div key={p.id}>
                <div className="text-xs font-semibold text-gray-400 mb-1 mt-3 first:mt-0">{p.name}</div>
                {models.filter(m => m.provider_id === p.id).map(m => (
                  <div key={m.id} className="bg-white border rounded-lg p-3 mb-1.5">
                    <div className="flex items-center justify-between">
                      <span className="font-medium text-sm">{m.name}</span>
                      <span className="text-xs text-gray-400">{(m.context_length / 1000).toFixed(0)}K ctx</span>
                    </div>
                    <div className="flex flex-wrap gap-1 mt-1.5">
                      {m.supports_reasoning && <Badge>推理</Badge>}
                      {m.supports_tool_call && <Badge>工具调用</Badge>}
                      {m.supports_vision && <Badge>视觉</Badge>}
                      {m.supports_attachment && <Badge>附件</Badge>}
                    </div>
                  </div>
                ))}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

function Badge({ children }: { children: string }) {
  return <span className="text-xs bg-purple-50 text-purple-700 px-1.5 py-0.5 rounded">{children}</span>;
}

// ===== AI 助手面板 =====
function AIChatPanel() {
  const [messages, setMessages] = useState<{ role: string; content: string }[]>([]);
  const [input, setInput] = useState('');

  const handleSend = () => {
    if (!input.trim()) return;
    setMessages(prev => [...prev, { role: 'user', content: input }]);
    setMessages(prev => [...prev, {
      role: 'assistant',
      content: '👋 AI 助手功能开发中！即将支持：\n\n• 流式对话\n• 使用你配置的模型\n• 一键安装未检测到的工具\n• 解答技术问题',
    }]);
    setInput('');
  };

  return (
    <div className="max-w-3xl mx-auto flex flex-col h-[calc(100vh-12rem)]">
      <div className="flex-1 overflow-y-auto space-y-3 mb-4">
        {messages.length === 0 && (
          <div className="text-center py-16 text-gray-400">
            <p className="text-3xl mb-3">💬</p>
            <p className="text-sm font-medium">AI 助手</p>
            <p className="text-xs mt-1">配置好厂商和模型后即可对话</p>
          </div>
        )}
        {messages.map((msg, i) => (
          <div key={i} className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}>
            <div className={`max-w-[70%] rounded-lg px-4 py-2.5 text-sm leading-relaxed ${
              msg.role === 'user'
                ? 'bg-blue-500 text-white'
                : 'bg-white border text-gray-800'
            }`}>
              {msg.content}
            </div>
          </div>
        ))}
      </div>

      <div className="flex gap-2 border-t pt-4">
        <input
          type="text"
          value={input}
          onChange={e => setInput(e.target.value)}
          onKeyDown={e => e.key === 'Enter' && handleSend()}
          placeholder="输入问题..."
          className="flex-1 border rounded-lg px-4 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <button onClick={handleSend}
          className="px-5 py-2.5 bg-blue-500 text-white rounded-lg text-sm hover:bg-blue-600">
          发送
        </button>
      </div>
    </div>
  );
}
