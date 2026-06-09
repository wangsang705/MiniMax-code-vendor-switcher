import { useState } from 'react';
import {
  BookOpen,
  ExternalLink,
  Hash,
  Sparkles,
  TrendingUp,
} from 'lucide-react';

interface NewsItem {
  id: string;
  title: string;
  summary: string;
  source: string;
  category: string;
  url?: string;
  date: string;
}

const FEATURED_MODELS = [
  { name: 'DeepSeek-V4', provider: 'DeepSeek', tag: '推理', desc: '最新的推理模型，SWE-bench 领先' },
  { name: 'GLM-5V-Turbo', provider: '智谱', tag: '多模态', desc: '支持图像理解的视觉语言模型' },
  { name: 'Qwen3-Plus', provider: '阿里云', tag: '通用', desc: '通义千问最新版，长上下文支持' },
  { name: 'MiniMax-M2.7', provider: 'MiniMax', tag: '对话', desc: 'MiniMax 最新对话模型' },
  { name: 'Kimi-K2.5', provider: '月之暗面', tag: '长文', desc: '超长上下文，适合深度阅读' },
  { name: 'Claude Sonnet 4.6', provider: 'Anthropic', tag: '编码', desc: '前沿编码能力，Agent 友好' },
];

const NEWS_ITEMS: NewsItem[] = [
  {
    id: '1',
    title: '观景 VISTA v0.1.0 发布',
    summary: '统一管理 18+ AI 编码工具配置，一次录入厂商信息，全局自动配置。支持 Claude Code、MiniMax Code、Codex CLI 等主流工具。',
    source: '观景团队',
    category: '产品',
    date: '2026-06',
  },
  {
    id: '2',
    title: 'DeepSeek-V4 发布：推理能力大幅提升',
    summary: 'DeepSeek-V4 在数学推理和代码生成上取得显著进步，支持更长的上下文窗口。',
    source: 'DeepSeek',
    category: '模型',
    date: '2026-06',
  },
  {
    id: '3',
    title: 'GLM-5 系列正式开放 API',
    summary: '智谱 GLM-5 系列模型全面开放 API 调用，支持多模态输入，中文理解能力业界领先。',
    source: '智谱 AI',
    category: '模型',
    date: '2026-05',
  },
  {
    id: '4',
    title: 'Claude Code CLI 更新：支持更多自定义配置',
    summary: 'Claude Code CLI 新增环境变量配置支持，可通过 ANTHROPIC_BASE_URL 自定义 API 端点。',
    source: 'Anthropic',
    category: '工具',
    date: '2026-05',
  },
  {
    id: '5',
    title: 'Qwen3 系列开源模型发布',
    summary: '阿里云开源 Qwen3 系列模型，包含 30B/235B 等多个规格，支持思维链推理。',
    source: '阿里云',
    category: '模型',
    date: '2026-05',
  },
  {
    id: '6',
    title: 'Aider 支持更多 LLM 后端',
    summary: 'AI 结对编程工具 Aider 现已支持通过环境变量配置任意 OpenAI 兼容 API。',
    source: 'Aider',
    category: '工具',
    date: '2026-04',
  },
];

const CATEGORIES = ['全部', '模型', '工具', '产品'];

