import { useState } from 'react';
import { api, type Provider, type VendorPreset } from '../api';
import { ANTHROPIC_PRESET_IDS, CUSTOM_PROVIDER_PRESET } from '../lib/app-types';
import { slugifyProviderId } from '../lib/app-utils';
import { Field } from './ui/Field';
import { Modal } from './ui/Modal';
import { useToast } from '../hooks/use-toast';

export function ProviderEditorModal({
  presets,
  provider,
  onClose,
  onSaved,
}: {
  presets: VendorPreset[];
  provider?: Provider;
  onClose: () => void;
  onSaved: () => void;
}) {
  const toast = useToast();
  const [selectedPresetId, setSelectedPresetId] = useState<string>(
    provider ? CUSTOM_PROVIDER_PRESET : CUSTOM_PROVIDER_PRESET
  );
  const [form, setForm] = useState({
    id: provider?.id ?? '',
    name: provider?.name ?? '',
    api_base: provider?.api_base ?? '',
    anthropic_mode: provider?.anthropic_mode ?? true,
    api_key: '',
  });
  const [saving, setSaving] = useState(false);

  const applyPreset = (presetId: string) => {
    setSelectedPresetId(presetId);
    if (presetId === CUSTOM_PROVIDER_PRESET) return;
    const preset = presets.find(item => item.id === presetId);
    if (!preset) return;
    setForm(current => ({
      ...current,
      id: preset.id,
      name: preset.name,
      api_base: preset.api_base,
      anthropic_mode: ANTHROPIC_PRESET_IDS.has(preset.id) || preset.api_base.includes('/anthropic'),
    }));
  };

  const save = async () => {
    let generatedId = form.id.trim() || slugifyProviderId(form.name);
    if (!generatedId || !form.name.trim() || !form.api_base.trim()) return;
    setSaving(true);
    try {
      if (provider) {
        await api.updateProvider({
          id: provider.id,
          name: form.name.trim(),
          api_base: form.api_base.trim(),
          anthropic_mode: form.anthropic_mode,
          api_key: form.api_key || undefined,
        });
      } else {
        await api.createProvider({
          id: generatedId,
          name: form.name.trim(),
          api_base: form.api_base.trim(),
          anthropic_mode: form.anthropic_mode,
          api_key: form.api_key || undefined,
        });
      }
      onSaved();
    } catch (error) {
      const errMsg = String(error);
      if (!provider && errMsg.includes('已存在')) {
        try {
          generatedId = `${generatedId}-${Date.now().toString(36).slice(-4)}`;
          await api.createProvider({
            id: generatedId,
            name: form.name.trim(),
            api_base: form.api_base.trim(),
            anthropic_mode: form.anthropic_mode,
            api_key: form.api_key || undefined,
          });
          onSaved();
          return;
        } catch (_) {}
      }
      toast.error(errMsg);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Modal title={provider ? '编辑厂商' : '添加厂商'} onClose={onClose}>
      <div className="space-y-4">
        {!provider && (
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-500">使用预设</label>
            <select
              value={selectedPresetId}
              onChange={event => applyPreset(event.target.value)}
              className="w-full rounded-xl border border-slate-200 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-[#d07347]"
            >
              <option value={CUSTOM_PROVIDER_PRESET}>自定义厂商</option>
              {presets.map(preset => (
                <option key={preset.id} value={preset.id}>{preset.name}</option>
              ))}
            </select>
          </div>
        )}
        <Field
          label="厂商标识"
          value={form.id}
          onChange={value => setForm(current => ({ ...current, id: slugifyProviderId(value) }))}
          placeholder="留空时按名称自动生成"
        />
        <Field
          label="名称"
          value={form.name}
          onChange={value => setForm(current => ({ ...current, name: value }))}
          placeholder="例如：DeepSeek"
        />
        <Field
          label="API Base URL"
          value={form.api_base}
          onChange={value => setForm(current => ({ ...current, api_base: value }))}
          placeholder="https://api.deepseek.com/anthropic"
        />
        <div>
          <label className="mb-1 block text-xs font-medium text-slate-500">
            {provider ? 'API Key（留空不修改）' : 'API Key（建议直接填写）'}
          </label>
          <input
            type="password"
            value={form.api_key}
            onChange={event => setForm(current => ({ ...current, api_key: event.target.value }))}
            placeholder="sk-..."
            className="w-full rounded-xl border border-slate-200 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-[#d07347]"
          />
        </div>
        <label className="flex items-center gap-2 text-sm text-slate-700">
          <input
            type="checkbox"
            checked={form.anthropic_mode}
            onChange={event => setForm(current => ({ ...current, anthropic_mode: event.target.checked }))}
          />
          Anthropic 兼容模式
        </label>
      </div>
      <div className="mt-6 flex justify-end gap-2">
        <button onClick={onClose} className="px-4 py-2 text-sm text-slate-600 hover:text-slate-800">取消</button>
        <button
          onClick={save}
          disabled={saving}
          className="rounded-xl bg-[#d07347] px-5 py-2 text-sm font-semibold text-white hover:bg-[#c26438] disabled:bg-slate-300"
        >
          {saving ? '保存中...' : provider ? '保存' : '添加'}
        </button>
      </div>
    </Modal>
  );
}
