import { useState } from 'react';
import { ExternalLink, Github, Star } from 'lucide-react';

interface Project {
  name: string;
  desc: string;
  url: string;
  stars: string;
  lang: string;
  tags: string[];
}

const PROJECTS: Project[] = [
  { name: 'Claude Code CLI', desc: 'Anthropic 官方 CLI 工具，支持 Agent 模式自动编码。可通过环境变量切换任意 API 后端。', url: 'https://docs.anthropic.com/en/docs/claude-code/overview', stars: '12k+', lang: 'TypeScript', tags: ['CLI', 'Agent', '编码'] },
  { name: 'MiniMax Code CLI', desc: 'MiniMax 出品的 AI 编码 CLI，兼容 Claude Code 协议，支持 MiniMax-M3/M2.7 等模型。', url: 'https://github.com/MiniMax-AI/MiniMax-code', stars: '5k+', lang: 'TypeScript', tags: ['CLI', '编码', '中文'] },
  { name: 'Aider', desc: 'AI 结对编程工具，终端内的 AI 编程助手。支持多文件编辑和 Git 自动提交。', url: 'https://aider.chat/', stars: '25k+', lang: 'Python', tags: ['CLI', '结对编程', '开源'] },
  { name: 'Codex CLI', desc: 'OpenAI 开源的编码 Agent CLI，支持对话式编程和自动执行命令。', url: 'https://github.com/openai/codex', stars: '18k+', lang: 'TypeScript', tags: ['CLI', 'Agent', '开源'] },
  { name: 'OpenCode CLI', desc: '开源的 AI 编码 CLI，支持多种 LLM 后端和自定义配置。', url: 'https://github.com/opencode-ai/opencode', stars: '3k+', lang: 'Rust', tags: ['CLI', '开源', '跨平台'] },
  { name: 'Kimi CLI', desc: '月之暗面推出的 AI 编码助手 CLI，擅长中文理解和长上下文处理。', url: 'https://kimi.moonshot.cn/', stars: '--', lang: '--', tags: ['CLI', '中文', '长上下文'] },
  { name: 'Qwen Code CLI', desc: '阿里云通义千问的 CLI 编码工具，支持 Qwen 系列模型。', url: 'https://github.com/QwenLM/Qwen-code-cli', stars: '2k+', lang: 'Python', tags: ['CLI', '中文', '开源'] },
  { name: 'OpenClaw', desc: '轻量级 AI Agent 框架，支持终端内自动执行任务和工具调用。', url: 'https://github.com/openclaw', stars: '1k+', lang: 'TypeScript', tags: ['Agent', '开源', '轻量'] },
  { name: 'Hermes Agent', desc: '通用 AI Agent，支持自主规划和执行复杂任务，可接入多种 LLM。', url: 'https://github.com/hermes-ai', stars: '--', lang: 'Python', tags: ['Agent', '规划', '自动化'] },
  { name: 'NanoBot', desc: '极简 AI Agent，适合日常自动化任务。配置简洁，开箱即用。', url: 'https://github.com/nanobot', stars: '--', lang: 'TypeScript', tags: ['Agent', '极简', '自动化'] },
  { name: 'Qwen3', desc: '阿里云通义千问最新开源模型，支持 30B/235B 多种规格和思维链推理。', url: 'https://github.com/QwenLM/Qwen3', stars: '10k+', lang: 'Python', tags: ['模型', '开源', '中文'] },
  { name: 'DeepSeek-V4', desc: 'DeepSeek 最新推理模型，在编程和数学推理上表现优异。', url: 'https://deepseek.com', stars: '--', lang: '--', tags: ['模型', '推理', '编程'] },
];

const LANG_COLORS: Record<string, string> = {
  TypeScript: 'bg-blue-500',
  Python: 'bg-yellow-500',
  Rust: 'bg-orange-500',
};

const ALL_TAGS = Array.from(new Set(PROJECTS.flatMap(p => p.tags))).sort();