export default function NewsPage() {
  const [activeCategory, setActiveCategory] = useState('全部');

  const filtered = activeCategory === '全部'
    ? NEWS_ITEMS
    : NEWS_ITEMS.filter(item => item.category === activeCategory);

  return (
    <div className="space-y-8">
      {/* 精选模型 */}
      <section>
        <div className="mb-6 flex items-center gap-3">
          <Sparkles className="h-6 w-6 text-[#d07347]" />
          <h3 className="text-2xl font-black text-[#2f241c]">精选模型</h3>
        </div>
        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
          {FEATURED_MODELS.map(model => (
            <div
              key={model.name}
              className="rounded-[24px] border border-[#e3d7c8] bg-[#fbf7ef] px-5 py-5 shadow-[0_12px_30px_rgba(92,70,44,0.06)] transition hover:bg-white"
            >
              <div className="mb-3 flex items-center gap-3">
                <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-[#ff8b4d] to-[#d8642f] text-sm font-bold text-white">
                  {model.provider.slice(0, 1)}
                </div>
                <div>
                  <div className="text-lg font-bold text-[#2f241c]">{model.name}</div>
                  <div className="text-sm text-[#9a8a79]">{model.provider}</div>
                </div>
              </div>
              <p className="mb-3 text-sm text-[#7d6f63]">{model.desc}</p>
              <span className="inline-flex rounded-full bg-[#efe7db] px-3 py-1 text-xs font-semibold text-[#7f7265]">
                {model.tag}
              </span>
            </div>
          ))}
        </div>
      </section>

      {/* 资讯列表 */}
      <section>
        <div className="mb-6 flex items-center gap-3">
          <TrendingUp className="h-6 w-6 text-[#d07347]" />
          <h3 className="text-2xl font-black text-[#2f241c]">最新资讯</h3>
        </div>

        <div className="mb-6 flex flex-wrap items-center gap-3">
          {CATEGORIES.map(cat => (
            <button
              key={cat}
              onClick={() => setActiveCategory(cat)}
              className={`rounded-xl px-5 py-2.5 text-base font-semibold transition ${
                activeCategory === cat
                  ? 'bg-[#d07347] text-white'
                  : 'bg-[#efe7db] text-[#5a4c40] hover:bg-[#e5dacd]'
              }`}
            >
              {cat}
            </button>
          ))}
        </div>

        <div className="space-y-4">
          {filtered.map(item => (
            <div
              key={item.id}
              className="rounded-[24px] border border-[#e3d7c8] bg-[#fbf7ef] px-6 py-5 transition hover:bg-white"
            >
              <div className="flex items-start justify-between gap-4">
                <div className="min-w-0">
                  <div className="flex items-center gap-3">
                    <h4 className="text-xl font-bold text-[#2f241c]">{item.title}</h4>
                    <span className="shrink-0 rounded-full bg-[#efe7db] px-2.5 py-0.5 text-xs font-medium text-[#7f7265]">
                      {item.category}
                    </span>
                  </div>
                  <p className="mt-2 text-base leading-relaxed text-[#7d6f63]">
                    {item.summary}
                  </p>
                  <div className="mt-3 flex items-center gap-4 text-sm text-[#9a8a79]">
                    <span className="flex items-center gap-1">
                      <Hash className="h-3 w-3" />
                      {item.source}
                    </span>
                    <span className="flex items-center gap-1">
                      <BookOpen className="h-3 w-3" />
                      {item.date}
                    </span>
                  </div>
                </div>
                {item.url && (
                  <a
                    href={item.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="shrink-0 text-[#d07347] hover:text-[#c26438]"
                  >
                    <ExternalLink className="h-5 w-5" />
                  </a>
                )}
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* 统计 */}
      <div className="rounded-[28px] border border-[#e3d7c8] bg-[#fbf7ef] p-6">
        <div className="grid gap-4 text-center md:grid-cols-3">
          <div>
            <div className="text-3xl font-black text-[#2f241c]">
              {FEATURED_MODELS.length}+
            </div>
            <div className="mt-1 text-sm text-[#9a8a79]">精选模型</div>
          </div>
          <div>
            <div className="text-3xl font-black text-[#2f241c]">
              {NEWS_ITEMS.length}
            </div>
            <div className="mt-1 text-sm text-[#9a8a79]">资讯文章</div>
          </div>
          <div>
            <div className="text-3xl font-black text-[#2f241c]">18+</div>
            <div className="mt-1 text-sm text-[#9a8a79]">支持工具</div>
          </div>
        </div>
      </div>
    </div>
  );
}
