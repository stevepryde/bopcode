import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Star } from "lucide-react";

interface CelebrationOverlayProps {
  isVisible: boolean;
  onNextPuzzle?: () => void;
  onRepeatLevel?: () => void;
  repeatLabel?: string;
  onWorldComplete?: () => void;
  completionLabel?: string;
  worldName?: string | null;
  hasNextPuzzle: boolean;
  starsMet: boolean[];
}

interface Confetti {
  id: number;
  x: number;
  color: string;
  delay: number;
  duration: number;
}

const CONFETTI_COLORS = [
  "#86efac", // soft mint
  "#a5b4fc", // soft periwinkle
  "#d8b4fe", // soft lavender
  "#fde68a", // soft gold
  "#f9a8d4", // soft pink
  "#a5f3fc", // soft cyan
];

export function CelebrationOverlay({
  isVisible,
  onNextPuzzle,
  onRepeatLevel,
  repeatLabel,
  onWorldComplete,
  completionLabel,
  worldName,
  hasNextPuzzle,
  starsMet,
}: CelebrationOverlayProps) {
  const [confetti, setConfetti] = useState<Confetti[]>([]);

  const starsEarned = starsMet.filter(Boolean).length;
  const totalStars = starsMet.length;
  const isLastLevelInWorld = !hasNextPuzzle && !completionLabel;

  useEffect(() => {
    if (isVisible) {
      // Generate confetti pieces
      const pieces: Confetti[] = Array.from({ length: 50 }, (_, i) => ({
        id: i,
        x: Math.random() * 100,
        color: CONFETTI_COLORS[Math.floor(Math.random() * CONFETTI_COLORS.length)] ?? "#34d399",
        delay: Math.random() * 0.5,
        duration: 2 + Math.random() * 2,
      }));
      setConfetti(pieces);
    } else {
      setConfetti([]);
    }
  }, [isVisible]);

  if (!isVisible) return null;

  return (
    <div className="absolute inset-0 flex items-center justify-center bg-black/70 z-50 overflow-hidden">
      {/* Confetti */}
      {confetti.map((piece) => (
        <div
          key={piece.id}
          className="absolute w-3 h-3 rounded-sm animate-confetti"
          style={{
            left: `${piece.x}%`,
            top: "-20px",
            backgroundColor: piece.color,
            animationDelay: `${piece.delay}s`,
            animationDuration: `${piece.duration}s`,
          }}
        />
      ))}

      {/* Celebration Content */}
      <div className="text-center space-y-6 bg-white/95 dark:bg-zinc-900/90 p-8 rounded-xl border border-zinc-200 dark:border-[var(--theme-400)]/25 shadow-2xl z-10">
        {/* Stars display */}
        <div className="flex justify-center gap-2">
          {starsMet.map((met, i) => (
            <Star
              key={i}
              className={`h-10 w-10 ${
                met
                  ? "text-yellow-300 fill-yellow-300"
                  : "text-zinc-600"
              }`}
            />
          ))}
          {totalStars === 0 && (
            <svg viewBox="0 0 24 24" className="inline w-16 h-16 text-yellow-300" fill="currentColor">
              <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z" />
            </svg>
          )}
        </div>
        <h2 className="text-3xl font-bold text-emerald-300">
          {isLastLevelInWorld ? "World Complete!" : "Puzzle Complete!"}
        </h2>
        {isLastLevelInWorld && (
          <p className="text-zinc-600 dark:text-zinc-200">
            You&apos;ve completed all the levels in {worldName ?? "this world"}.
          </p>
        )}
        {totalStars > 0 && (
          <p className="text-zinc-500 dark:text-zinc-300">
            {starsEarned}/{totalStars} stars
          </p>
        )}
        <div className="flex gap-4 justify-center pt-4">
          {onRepeatLevel && (
            <Button
              variant="ghost"
              onClick={onRepeatLevel}
              className="px-6 bg-transparent border border-[var(--theme-500)]/30 text-[var(--theme-700)] dark:text-[var(--theme-400)] hover:text-zinc-900 dark:hover:text-white hover:border-[var(--theme-500)]/50 hover:bg-[var(--theme-500)]/10"
            >
              {repeatLabel ?? (starsEarned < totalStars ? "Try for More Stars" : "Repeat Level")}
            </Button>
          )}
          {hasNextPuzzle && onNextPuzzle && (
            <Button
              onClick={onNextPuzzle}
              className="px-6"
            >
              Next Puzzle
            </Button>
          )}
          {(isLastLevelInWorld || completionLabel) && onWorldComplete && (
            <Button
              onClick={onWorldComplete}
              className="px-6"
            >
              {completionLabel ?? "Back to Worlds"}
            </Button>
          )}
        </div>
      </div>

      {/* CSS for confetti animation */}
      <style>{`
        @keyframes confetti-fall {
          0% {
            transform: translateY(0) rotate(0deg);
            opacity: 1;
          }
          100% {
            transform: translateY(100vh) rotate(720deg);
            opacity: 0;
          }
        }
        .animate-confetti {
          animation: confetti-fall ease-out forwards;
        }
      `}</style>
    </div>
  );
}
