import { useEffect, useState } from 'react';
import { VendorList } from './components/VendorList';
import { VendorDialog } from './components/VendorDialog';
import { Button } from './components/ui/button';
import { api, VendorInstance } from './api';

export default function App() {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editing, setEditing] = useState<VendorInstance | null>(null);
  const [claudeInstalled, setClaudeInstalled] = useState<boolean | null>(null);

  useEffect(() => { api.isClaudeInstalled().then(setClaudeInstalled); }, []);

  return (
    <div className="min-h-screen bg-gray-50 p-6">
      <div className="max-w-3xl mx-auto">
        <header className="flex justify-between items-center mb-6">
          <h1 className="text-xl font-bold">⚡ MiniMax Code Vendor Switcher</h1>
          <div className="flex gap-2">
            <Button onClick={() => api.launchClaude()} disabled={claudeInstalled === false}>
              🚀 启动 MiniMax Code
            </Button>
          </div>
        </header>

        {claudeInstalled === false && (
          <div className="mb-4 p-3 bg-yellow-50 border border-yellow-200 rounded text-sm">
            未检测到 MiniMax Code CLI，请先安装后再启动。
          </div>
        )}

        <VendorList
          onAdd={() => { setEditing(null); setDialogOpen(true); }}
          onEdit={(v) => { setEditing(v); setDialogOpen(true); }}
        />
      </div>

      {dialogOpen && (
        <VendorDialog
          editing={editing}
          onClose={() => setDialogOpen(false)}
          onSaved={() => setDialogOpen(false)}
        />
      )}
    </div>
  );
}
