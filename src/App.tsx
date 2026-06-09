import { lazy, Suspense, useEffect, useState } from 'react';
import {
  BookOpen,
  Boxes,
  BrainCircuit,
  Cpu,
  GraduationCap,
  MessageSquareText,
  Sparkles,
  Star,
  Wrench,
} from 'lucide-react';
import type { SectionId } from './lib/app-types';
import { sectionMeta } from './lib/app-utils';
const ModelCenterPage = lazy(() => import('./pages/ModelCenterPage'));
const ApplicationStudioPage = lazy(() => import('./pages/ApplicationStudioPage'));
const RepairPage = lazy(() => import('./pages/RepairPage'));
const LocalModelsPage = lazy(() => import('./pages/LocalModelsPage'));
const NewsPage = lazy(() => import('./pages/NewsPage'));
const FeedbackPage = lazy(() => import('./pages/FeedbackPage'));
const FeaturedPage = lazy(() => import('./pages/FeaturedPage'));
const CoursesPage = lazy(() => import('./pages/CoursesPage'));

const FALLBACK = (
  <div className="flex min-h-[400px] items-center justify-center text-[#8f7c6a]">
    <span className="text-lg">加载中...</span>
  </div>
);

const sidebarItems: Array<{
  id: SectionId;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
}> = [
  { id: 'news', label: 'AI 资讯', icon: BookOpen },
  { id: 'featured', label: '明星项目', icon: Star },
  { id: 'courses', label: 'AI 公开课', icon: GraduationCap },
  { id: 'models', label: '模型中心', icon: Boxes },
  { id: 'apps', label: '应用管理', icon: Cpu },
  { id: 'local-models', label: '本地大模型', icon: BrainCircuit },
  { id: 'repair', label: '安装与修复', icon: Wrench },
  { id: 'feedback', label: '问题反馈', icon: MessageSquareText },
];

function renderSection(section: SectionId) {
  switch (section) {
    case 'models':
      return <ModelCenterPage />;
    case 'apps':
      return <ApplicationStudioPage />;
    case 'local-models':
      return <LocalModelsPage />;
    case 'repair':
      return <RepairPage />;
    case 'news':
      return <NewsPage />;
    case 'featured':
      return <FeaturedPage />;
    case 'courses':
      return <CoursesPage />;
    case 'feedback':
      return <FeedbackPage />;
  }
}

export default function App() {
  const [activeSection, setActiveSection] = useState<SectionId>('models');
  const meta = sectionMeta(activeSection);
  const [compactLayout, setCompactLayout] = useState(false);

  useEffect(() => {
    const updateLayout = () => setCompactLayout(window.innerWidth < 1600);
    updateLayout();
    window.addEventListener('resize', updateLayout);
    return () => window.removeEventListener('resize', updateLayout);
  }, []);

  return (
    <div className="min-h-screen bg-[#f6f1e8] text-slate-900">
      <div
        className={`grid min-h-screen ${
          compactLayout
            ? 'grid-cols-[220px_minmax(0,1fr)]'
            : 'grid-cols-[280px_minmax(0,1fr)]'
        }`}
      >
        <aside
          className={`border-r border-[#e7ddd0] bg-[#f7f1e7] ${
            compactLayout ? 'px-6 py-6' : 'px-10 py-8'
          }`}
        >
          <div className="mb-10 flex items-center gap-4">
            <div className="grid h-14 w-14 place-items-center rounded-2xl bg-gradient-to-br from-[#ff8b4d] to-[#d8642f] text-white shadow-[0_18px_35px_rgba(216,100,47,0.22)]">
              <Sparkles className="h-7 w-7" />
            </div>
            <div>
              <div className="flex items-end gap-3">
                <h1
                  className={`${
                    compactLayout ? 'text-3xl' : 'text-4xl'
                  } font-black tracking-tight text-[#2f241c]`}
                >
                  观景
                </h1>
                <span className="pb-1 text-sm tracking-[0.35em] text-[#cc6a43]">
                  VISTA
                </span>
              </div>
              <p className="mt-1 text-sm text-[#8f7c6a]">
                中文模型与工具编排控制台
              </p>
            </div>
          </div>

          <nav className="space-y-2">
            {sidebarItems.map(item => {
              const Icon = item.icon;
              const active = item.id === activeSection;
              return (
                <button
                  key={item.id}
                  onClick={() => setActiveSection(item.id)}
                  className={`flex w-full items-center gap-4 rounded-2xl px-5 py-4 text-left transition-all ${
                    active
                      ? 'bg-[#ddd3c6] text-[#2f241c] shadow-[0_12px_30px_rgba(80,60,40,0.08)]'
                      : 'text-[#5f554d] hover:bg-[#ede5da]'
                  }`}
                >
                  <Icon className="h-6 w-6" />
                  <span
                    className={`${
                      compactLayout ? 'text-lg' : 'text-xl'
                    } font-semibold`}
                  >
                    {item.label}
                  </span>
                </button>
              );
            })}
          </nav>

          <div className="mt-auto pt-12 text-sm text-[#8f7c6a]">
            <div className="rounded-2xl border border-[#eadfce] bg-[#fbf7f0] px-5 py-4">
              <div className="font-semibold text-[#5a4b3d]">本地大模型</div>
              <div className="mt-1 text-[#9e8d7d]">离线</div>
            </div>
          </div>
        </aside>

        <main
          className={`${
            compactLayout ? 'px-6 py-6' : 'px-10 py-8'
          }`}
        >
          <header className="mb-8 flex items-start justify-between">
            <div>
              <div className="flex items-end gap-4">
                <h2
                  className={`${
                    compactLayout ? 'text-4xl' : 'text-5xl'
                  } font-black tracking-tight text-[#2d241b]`}
                >
                  {meta.title}
                </h2>
                <span className="pb-2 text-sm font-semibold tracking-[0.35em] text-[#c96f46]">
                  {meta.accent}
                </span>
              </div>
              <p className="mt-3 text-base text-[#8c7a69]">{meta.subtitle}</p>
            </div>
            <div className="rounded-full bg-[#efe7db] px-4 py-2 text-sm font-semibold text-[#786a5d]">
              中文界面
            </div>
          </header>

          <Suspense fallback={FALLBACK}>
            {renderSection(activeSection)}
          </Suspense>
        </main>
      </div>
    </div>
  );
}
