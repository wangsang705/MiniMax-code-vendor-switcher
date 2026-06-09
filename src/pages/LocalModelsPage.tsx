import { useState } from 'react';
import { BrainCircuit, Cpu, FlaskConical, Plug, Power, RefreshCw, Wifi } from 'lucide-react';
import { SummaryCard } from '../components/ui/SummaryCard';

type ConnectionStatus = 'idle' | 'testing' | 'online' | 'offline';

interface LocalEndpoint {
  name: string;
  url: string;
  status: ConnectionStatus;
  models: string[];
}

const PRESETS = [
  { name: 'Ollama', url: 'http://localhost:11434' },
  { name: 'LM Studio', url: 'http://localhost:1234/v1' },
  { name: 'vLLM', url: 'http://localhost:8000/v1' },
  { name: 'llama.cpp', url: 'http://localhost:8080/v1' },
];

export default function LocalModelsPage() {
  const [endpoints, setEndpoints] = useState<LocalEndpoint[]>([
    { name: 'Ollama', url: 'http://localhost:11434', status: 'idle', models: [] },
    { name: 'LM Studio', url: 'http://localhost:1234/v1', status: 'idle', models: [] },
  ]);
  const [customUrl, setCustomUrl] = useState('');
  const [customName, setCustomName] = useState('');

  const testConnection = async (index: number) => {
    setEndpoints(prev =>
      prev.map((ep, i) => (i === index ? { ...ep, status: 'testing' as const } : ep))
    );

    const ep = endpoints[index];
    try {
      const baseUrl = ep.url.replace(/\/v1\/?$/, '').replace(/\/$/, '');
      const resp = await fetch(`${baseUrl}/v1/models`, {
        signal: AbortSignal.timeout(5000),
      });
      if (resp.ok) {
        const data = await resp.json();
        const models: string[] = (data.data ?? data.models ?? []).map(
          (m: { id: string; name?: string; model?: string }) => m.id ?? m.name ?? m.model ?? ''
        ).filter(Boolean);
        setEndpoints(prev =>
          prev.map((ep, i) =>
            i === index ? { ...ep, status: 'online' as const, models } : ep
          )
        );
      } else {
        setEndpoints(prev =>
          prev.map((ep, i) => (i === index ? { ...ep, status: 'offline' as const } : ep))
        );
      }
    } catch {
      setEndpoints(prev =>
        prev.map((ep, i) => (i === index ? { ...ep, status: 'offline' as const } : ep))
      );
    }
  };

  const testAll = () => {
    endpoints.forEach((_, i) => testConnection(i));
  };

  const addCustom = () => {
    if (!customUrl.trim()) return;
    setEndpoints(prev => [
      ...prev,
      {
        name: customName.trim() || '自定义',
        url: customUrl.trim(),
        status: 'idle',
        models: [],
      },
    ]);
    setCustomUrl('');
    setCustomName('');
  };

  const removeEndpoint = (index: number) => {
    setEndpoints(prev => prev.filter((_, i) => i !== index));
  };

  const onlineCount = endpoints.filter(ep => ep.status === 'online').length;
  const totalModels = endpoints.reduce((sum, ep) => sum + ep.models.length, 0);

  return (
    <div className="space-y-6">
      <div className="grid gap-4 md:grid-cols-3">
        <SummaryCard
          title="已配置服务"
          value={`${endpoints.length}`}
          detail={`${onlineCount} 个在线`}
        />
        <SummaryCard
          title="可用模型"
          value={`${totalModels}`}
          detail="本地推理模型"
        />
        <SummaryCard
          title="推理引擎"
          value={onlineCount > 0 ? '运行中' : '未连接'}
          detail={onlineCount > 0 ? '本地推理可用' : '请配置本地服务'}
        />
      </div>

      <div className="flex flex-wrap items-center gap-4">
        <button
          onClick={testAll}
          className="flex items-center gap-2 rounded-xl bg-[#d8cfbf] px-6 py-3 text-lg font-semibold text-[#3a3128] transition hover:bg-[#cbc1b0]"
        >
          <RefreshCw className="h-5 w-5" />
          全部测试
        </button>
      </div>

      <div className="rounded-[28px] border border-[#e3d7c8] bg-[#fbf7ef] p-6">
        <h3 className="mb-4 flex items-center gap-3 text-xl font-bold text-[#2f241c]">
          <Plug className="h-6 w-6 text-[#d07347]" />
          添加本地端点
        </h3>

        <div className="mb-4 flex flex-wrap gap-2">
          {PRESETS.map(preset => (
            <button
              key={preset.name}
              onClick={() => {
                setCustomName(preset.name);
                setCustomUrl(preset.url);
              }}
              className="rounded-xl border border-[#eadfce] bg-white px-4 py-2 text-sm font-semibold text-[#5a4c40] transition hover:bg-[#f5efe6]"
            >
              {preset.name}
            </button>
          ))}
        </div>

        <div className="flex flex-wrap items-end gap-3">
          <div className="min-w-[140px] flex-1">
            <label className="mb-1 block text-xs font-medium text-slate-500">名称</label>
            <input
              type="text"
              value={customName}
              onChange={e => setCustomName(e.target.value)}
              placeholder="Ollama"
              className="w-full rounded-xl border border-slate-200 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-[#d07347]"
            />
          </div>
          <div className="min-w-[200px] flex-[2]">
            <label className="mb-1 block text-xs font-medium text-slate-500">API URL</label>
            <input
              type="text"
              value={customUrl}
              onChange={e => setCustomUrl(e.target.value)}
              placeholder="http://localhost:11434"
              className="w-full rounded-xl border border-slate-200 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-[#d07347]"
            />
          </div>
          <button
            onClick={addCustom}
            disabled={!customUrl.trim()}
            className="rounded-xl bg-[#d07347] px-5 py-2 text-sm font-semibold text-white transition hover:bg-[#c26438] disabled:bg-slate-300"
          >
            添加
          </button>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        {endpoints.map((ep, index) => (
          <div
            key={index}
            className={`rounded-[28px] border p-6 shadow-[0_18px_40px_rgba(92,70,44,0.08)] ${
              ep.status === 'online'
                ? 'border-[#d4e8d4] bg-gradient-to-br from-[#f2faf2] to-white'
                : ep.status === 'offline'
                  ? 'border-[#f5d8d0] bg-gradient-to-br from-[#fef2ef] to-white'
                  : 'border-[#e3d7c8] bg-[#fbf7ef]'
            }`}
          >
            <div className="mb-4 flex items-start justify-between gap-4">
              <div className="flex items-center gap-3">
                <div
                  className={`flex h-12 w-12 items-center justify-center rounded-2xl text-xl font-bold ${
                    ep.status === 'online'
                      ? 'bg-emerald-100 text-emerald-700'
                      : ep.status === 'offline'
                        ? 'bg-red-100 text-red-600'
                        : 'bg-[#ece5dc] text-[#9f9387]'
                  }`}
                >
                  {ep.name.slice(0, 2).toUpperCase()}
                </div>
                <div>
                  <div className="text-xl font-bold text-[#2f241c]">{ep.name}</div>
                  <div className="mt-0.5 text-sm text-[#9a8a79]">{ep.url}</div>
                </div>
              </div>
              <div className="flex items-center gap-2">
                {ep.status === 'online' && (
                  <span className="flex items-center gap-1 rounded-full bg-emerald-100 px-3 py-1 text-xs font-semibold text-emerald-700">
                    <Wifi className="h-3 w-3" />
                    在线
                  </span>
                )}
                {ep.status === 'offline' && (
                  <span className="flex items-center gap-1 rounded-full bg-red-100 px-3 py-1 text-xs font-semibold text-red-600">
                    <Power className="h-3 w-3" />
                    离线
                  </span>
                )}
                {ep.status === 'testing' && (
                  <span className="flex items-center gap-1 rounded-full bg-amber-100 px-3 py-1 text-xs font-semibold text-amber-700">
                    <RefreshCw className="h-3 w-3 animate-spin" />
                    检测中
                  </span>
                )}
                {ep.status === 'idle' && (
                  <span className="rounded-full bg-slate-100 px-3 py-1 text-xs font-semibold text-slate-500">
                    待检测
                  </span>
                )}
              </div>
            </div>

            <div className="space-y-2 text-sm text-[#7d6f63]">
              <div className="flex items-center gap-2">
                <Cpu className="h-4 w-4" />
                可用模型：{ep.models.length > 0 ? ep.models.join(', ') : '未检测'}
              </div>
            </div>

            <div className="mt-4 flex flex-wrap items-center gap-3">
              <button
                onClick={() => testConnection(index)}
                disabled={ep.status === 'testing'}
                className="flex items-center gap-1 rounded-full bg-[#d07347] px-4 py-1.5 text-xs font-semibold text-white transition hover:bg-[#c26438] disabled:opacity-50"
              >
                <FlaskConical className="h-3 w-3" />
                {ep.status === 'testing' ? '检测中...' : '测试连接'}
              </button>
              <button
                onClick={() => removeEndpoint(index)}
                className="rounded-full px-3 py-1.5 text-xs font-semibold text-[#c35c44] transition hover:bg-red-50"
              >
                删除
              </button>
            </div>

            {ep.models.length > 0 && (
              <div className="mt-4 flex flex-wrap gap-2">
                {ep.models.map(model => (
                  <span
                    key={model}
                    className="rounded-full bg-white/80 px-2.5 py-1 text-xs font-medium text-[#7f7265]"
                  >
                    {model}
                  </span>
                ))}
              </div>
            )}
          </div>
        ))}

        {endpoints.length === 0 && (
          <div className="col-span-full rounded-[28px] border border-dashed border-[#d9cdbd] bg-[#fbf8f1] px-8 py-12 text-center text-lg text-[#8f7d6a]">
            <BrainCircuit className="mx-auto mb-4 h-12 w-12 text-[#c6b6a4]" />
            添加本地推理服务（Ollama / LM Studio / vLLM）来管理本地模型
          </div>
        )}
      </div>
    </div>
  );
}
