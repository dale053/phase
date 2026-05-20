import type { ReactNode } from "react";

interface GameplayTooltipProps {
  id?: string;
  children: ReactNode;
  className?: string;
}

export function GameplayTooltip({
  id,
  children,
  className,
}: GameplayTooltipProps) {
  return (
    <span
      id={id}
      role="tooltip"
      className={[
        "pointer-events-none absolute right-0 bottom-full z-50 mb-2 hidden w-64 rounded-md border border-white/10 bg-slate-950/95 px-3 py-2 text-left text-[11px] leading-snug font-medium text-slate-100 shadow-2xl shadow-black/40 backdrop-blur-xl group-hover:block group-focus-visible:block",
        className,
      ]
        .filter(Boolean)
        .join(" ")}
    >
      {children}
    </span>
  );
}
