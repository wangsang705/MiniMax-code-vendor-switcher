export function SummaryCard({
  title,
  value,
  detail,
}: {
  title: string;
  value: string;
  detail: string;
}) {
  return (
    <div className="rounded-[24px] border border-[#e3d7c8] bg-[#fbf7ef] px-5 py-5 shadow-[0_18px_40px_rgba(92,70,44,0.06)]">
      <div className="text-sm tracking-[0.22em] text-[#9a8a79]">{title}</div>
      <div className="mt-3 text-4xl font-black text-[#2f241c]">{value}</div>
      <div className="mt-2 text-base text-[#7c6c5b]">{detail}</div>
    </div>
  );
}
