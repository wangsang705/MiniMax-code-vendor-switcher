import { useCallback, useEffect, useState } from 'react';
import { AlertTriangle, CheckCircle2, Cpu, Download, HardDrive, RefreshCw, Terminal, Wifi, Wrench, XCircle } from 'lucide-react';
import { api, type DetectionResult, type Tool } from '../api';
import { SummaryCard } from '../components/ui/SummaryCard';

type ToolState = 'idle' | 'scanning' | 'installing';

interface ToolWithDetection {
  tool: Tool;
  detection?: DetectionResult;
}

export default function RepairPage() {
  const [tools, setTools] = useState<ToolWithDetection[]>([]);
  const [state, setState] = useState<ToolState>('idle');
  const [installingId, setInstallingId] = useState<string | null>(null);
  const [message, setMessage] = useState('');
  const [hasRun, setHasRun] = useState(false);

  const scan = useCallback(async () => {
    setState('scanning');
    setMessage('');
    try {
      const [toolList, detectionList] = await Promise.all([
        api.listTools(),
        api.detectInstalledTools(),
      ]);
      setTools(
        toolList.map(tool => ({
          tool,
          detection: detectionList.find(d => d.tool_id === tool.id),
        }))
      );
      setHasRun(true);
    } catch (err) {
      setMessage(`检测失败: ${err}`);
    } finally {
      setState('idle');
    }
  }, []);

  useEffect(() => {
    scan();
  }, [scan]);

  const installTool = async (toolId: string) => {
    setInstallingId(toolId);
    setMessage('');
    try {
      const result = await api.installTool(toolId);
      setMessage(`✅ ${result}`);
      await scan();
    } catch (err) {
      setMessage(`❌ 安装失败: ${err}`);
    } finally {
      setInstallingId(null);
    }
  };

  const installAllMissing = async () => {
    const missing = tools.filter(t => !t.detection?.installed);
    for (const { tool } of missing) {
      await installTool(tool.id);
    }
  };

  const installedCount = tools.filter(t => t.detection?.installed).length;
  const missingCount = tools.length - installedCount;
  const hasIssues = tools.some(
    t => t.detection?.installed && t.detection.install_type === 'cli' && !t.tool.launch_command
  );

  return (
    <div className="space-y-6">
      <div className="grid gap-4 md:grid-cols-3">
        <SummaryCard
          title="工具总数"
          value={`${tools.length}`}
          detail={`${installedCount} 个已安装 · ${missingCount} 个待安装`}
        />
        <SummaryCard
          title="安装状态"
          value={installedCount === tools.length && tools.length > 0 ? '全部就绪' : `${missingCount} 个缺失`}
          detail={installedCount === tools.length ? '所有工具已安装' : '部分工具需要安装'}
        />
        <SummaryCard
          title="上次检测"
          value={hasRun ? '已完成' : '未检测'}
          detail={hasRun ? `共 ${tools.length} 个工具` : '点击"重新检测"'}
        />
      </div>

      <div className="flex flex-wrap items-center gap-4">
        <button
          onClick={scan}
          disabled={state === 'scanning'}
          className="flex items-center gap-2 rounded-xl bg-[#d8cfbf] px-6 py-3 text-lg font-semibold text-[#3a3128] transition hover:bg-[#cbc1b0] disabled:opacity-50"
        >
          <RefreshCw className={`h-5 w-5 ${state === 'scanning' ? 'animate-spin' : ''}`} />
          {state === 'scanning' ? '检测中...' : '重新检测'}
        </button>

        {missingCount > 0 && (
          <button
            onClick={installAllMissing}
            disabled={installingId !== null}
            className="flex items-center gap-2 rounded-xl bg-[#d07347] px-6 py-3 text-lg font-semibold text-white transition hover:bg-[#c26438] disabled:opacity-50"
          >
            <Download className="h-5 w-5" />
            一键安装全部（{missingCount} 个）
          </button>
        )}
      </div>

      {message && (
        <div className="rounded-[20px] border border-[#e5d8cb] bg-white px-5 py-4 text-base text-[#6d5a4b]">
          {message}
        </div>
      )}

      {hasIssues && (
        <div className="rounded-[20px] border border-amber-200 bg-amber-50 px-5 py-4 text-base text-amber-800">
          <AlertTriangle className="mr-2 inline-block h-5 w-5" />
          部分已安装工具可能缺少启动命令配置，建议重新安装。
        </div>
      )}

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {tools.map(({ tool, detection }) => {
          const isInstalled = !!detection?.installed;
          const isInstalling = installingId === tool.id;
          const versions = detection?.versions ?? [];

          return (
            <div
              key={tool.id}
              className={`rounded-[28px] border p-6 shadow-[0_18px_40px_rgba(92,70,44,0.08)] ${
                isInstalled
                  ? 'border-[#d4e8d4] bg-gradient-to-br from-[#f2faf2] to-white'
                  : 'border-[#e8d9c5] bg-white'
              }`}
            >
              <div className="mb-4 flex items-start justify-between gap-4">
                <div className="flex items-center gap-3">
                  <div
                    className={`flex h-10 w-10 items-center justify-center rounded-xl text-xl font-bold ${
                      isInstalled ? 'bg-emerald-100 text-emerald-700' : 'bg-amber-100 text-amber-700'
                    }`}
                  >
                    {tool.name.slice(0, 1).toUpperCase()}
                  </div>
                  <div>
                    <div className="text-lg font-bold text-[#2f241c]">{tool.name}</div>
                    <div className="text-sm text-[#9a8a79]">
                      {tool.category === 'cli'
                        ? '命令行'
                        : tool.category === 'desktop'
                          ? '桌面应用'
                          : 'Agent'}
                    </div>
                  </div>
                </div>
                {isInstalled ? (
                  <CheckCircle2 className="h-6 w-6 text-emerald-500" />
                ) : (
                  <XCircle className="h-6 w-6 text-[#c35c44]" />
                )}
              </div>

              <div className="space-y-2 text-sm text-[#7d6f63]">
                <div className="flex items-center gap-2">
                  <HardDrive className="h-4 w-4" />
                  版本：{versions[0] ?? '-'}
                </div>
                <div className="flex items-center gap-2">
                  <Terminal className="h-4 w-4" />
                  启动命令：{tool.launch_command ?? '-'}
                </div>
                {tool.config_path && (
                  <div className="flex items-center gap-2">
                    <Cpu className="h-4 w-4" />
                    配置路径：{tool.config_path}
                  </div>
                )}
              </div>

              <div className="mt-4 flex flex-wrap items-center gap-3">
                <span
                  className={`rounded-full px-3 py-1 text-xs font-semibold ${
                    isInstalled
                      ? 'bg-emerald-100 text-emerald-700'
                      : 'bg-amber-100 text-amber-700'
                  }`}
                >
                  {isInstalled ? '已安装' : '未安装'}
                </span>
                {!isInstalled && (
                  <button
                    onClick={() => installTool(tool.id)}
                    disabled={isInstalling}
                    className="flex items-center gap-1 rounded-full bg-[#d07347] px-4 py-1.5 text-xs font-semibold text-white transition hover:bg-[#c26438] disabled:opacity-50"
                  >
                    {isInstalling ? (
                      <>
                        <RefreshCw className="h-3 w-3 animate-spin" />
                        安装中...
                      </>
                    ) : (
                      <>
                        <Download className="h-3 w-3" />
                        安装
                      </>
                    )}
                  </button>
                )}
              </div>
            </div>
          );
        })}

        {tools.length === 0 && state !== 'scanning' && (
          <div className="col-span-full rounded-[28px] border border-dashed border-[#d9cdbd] bg-[#fbf8f1] px-8 py-12 text-center text-lg text-[#8f7d6a]">
            <Wrench className="mx-auto mb-4 h-12 w-12 text-[#c6b6a4]" />
            点击"重新检测"扫描已安装的 AI 工具
          </div>
        )}
      </div>

      <div className="rounded-[28px] border border-[#e3d7c8] bg-[#fbf7ef] p-6">
        <h3 className="text-xl font-bold text-[#2f241c]">环境信息</h3>
        <div className="mt-4 grid gap-4 text-sm text-[#7d6f63] md:grid-cols-2">
          <div className="flex items-center gap-2">
            <Wifi className="h-4 w-4" />
            网络状态：{typeof navigator !== 'undefined' && navigator.onLine ? '在线' : '离线'}
          </div>
          <div className="flex items-center gap-2">
            <Terminal className="h-4 w-4" />
            平台：{typeof navigator !== 'undefined' ? navigator.platform : '-'}
          </div>
        </div>
      </div>
    </div>
  );
}
