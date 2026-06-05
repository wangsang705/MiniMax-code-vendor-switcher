import { useEffect, useState } from 'react';
import { api, VendorInstance, VendorPreset } from '../api';
import { Button } from './ui/button';
import { Card } from './ui/card';
import { Input } from './ui/input';
import { Label } from './ui/label';

export function VendorDialog({
  editing,
  onClose,
  onSaved,
}: {
  editing: VendorInstance | null;
  onClose: () => void;
  onSaved: () => void;
}) {
  const [presets, setPresets] = useState<VendorPreset[]>([]);
  const [name, setName] = useState(editing?.name ?? '');
  const [apiBase, setApiBase] = useState(editing?.api_base ?? '');
  const [model, setModel] = useState(editing?.model ?? '');
  const [apiKey, setApiKey] = useState('');
  const [presetId, setPresetId] = useState<string | null>(editing?.preset_id ?? null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    api.listPresets().then(setPresets);
  }, []);

  const choosePreset = (id: string) => {
    if (id === '__custom__') {
      setPresetId(null);
      return;
    }
    setPresetId(id);
    const p = presets.find((x) => x.id === id);
    if (p) {
      setName(p.name);
      setApiBase(p.api_base);
      setModel(p.default_model);
    }
  };

  const save = async () => {
    if (!name || !apiBase || !model) {
      alert('请填写名称、API Base 和模型');
      return;
    }
    if (!editing && !apiKey) {
      alert('请填写 API Key');
      return;
    }
    setSaving(true);
    try {
      if (editing) {
        await api.updateVendor({
          id: editing.id,
          name,
          api_base: apiBase,
          model,
          api_key: apiKey || undefined,
        });
      } else {
        await api.createVendor({
          preset_id: presetId,
          name,
          api_base: apiBase,
          model,
          api_key: apiKey,
        });
      }
      onSaved();
    } catch (e) {
      alert('保存失败: ' + e);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <Card className="w-full max-w-md p-6 bg-white">
        <h2 className="text-lg font-semibold mb-4">
          {editing ? '编辑厂商' : '添加厂商'}
        </h2>

        {!editing && (
          <div className="mb-4">
            <Label>选择预设</Label>
            <select
              className="w-full mt-1 border rounded px-2 py-1.5 text-sm"
              onChange={(e) => choosePreset(e.target.value)}
              defaultValue=""
            >
              <option value="" disabled>请选择...</option>
              {presets.map((p) => (
                <option key={p.id} value={p.id}>{p.name}</option>
              ))}
              <option value="__custom__">自定义 OpenAI 兼容端点</option>
            </select>
          </div>
        )}

        <div className="space-y-3">
          <div>
            <Label>名称</Label>
            <Input value={name} onChange={(e) => setName(e.target.value)} />
          </div>
          <div>
            <Label>API Base URL</Label>
            <Input value={apiBase} onChange={(e) => setApiBase(e.target.value)} placeholder="https://..." />
          </div>
          <div>
            <Label>模型名</Label>
            <Input value={model} onChange={(e) => setModel(e.target.value)} />
          </div>
          <div>
            <Label>{editing ? 'API Key（留空表示不修改）' : 'API Key'}</Label>
            <Input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="sk-..."
            />
          </div>
        </div>

        <div className="flex justify-end gap-2 mt-6">
          <Button variant="outline" onClick={onClose} disabled={saving}>取消</Button>
          <Button onClick={save} disabled={saving}>{saving ? '保存中...' : '保存'}</Button>
        </div>
      </Card>
    </div>
  );
}
