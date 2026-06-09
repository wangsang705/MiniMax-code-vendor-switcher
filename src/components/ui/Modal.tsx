import type { ReactNode } from 'react';

interface ModalProps {
  title: string;
  children: ReactNode;
  onClose: () => void;
}

export function Modal({ title, children, onClose }: ModalProps) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-[rgba(30,24,18,0.28)] p-6"
      onClick={onClose}
    >
      <div
        className="w-full max-w-xl rounded-[32px] border border-[#eadfce] bg-[#fffdfa] p-8 shadow-[0_28px_60px_rgba(65,45,25,0.18)]"
        onClick={event => event.stopPropagation()}
      >
        <h3 className="mb-6 text-3xl font-black text-[#2f241c]">{title}</h3>
        {children}
      </div>
    </div>
  );
}
