import { useState, useRef, useEffect, type ReactNode } from "react";

interface TooltipProps {
  content: string;
  children: ReactNode;
  delay?: number;
}

export function Tooltip({ content, children, delay = 400 }: TooltipProps) {
  const [visible, setVisible] = useState(false);
  const [position, setPosition] = useState<"top" | "bottom">("top");
  const timeoutRef = useRef<number | null>(null);
  const triggerRef = useRef<HTMLDivElement>(null);

  const show = () => {
    timeoutRef.current = window.setTimeout(() => {
      if (triggerRef.current) {
        const rect = triggerRef.current.getBoundingClientRect();
        setPosition(rect.top < 60 ? "bottom" : "top");
      }
      setVisible(true);
    }, delay);
  };

  const hide = () => {
    if (timeoutRef.current) clearTimeout(timeoutRef.current);
    setVisible(false);
  };

  useEffect(() => {
    return () => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
    };
  }, []);

  return (
    <div
      ref={triggerRef}
      className="relative inline-flex"
      onMouseEnter={show}
      onMouseLeave={hide}
      onFocus={show}
      onBlur={hide}
    >
      {children}
      {visible && (
        <div
          role="tooltip"
          style={{ animation: "tooltip-in 0.15s ease-out" }}
          className={`absolute left-1/2 -translate-x-1/2 z-50 px-2.5 py-1.5 text-xs font-medium text-white bg-zinc-800 dark:bg-zinc-700 rounded-md shadow-lg whitespace-nowrap pointer-events-none border border-zinc-700 dark:border-zinc-600 ${
            position === "top" ? "bottom-full mb-2" : "top-full mt-2"
          }`}
        >
          {content}
          <span
            className={`absolute left-1/2 -translate-x-1/2 w-2 h-2 bg-zinc-800 dark:bg-zinc-700 rotate-45 ${
              position === "top"
                ? "top-full -mt-1 border-r border-b border-zinc-700 dark:border-zinc-600"
                : "bottom-full -mb-1 border-l border-t border-zinc-700 dark:border-zinc-600"
            }`}
          />
        </div>
      )}
    </div>
  );
}
