import type { DetectionResult, Tool } from '../api';
import { appCategoryLabel, toolStatus, trimPath, type ToolDisplayMeta } from '../lib/app-utils';
import { SUPPORTED_BINDING_TOOL_IDS } from '../lib/app-types';
import { Tag } from './ui/Tag';

export function StudioToolCard({
  tool,
  detection,
  meta,
  active,
  selectedBindingLabel,
  onSelect,
}: {
  tool: Tool;
  detection?: DetectionResult;
  meta: ToolDisplayMeta;
  active: boolean;
  selectedBindingLabel: string;
  onSelect: () => void;
}) {
  const isInstalled = !!detection?.installed;

  return (
    <button
      onClick={onSelect}
      className={`min-h-[250px] rounded-[28px] border p-6 text-left shadow-[0_18px_40px_rgba(92,70,44,0.08)] transition ${
        active
          ? 'border-[#d07347] bg-[#fff8f1]'
          : 'border-[#e3d7c8] bg-[#fbf7ef] hover:bg-white'
      }`}
    >
      <div className="mb-6 flex items-start justify-between gap-4">
        <div>
          <div className={`mb-4 inline-flex h-14 w-14 items-center justify-center rounded-2xl text-3xl font-black ${
            isInstalled ? 'bg-[#f3e4d6] text-[#c86d46]' : 'bg-[#ece5dc] text-[#9f9387]'
          }`}>
            {meta.icon}
          </div>
          <div className="text-2xl font-black text-[#2f241c]">{meta.title}</div>
          <div className="mt-3 text-lg text-[#7b6c5d]">模型：{selectedBindingLabel}</div>
        </div>
        <div className={`rounded-2xl px-4 py-2 text-sm font-semibold ${isInstalled ? 'bg-emerald-100 text-emerald-700' : 'bg-amber-100 text-amber-700'}`}>
          {toolStatus(detection)}
        </div>
      </div>
      <div className="space-y-2 text-base text-[#8d7d6d]">
        <div>应用：{trimPath(tool.launch_path ?? tool.launch_command ?? '-')}</div>
        <div>配置：{trimPath(tool.config_path ?? '-')}</div>
        <div>版本：{detection?.versions[0]?.replace(/^cli:/, '') ?? '-'}</div>
      </div>
      <div className="mt-5 flex flex-wrap gap-2">
        <Tag tone="slate">{appCategoryLabel(meta.category)}</Tag>
        <Tag tone={SUPPORTED_BINDING_TOOL_IDS.has(tool.id) ? 'green' : 'amber'}>
          {SUPPORTED_BINDING_TOOL_IDS.has(tool.id) ? '支持配置' : '待接入'}
        </Tag>
      </div>
      {!isInstalled && (
        <div className="mt-8">
          <span className="inline-flex rounded-2xl bg-[#ddd2c5] px-5 py-3 text-lg font-semibold text-[#5e5248]">
            {meta.installLabel ?? 'AI 自动安装'}
          </span>
        </div>
      )}
    </button>
  );
}
