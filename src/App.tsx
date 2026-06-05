import { useCallback, useEffect, useState } from 'react';
import { VendorList } from './components/VendorList';
import { VendorDialog } from './components/VendorDialog';
import { Button } from './components/ui/button';
import { api, VendorInstance } from './api';

export default function App() {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editing, setEditing] = useState<VendorInstance | null>(null);
  const [claudeInstalled, setClaudeInstalled] = useState<boolean | null>(null);
  // VendorList 刷新的"信号量"：每次 dialog 保存/删除/应用成功都 +1，
  // 子组件 useEffect 依赖 refreshKey 触发重新拉取。
  const [listKey, setListKey] = useState(0);
  // 启动 Claude Code 的反馈（成功显示 PID，失败显示原因）
  const [launchError, setLaunchError] = useState<string | null>(null);
  const [launching, setLaunching] = useState(false);

  useEffect(() => { api.isClaudeInstalled().then(setClaudeInstalled); }, []);

  // dialog 保存完成 -> 关 dialog + 触发列表刷新
  const handleSaved = useCallback(() => {
    setDialogOpen(false);
    setListKey((k) => k + 1);
  }, []);

  const handleLaunch = async () => {
    setLaunchError(null);
    setLaunching(true);
    try {
      const pid = await api.launchClaude();
      setLaunchError(`已启动 (PID: ${pid})`);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setLaunchError(`启动失败: ${msg}`);
    } finally {
      setLaunching(false);
      // 5 秒后自动清空反馈
      setTimeout(() => setLaunchError(null), 5000);
    }
  };

  const launchFeedbackClass = launchError?.startsWith('启动失败')
    ? 'mb-4 p-3 border rounded text-sm bg-red-50 border-red-200 text-red-800'
    : 'mb-4 p-3 border rounded text-sm bg-green-50 border-green-200 text-green-800';

  return (
    <div className="min-h-screen bg-gray-50 p-6">
      <div className="max-w-3xl mx-auto">
        <header className="flex justify-between items-center mb-6">
          <h1 className="text-xl font-bold">⚡ MiniMax Code Vendor Switcher</h1>
          <div className="flex gap-2">
            <Button onClick={handleLaunch} disabled={claudeInstalled === false || launching}>
              {launching ? '启动中...' : '🚀 启动 MiniMax Code'}
            </Button>
          </div>
        </header>

        {claudeInstalled === false && (
          <div className="mb-4 p-3 bg-yellow-50 border border-yellow-200 rounded text-sm">
            未检测到 MiniMax Code CLI，请先安装后再启动。
          </div>
        )}

        {launchError && (
          <div className={launchFeedbackClass}>{launchError}</div>
        )}

        <VendorList
          refreshKey={listKey}
          onAdd={() => { setEditing(null); setDialogOpen(true); }}
          onEdit={(v) => { setEditing(v); setDialogOpen(true); }}
          onChanged={handleSaved}
        />
      </div>

      {dialogOpen && (
        <VendorDialog
          editing={editing}
          onClose={() => setDialogOpen(false)}
          onSaved={handleSaved}
        />
      )}
    </div>
  );
}
