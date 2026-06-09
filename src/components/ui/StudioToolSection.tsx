import type { ReactNode } from 'react';

export function StudioToolSection({
  title,
  subtitle,
  children,
}: {
  title: string;
  subtitle: string;
  children: ReactNode;
}) {
  return (
    <div>
      <div className="mb-4 flex items-end justify-between gap-4">
        <div>
          <h4 className="text-2xl font-black text-[#2f241c]">{title}</h4>
          <p className="mt-1 text-base text-[#867867]">{subtitle}</p>
        </div>
      </div>
      <div className="grid gap-6 md:grid-cols-2 xl:grid-cols-3">{children}</div>
    </div>
  );
}
