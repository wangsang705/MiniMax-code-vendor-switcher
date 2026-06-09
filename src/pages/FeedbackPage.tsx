import { useState } from 'react';
import {
  Bug,
  ClipboardCopy,
  FileText,
  MessageSquareText,
  Monitor,
  Server,
  Terminal,
  Wifi,
} from 'lucide-react';

interface LogEntry {
  time: string;
  level: 'info' | 'warn' | 'error';
  message: string;
  source: string;
}

const SAMPLE_LOGS: LogEntry[] = [
  { time: new Date(Date.now() - 120_000).toISOString(), level: 'info', message: '观景 VISTA v0.1.0 启动', source: 'app' },
  { time: new Date(Date.now() - 119_000).toISOString(), level: 'info', message: 'SQLite 数据库已连接（WAL 模式）', source: 'db' },
  { time: new Date(Date.now() - 118_000).toISOString(), level: 'info', message: 'Keyring 服务初始化', source: 'keyring' },
  { time: new Date(Date.now() - 117_000).toISOString(), level: 'info', message: '已注册 14 个工具配置写入器', source: 'config_writer' },
  { time: new Date(Date.now() - 116_000).toISOString(), level: 'info', message: '正在检测已安装工具...', source: 'detector' },
  { time: new Date(Date.now() - 115_000).toISOString(), level: 'info', message: '检测完成：18 个工具平台，5 个已安装', source: 'detector' },
  { time: new Date(Date.now() - 60_000).toISOString(), level: 'warn', message: 'Tools 表为空，执行初始化种子数据', source: 'db' },
  { time: new Date(Date.now() - 30_000).toISOString(), level: 'info', message: '已切换厂商: DeepSeek（deepseek-chat）', source: 'binding' },
  { time: new Date(Date.now() - 10_000).toISOString(), level: 'error', message: '启动 claude-code-cli 失败：执行文件未找到', source: 'launcher' },
];

const SYSINFO = {
  platform: typeof navigator !== 'undefined' ? navigator.platform : '-',
  userAgent: typeof navigator !== 'undefined' ? navigator.userAgent : '-',
  language: typeof navigator !== 'undefined' ? navigator.language : '-',
  online: typeof navigator !== 'undefined' ? navigator.onLine : false,
  appVersion: '0.1.0',
  tauriVersion: '2.0',
  rustVersion: '1.75+',
};

type FeedbackTab = 'logs' | 'info' | 'feedback';

