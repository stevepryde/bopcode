import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex items-center rounded-full px-3 py-1 text-sm font-medium transition-colors",
  {
    variants: {
      variant: {
        default: "bg-zinc-800 text-zinc-200 border border-zinc-700",
        success: "bg-emerald-900/50 text-emerald-300 border border-emerald-700/50",
        warning: "bg-amber-900/50 text-amber-300 border border-amber-700/50",
        error: "bg-red-900/50 text-red-300 border border-red-700/50",
        accent: "bg-[var(--theme-500)]/10 text-[var(--theme-700)] dark:text-[var(--theme-300)] border border-[var(--theme-700)]/50",
        outline: "border border-zinc-600 text-zinc-300",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  },
);

interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

export function Badge({ className, variant, ...props }: BadgeProps) {
  return (
    <div className={cn(badgeVariants({ variant }), className)} {...props} />
  );
}
