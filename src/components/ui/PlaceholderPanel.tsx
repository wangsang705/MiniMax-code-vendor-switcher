export function PlaceholderPanel({ title, body }: { title: string; body: string }) {
  return (
    <div className="rounded-[32px] border border-[#e3d7c8] bg-[#fbf7ef] p-10 shadow-[0_18px_40px_rgba(92,70,44,0.08)]">
      <div className="max-w-3xl">
        <div className="text-3xl font-black text-[#2f241c]">{title}</div>
        <p className="mt-4 text-lg leading-8 text-[#7d6f63]">{body}</p>
      </div>
    </div>
  );
}