export default function FeedbackPage() {
  const [activeTab, setActiveTab] = useState<FeedbackTab>('logs');
  const [logs] = useState<LogEntry[]>(SAMPLE_LOGS);
  const [feedbackText, setFeedbackText] = useState('');
  const [feedbackEmail, setFeedbackEmail] = useState('');
  const [feedbackSent, setFeedbackSent] = useState(false);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text).catch(console.error);
  };

  const exportLogs = () => {
    const text = logs
      .map(l => `[${l.time}] [${l.level.toUpperCase()}] [${l.source}] ${l.message}`)
      .join('\n');
    const blob = new Blob([text], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `vista-logs-${Date.now()}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const submitFeedback = () => {
    if (!feedbackText.trim()) return;
    setFeedbackSent(true);
    setTimeout(() => {
      setFeedbackSent(false);
      setFeedbackText('');
      setFeedbackEmail('');
    }, 3000);
  };

  const tabs: Array<{ id: FeedbackTab; label: string; icon: React.ComponentType<{ className?: string }> }> = [
    { id: 'logs', label: '运行日志', icon: FileText },
    { id: 'info', label: '系统信息', icon: Monitor },
    { id: 'feedback', label: '问题反馈', icon: MessageSquareText },
  ];

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center gap-4 border-b border-[#e5dacc] pb-4">
        {tabs.map(tab => {
          const Icon = tab.icon;
          const active = activeTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center gap-2 border-b-4 px-4 pb-4 text-xl font-semibold transition ${
                active
                  ? 'border-[#d07347] text-[#2f241c]'
                  : 'border-transparent text-[#6e6258] hover:text-[#3f3329]'
              }`}
            >
              <Icon className="h-5 w-5" />
              {tab.label}
            </button>
          );
        })}
      </div>

      {activeTab === 'logs' && (
        <div>
          <div className="mb-4 flex items-center justify-between">
            <div className="flex items-center gap-2 text-sm text-[#8d7d6d]">
              <Terminal className="h-4 w-4" />
              最近 {logs.length} 条日志
            </div>
            <button
              onClick={exportLogs}
              className="flex items-center gap-1 rounded-xl border border-[#e3d7c8] bg-[#fbf7ef] px-4 py-2 text-sm font-semibold text-[#5a4c40] transition hover:bg-white"
            >
              <ClipboardCopy className="h-4 w-4" />
              导出日志
            </button>
          </div>

          <div className="max-h-[500px] space-y-1 overflow-y-auto rounded-[20px] border border-[#e3d7c8] bg-[#1a1a1a] p-4 font-mono text-xs leading-6">
            {logs.length === 0 && (
              <div className="py-8 text-center text-[#666]">暂无日志</div>
            )}
            {logs.map((log, i) => (
              <div key={i} className="flex gap-3">
                <span className="shrink-0 text-[#555]">
                  {new Date(log.time).toLocaleTimeString()}
                </span>
                <span
                  className={`shrink-0 font-bold ${
                    log.level === 'error'
                      ? 'text-red-400'
                      : log.level === 'warn'
                        ? 'text-amber-400'
                        : 'text-emerald-400'
                  }`}
                >
                  [{log.level.toUpperCase()}]
                </span>
                <span className="shrink-0 text-[#777]">[{log.source}]</span>
                <span className="text-[#ccc]">{log.message}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {activeTab === 'info' && (
        <div className="grid gap-4 md:grid-cols-2">
          {[
            { label: '应用版本', value: SYSINFO.appVersion, icon: Bug },
            { label: 'Tauri 版本', value: SYSINFO.tauriVersion, icon: Server },
            { label: 'Rust 版本', value: SYSINFO.rustVersion, icon: Terminal },
            { label: '平台', value: SYSINFO.platform, icon: Monitor },
            { label: '语言', value: SYSINFO.language, icon: MessageSquareText },
            {
              label: '网络状态',
              value: SYSINFO.online ? '在线' : '离线',
              icon: Wifi,
              valueClass: SYSINFO.online ? 'text-emerald-600' : 'text-red-500',
            },
          ].map(({ label, value, icon: Icon, valueClass }) => (
            <div
              key={label}
              className="rounded-[24px] border border-[#e3d7c8] bg-[#fbf7ef] px-6 py-5"
            >
              <div className="flex items-center gap-3">
                <Icon className="h-5 w-5 text-[#9a8a79]" />
                <span className="text-sm text-[#9a8a79]">{label}</span>
              </div>
              <div className={`mt-2 text-lg font-semibold text-[#2f241c] ${valueClass ?? ''}`}>
                {value}
              </div>
            </div>
          ))}

          <div className="col-span-full rounded-[24px] border border-[#e3d7c8] bg-[#fbf7ef] px-6 py-5">
            <div className="flex items-center gap-3">
              <Terminal className="h-5 w-5 text-[#9a8a79]" />
              <span className="text-sm text-[#9a8a79]">User Agent</span>
            </div>
            <div className="mt-2 break-all text-sm text-[#6d5a4b]">{SYSINFO.userAgent}</div>
            <button
              onClick={() => copyToClipboard(SYSINFO.userAgent)}
              className="mt-3 flex items-center gap-1 text-xs font-semibold text-[#d07347] hover:text-[#c26438]"
            >
              <ClipboardCopy className="h-3 w-3" />
              复制
            </button>
          </div>
        </div>
      )}

      {activeTab === 'feedback' && (
        <div className="mx-auto max-w-xl">
          {feedbackSent ? (
            <div className="rounded-[28px] border border-emerald-200 bg-emerald-50 px-8 py-12 text-center">
              <div className="text-4xl mb-4">✅</div>
              <div className="text-xl font-bold text-emerald-800">感谢反馈！</div>
              <div className="mt-2 text-emerald-600">我们会认真对待每一条反馈</div>
            </div>
          ) : (
            <div className="rounded-[28px] border border-[#e3d7c8] bg-[#fbf7ef] p-8">
              <h3 className="mb-2 text-2xl font-black text-[#2f241c]">告诉我们你的想法</h3>
              <p className="mb-6 text-[#8d7d6d]">
                遇到问题或有建议？写下你的反馈，我们会尽快处理。
              </p>

              <div className="space-y-4">
                <div>
                  <label className="mb-1 block text-xs font-medium text-slate-500">邮箱（选填）</label>
                  <input
                    type="email"
                    value={feedbackEmail}
                    onChange={e => setFeedbackEmail(e.target.value)}
                    placeholder="your@email.com"
                    className="w-full rounded-xl border border-slate-200 bg-white px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-[#d07347]"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-xs font-medium text-slate-500">反馈内容</label>
                  <textarea
                    value={feedbackText}
                    onChange={e => setFeedbackText(e.target.value)}
                    placeholder="请描述你遇到的问题或改进建议..."
                    rows={5}
                    className="w-full resize-none rounded-xl border border-slate-200 bg-white px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-[#d07347]"
                  />
                </div>
                <button
                  onClick={submitFeedback}
                  disabled={!feedbackText.trim()}
                  className="w-full rounded-2xl bg-[#d07347] py-4 text-lg font-bold text-white transition hover:bg-[#c26438] disabled:bg-slate-300"
                >
                  提交反馈
                </button>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