const TAG_EMOJI: Record<string, string> = {
  CLI: '💻', Agent: '🤖', 编码: '⌨️', 开源: '📖',
  中文: '🇨🇳', 结对编程: '👥', 跨平台: '🔄', 模型: '🧠',
  推理: '🔍', 编程: '⚡', 轻量: '🪶', 规划: '📋',
  自动化: '⚙️', 极简: '📐', 长上下文: '📏',
};

export default function FeaturedPage() {
  const [activeTag, setActiveTag] = useState('');

  const filtered = activeTag
    ? PROJECTS.filter(p => p.tags.includes(activeTag))
    : PROJECTS;

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center gap-3">
        <button
          onClick={() => setActiveTag('')}
          className={`rounded-xl px-5 py-2.5 text-base font-semibold transition ${
            !activeTag
              ? 'bg-[#d07347] text-white'
              : 'bg-[#efe7db] text-[#5a4c40] hover:bg-[#e5dacd]'
          }`}
        >
          全部项目
        </button>
        {ALL_TAGS.map(tag => (
          <button
            key={tag}
            onClick={() => setActiveTag(tag)}
            className={`rounded-xl px-5 py-2.5 text-base font-semibold transition ${
              activeTag === tag
                ? 'bg-[#d07347] text-white'
                : 'bg-[#efe7db] text-[#5a4c40] hover:bg-[#e5dacd]'
            }`}
          >
            {TAG_EMOJI[tag] ?? ''} {tag}
          </button>
        ))}
      </div>

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {filtered.map(project => (
          <div
            key={project.name}
            className="group rounded-[28px] border border-[#e3d7c8] bg-[#fbf7ef] p-6 shadow-[0_18px_40px_rgba(92,70,44,0.06)] transition hover:bg-white hover:shadow-[0_18px_40px_rgba(92,70,44,0.12)]"
          >
            <div className="mb-4 flex items-start justify-between gap-4">
              <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-gradient-to-br from-[#ff8b4d] to-[#d8642f] text-lg font-bold text-white">
                {project.name.slice(0, 2).toUpperCase()}
              </div>
              {project.url && (
                <a
                  href={project.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="shrink-0 text-[#9a8a79] opacity-0 transition group-hover:opacity-100"
                >
                  <ExternalLink className="h-5 w-5 hover:text-[#d07347]" />
                </a>
              )}
            </div>

            <h3 className="text-xl font-bold text-[#2f241c]">{project.name}</h3>
            <p className="mt-2 text-sm leading-relaxed text-[#7d6f63]">{project.desc}</p>

            <div className="mt-4 flex flex-wrap items-center gap-3">
              {project.lang !== '--' && (
                <span className="flex items-center gap-1.5 text-sm text-[#7d6f63]">
                  <span className={`inline-block h-3 w-3 rounded-full ${LANG_COLORS[project.lang] ?? 'bg-slate-400'}`} />
                  {project.lang}
                </span>
              )}
              {project.stars !== '--' && (
                <span className="flex items-center gap-1 text-sm text-amber-600">
                  <Star className="h-3.5 w-3.5 fill-amber-400" />
                  {project.stars}
                </span>
              )}
            </div>

            <div className="mt-4 flex flex-wrap gap-2">
              {project.tags.map(tag => (
                <span
                  key={tag}
                  className="rounded-full bg-white/80 px-2.5 py-1 text-xs font-medium text-[#7f7265]"
                >
                  {TAG_EMOJI[tag] ?? ''} {tag}
                </span>
              ))}
            </div>
          </div>
        ))}
      </div>

      <div className="rounded-[28px] border border-[#e3d7c8] bg-[#fbf7ef] px-6 py-6">
        <div className="flex items-center gap-3">
          <Github className="h-6 w-6 text-[#d07347]" />
          <h3 className="text-xl font-bold text-[#2f241c]">观景 VISTA 也是开源的</h3>
        </div>
        <p className="mt-2 text-sm text-[#7d6f63]">
          观景本身也是开源项目，欢迎贡献新的工具适配器、修复 bug 或提出功能建议。
        </p>
      </div>
    </div>
  );
}
