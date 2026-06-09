import type { ReactNode } from 'react';

type Tone = 'slate' | 'amber' | 'blue' | 'green' | 'rose';

const styles: Record<Tone, string> = {
  slate: 'bg-white/80 text-[#7f7265] border border-[#e5dacc]',
  amber: 'bg-[#fff1d6] text-[#b27016]',
  blue: 'bg-[#e5f0ff] text-[#3f72c8]',
  green: 'bg-[#def6e8] text-[#2f9157]',
  rose: 'bg-[#ffe5df] text-[#c35c44]',
};

export function Tag({ tone, children }: { tone: Tone; children: ReactNode }) {
  return (
    <span className={`rounded-full px-3 py-1 text-sm font-semibold ${styles[tone]}`}>
      {children}
    </span>
  );
}
