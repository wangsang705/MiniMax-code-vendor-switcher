import { useState } from 'react';
import { ExternalLink, GraduationCap, Play, FileText, BookOpen } from 'lucide-react';

interface Course {
  title: string;
  desc: string;
  level: '入门' | '进阶' | '高级';
  duration: string;
  tags: string[];
  type: '视频' | '文章' | '教程';
}

const COURSES: Course[] = [
  { title: '观景 VISTA 快速上手指南', desc: '5 分钟学会使用观景管理你的 AI 编码工具配置，从添加厂商到绑定工具。', level: '入门', duration: '5min', tags: ['观景', '配置'], type: '教程' },
  { title: 'Claude Code CLI 入门', desc: '了解如何安装、配置和使用 Claude Code CLI 进行 AI 辅助编程。', level: '入门', duration: '15min', tags: ['Claude', 'CLI'], type: '视频' },
  { title: 'MiniMax Code 完全配置指南', desc: '从安装到高级配置，掌握 MiniMax Code CLI 和桌面端的所有玩法。', level: '入门', duration: '20min', tags: ['MiniMax', '配置'], type: '教程' },
  { title: 'AI 编码工具对比：选哪个？', desc: 'Claude Code vs Codex CLI vs Aider vs MiniMax Code — 深度对比各个工具的优劣。', level: '入门', duration: '10min', tags: ['对比', '选型'], type: '文章' },
  { title: '使用 Aider 进行结对编程', desc: 'Aider 是终端内的 AI 编程搭档，学会用自然语言驱动代码变更。', level: '进阶', duration: '30min', tags: ['Aider', '结对编程'], type: '视频' },
  { title: '配置多个 LLM 厂商切换策略', desc: '如何优雅地在 DeepSeek、Kimi、智谱、Qwen 之间切换，找到性价比最优的组合。', level: '进阶', duration: '15min', tags: ['多厂商', '策略'], type: '文章' },
  { title: 'AI Agent 入门：OpenClaw/Hermes/NanoBot', desc: '了解三种 AI Agent 的安装、配置和使用场景，让 AI 帮你自动完成任务。', level: '进阶', duration: '25min', tags: ['Agent', '自动化'], type: '教程' },
  { title: '本地大模型部署指南', desc: '使用 Ollama/LM Studio/vLLM 部署本地推理服务，并通过观景接入。', level: '进阶', duration: '40min', tags: ['本地', 'Ollama', '部署'], type: '教程' },
  { title: 'Rust + Tauri 2.0 桌面开发实战', desc: '从零开始构建跨平台桌面应用，观景 VISTA 就是最好的参考案例。', level: '高级', duration: '60min', tags: ['Rust', 'Tauri', '开发'], type: '视频' },
  { title: 'API Key 安全管理最佳实践', desc: 'Keyring 加密存储、环境变量注入、日志脱敏 — 保护你的 API 凭证。', level: '进阶', duration: '10min', tags: ['安全', 'Keyring'], type: '文章' },
  { title: '策略模式在桌面应用中的应用', desc: '观景 VISTA 的 WriterRegistry 设计解析 — 学习如何用策略模式解耦配置写入。', level: '高级', duration: '20min', tags: ['架构', '设计模式'], type: '文章' },
  { title: '从自然语言到可运行代码：Forge 协议', desc: '6 层强制协议：需求澄清→技术选型→骨架先行→里程碑门控→自检闭环→反脆弱迭代。', level: '进阶', duration: '15min', tags: ['方法论', '协议'], type: '文章' },
];

const LEVEL_STYLES: Record<string, string> = {
  '入门': 'bg-emerald-100 text-emerald-700',
  '进阶': 'bg-amber-100 text-amber-700',
  '高级': 'bg-rose-100 text-rose-700',
};

const TYPE_ICONS: Record<string, React.ComponentType<{ className?: string }>> = {
  '视频': Play,
  '文章': FileText,
  '教程': BookOpen,
};

const ALL_LEVELS = ['全部', '入门', '进阶', '高级'];

export default function CoursesPage() {
  const [level, setLevel] = useState('全部');

  const filtered = level === '全部' ? COURSES : COURSES.filter(c => c.level === level);

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center gap-3">
        {ALL_LEVELS.map(l => (
          <button
            key={l}
            onClick={() => setLevel(l)}
            className={`rounded-xl px-5 py-2.5 text-base font-semibold transition ${
              level === l
                ? 'bg-[#d07347] text-white'
                : 'bg-[#efe7db] text-[#5a4c40] hover:bg-[#e5dacd]'
            }`}
          >
            {l}
          </button>
        ))}
      </div>

      <div className="space-y-4">
        {filtered.map(course => {
          const TypeIcon = TYPE_ICONS[course.type] ?? BookOpen;
          return (
            <div
              key={course.title}
              className="group rounded-[24px] border border-[#e3d7c8] bg-[#fbf7ef] px-6 py-5 transition hover:bg-white"
            >
              <div className="flex items-start justify-between gap-4">
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-center gap-3">
                    <h4 className="text-xl font-bold text-[#2f241c]">{course.title}</h4>
                    <span className={`rounded-full px-2.5 py-0.5 text-xs font-semibold ${LEVEL_STYLES[course.level]}`}>
                      {course.level}
                    </span>
                    <span className="flex items-center gap-1 rounded-full bg-[#efe7db] px-2.5 py-0.5 text-xs font-medium text-[#7f7265]">
                      <TypeIcon className="h-3 w-3" />
                      {course.type}
                    </span>
                  </div>
                  <p className="mt-2 text-base leading-relaxed text-[#7d6f63]">{course.desc}</p>
                  <div className="mt-3 flex flex-wrap items-center gap-4 text-sm text-[#9a8a79]">
                    <span className="flex items-center gap-1">
                      <GraduationCap className="h-3.5 w-3.5" />
                      {course.duration}
                    </span>
                    <span className="flex flex-wrap gap-1.5">
                      {course.tags.map(tag => (
                        <span key={tag} className="text-[#9a8a79]">#{tag}</span>
                      ))}
                    </span>
                  </div>
                </div>
                <ExternalLink className="mt-1 h-5 w-5 shrink-0 text-[#9a8a79] opacity-0 transition group-hover:opacity-100" />
              </div>
            </div>
          );
        })}

        {filtered.length === 0 && (
          <div className="rounded-[28px] border border-dashed border-[#d9cdbd] bg-[#fbf8f1] px-8 py-12 text-center text-lg text-[#8f7d6a]">
            该分类还没有课程
          </div>
        )}
      </div>

      <div className="rounded-[28px] border border-[#e3d7c8] bg-[#fbf7ef] p-6">
        <div className="flex items-center gap-3">
          <GraduationCap className="h-6 w-6 text-[#d07347]" />
          <div>
            <h3 className="text-xl font-bold text-[#2f241c]">想了解什么？</h3>
            <p className="mt-1 text-sm text-[#7d6f63]">
              如果你有想学习的话题，可以通过"问题反馈"页面告诉我们。我们会持续更新课程内容。
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
