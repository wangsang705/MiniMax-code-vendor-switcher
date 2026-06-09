import { useState } from 'react';
import { api, type Model, type Provider } from '../api';
import { Field } from './ui/Field';
import { Modal } from './ui/Modal';
import { useToast } from '../hooks/use-toast';

export function ModelEditorModal({
  providers,
  model,
  onClose,
  onSaved,
}: {
  providers: Provider[];
  model?: Model;
  onClose: () => void;
  onSaved: () => void;
}) {
  const [providerId, setProviderId] = useState(model?.provider_id ?? providers[0]?.id ?? '');
  const [name, setName] = useState(model?.name ?? '');
  const [modelId, setModelId] = useState(model?.model_id ?? '');
  const [ctxLen, setCtxLen] = useState(String(model?.context_length ?? 128000));
  const [maxOut, setMaxOut] = useState(String(model?.max_output ?? 8192));
  const [saving, setSaving] = useState(false);
  const toast = useToast();

  const save = async () => {
    if (!providerId || !name.trim() || !modelId.trim()) return;
    setSaving(true);
    try {
      if (model) {
        await api.updateModel({
          id: model.id,
          provider_id: providerId,
          name: name.trim(),
          model_id: modelId.trim(),
          context_length: Number.parseInt(ctxLen, 10) || 128000,
          max_output: Number.parseInt(maxOut, 10) || 8192,
        });
      } else {
        await api.createModel({
          provider_id: providerId,
          name: name.trim(),
          model_id: modelId.trim(),
          context_length: Number.parseInt(ctxLen, 10) || 128000,
          max_output: Number.parseInt(maxOut, 10) || 8192,
        });
      }
      onSaved();
    } catch (error) {
      toast.error(`保存失败: ${error}`);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Modal title={model ? '编辑模型' : '添加模型'} onClose={onClose}>
      <div className="space-y-4">
        <div>
          <label className="mb-1 block text-xs font-medium text-slate-500">所属厂商</label>
          <select
            value={providerId}
            onChange={event => setProviderId(event.target.value)}
            className="w-full rounded-xl border border-slate-200 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-[#d07347]"
          >
            {providers.map(provider => (
              <option key={provider.id} value={provider.id}>
                {provider.name} ({provider.id})
              </option>
            ))}
          </select>
        </div>
        <Field label="显示名称" value={name} onChange={setName} placeholder="例如：DeepSeek Chat" />
        <Field label="模型 ID" value={modelId} onChange={setModelId} placeholder="例如：deepseek-chat" />
        <div className="grid grid-cols-2 gap-3">
          <Field label="上下文长度" value={ctxLen} onChange={setCtxLen} placeholder="128000" />
          <Field label="最大输出" value={maxOut} onChange={setMaxOut} placeholder="8192" />
        </div>
      </div>
      <div className="mt-6 flex justify-end gap-2">
        <button onClick={onClose} className="px-4 py-2 text-sm text-slate-600 hover:text-slate-800">取消</button>
        <button
          onClick={save}
          disabled={saving}
          className="rounded-xl bg-[#d07347] px-5 py-2 text-sm font-semibold text-white hover:bg-[#c26438] disabled:bg-slate-300"
        >
          {saving ? '保存中...' : model ? '保存' : '添加'}
        </button>
      </div>
    </Modal>
  );
}
