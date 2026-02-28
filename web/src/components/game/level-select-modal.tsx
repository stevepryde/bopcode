import { useEffect, useRef, useState } from "react";
import { X, Lock, Check, Star, RotateCcw } from "lucide-react";
import { GameGrid } from "./game-grid";
import { getLevel } from "@/lib/wasm";
import type { LevelSummary, GameProgress, PuzzleConfig } from "@/types/game";

interface LevelSelectModalProps {
  levels: LevelSummary[];
  currentIndex: number;
  progress: GameProgress;
  onSelect: (index: number) => void;
  onClose: () => void;
  onReset: () => void;
}

export function LevelSelectModal({
  levels,
  currentIndex,
  progress,
  onSelect,
  onClose,
  onReset,
}: LevelSelectModalProps) {
  const [confirmReset, setConfirmReset] = useState(false);
  const backdropRef = useRef<HTMLDivElement>(null);

  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === backdropRef.current) onClose();
  };

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onClose]);

  const completedIds = new Set(
    Object.entries(progress.levels)
      .filter(([, lp]) => lp.completed)
      .map(([id]) => id),
  );

  return (
    <div
      ref={backdropRef}
      onClick={handleBackdropClick}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4"
    >
      <div className="bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700/40 rounded-xl shadow-2xl max-w-2xl w-full max-h-[80vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-200 dark:border-zinc-800">
          <h2 className="text-lg font-bold text-zinc-900 dark:text-white">Select Level</h2>
          <button
            onClick={onClose}
            className="text-zinc-400 hover:text-zinc-900 dark:hover:text-white transition-colors cursor-pointer"
          >
            <X className="h-5 w-5" />
          </button>
        </div>

        {/* Puzzle list */}
        <div className="flex-1 overflow-y-auto p-4 space-y-3">
          {levels.map((level, index) => {
            const isCompleted = completedIds.has(level.puzzle_id);
            const isCurrent = index === currentIndex;
            const isUnlocked =
              index === 0 ||
              (levels[index - 1] != null &&
                completedIds.has(levels[index - 1]!.puzzle_id));
            const canSelect = isUnlocked || isCompleted;

            const lp = progress.levels[level.puzzle_id];
            const stars = lp?.completed ? Math.min(3, lp.stars) : 0;

            return (
              <LevelCard
                key={level.puzzle_id}
                level={level}
                index={index}
                isCompleted={isCompleted}
                isCurrent={isCurrent}
                isUnlocked={canSelect}
                stars={stars}
                onSelect={() => {
                  if (canSelect) onSelect(index);
                }}
              />
            );
          })}
        </div>

        {/* Footer -- reset progress */}
        <div className="px-6 py-3 border-t border-zinc-200 dark:border-zinc-800">
          {confirmReset ? (
            <div className="flex items-center justify-between">
              <span className="text-sm text-zinc-600 dark:text-zinc-300">
                Reset all progress and code for this world?
              </span>
              <div className="flex gap-2">
                <button
                  onClick={() => setConfirmReset(false)}
                  className="px-3 py-1 text-sm text-zinc-400 hover:text-zinc-900 dark:hover:text-white transition-colors cursor-pointer"
                >
                  Cancel
                </button>
                <button
                  onClick={() => {
                    setConfirmReset(false);
                    onReset();
                  }}
                  className="px-3 py-1 text-sm text-red-400 hover:text-red-300 font-medium transition-colors cursor-pointer"
                >
                  Reset
                </button>
              </div>
            </div>
          ) : (
            <button
              onClick={() => setConfirmReset(true)}
              className="flex items-center gap-1.5 text-xs text-zinc-500 hover:text-zinc-300 transition-colors cursor-pointer"
            >
              <RotateCcw className="h-3 w-3" />
              Reset progress
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function LevelCard({
  level,
  index,
  isCompleted,
  isCurrent,
  isUnlocked,
  stars,
  onSelect,
}: {
  level: LevelSummary;
  index: number;
  isCompleted: boolean;
  isCurrent: boolean;
  isUnlocked: boolean;
  stars: number;
  onSelect: () => void;
}) {
  const [preview, setPreview] = useState<PuzzleConfig | null>(null);

  useEffect(() => {
    if (!isUnlocked) return;
    try {
      const config = getLevel(level.puzzle_id);
      if (config) setPreview(config);
    } catch {
      // WASM not ready yet -- ignore
    }
  }, [level.puzzle_id, isUnlocked]);

  const borderColor = isCurrent
    ? "border-[var(--theme-400)]"
    : isCompleted
      ? "border-[var(--theme-300)]/25"
      : isUnlocked
        ? "border-zinc-300 dark:border-zinc-700"
        : "border-zinc-200 dark:border-zinc-800";

  return (
    <button
      onClick={onSelect}
      disabled={!isUnlocked}
      className={`w-full flex items-center gap-4 p-3 rounded-lg border ${borderColor} transition-colors text-left ${
        isUnlocked
          ? "bg-zinc-50 dark:bg-zinc-800/50 hover:bg-zinc-100 dark:hover:bg-zinc-800 cursor-pointer"
          : "bg-zinc-50/50 dark:bg-zinc-900/50 opacity-50 cursor-not-allowed"
      }`}
    >
      {/* Thumbnail */}
      <div className="shrink-0 w-20 h-20 flex items-center justify-center rounded bg-zinc-950 overflow-hidden">
        {preview ? (
          <GameGrid
            grid={preview.grid}
            botState={preview.bot_start}
            tileSize={10}
          />
        ) : isUnlocked ? (
          <div className="w-6 h-6 border-2 border-zinc-700 border-t-zinc-400 rounded-full animate-spin" />
        ) : (
          <Lock className="h-5 w-5 text-zinc-600" />
        )}
      </div>

      {/* Info */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-xs font-medium text-zinc-400">
            Level {index + 1}
          </span>
          {isCompleted && (
            <span className="flex items-center gap-0.5 text-xs text-emerald-300">
              <Check className="h-3 w-3 mr-1" />
              {[0, 1, 2].map((i) => (
                <Star
                  key={i}
                  className={`h-3 w-3 ${
                    i < stars
                      ? "text-yellow-300 fill-yellow-300"
                      : "text-zinc-600"
                  }`}
                />
              ))}
            </span>
          )}
          {isCurrent && !isCompleted && (
            <span className="text-xs text-[var(--theme-700)] dark:text-[var(--theme-400)]">Current</span>
          )}
        </div>
        <p className="text-sm font-semibold text-zinc-900 dark:text-white truncate">
          {level.title}
        </p>
        <p className="text-xs text-zinc-400 line-clamp-2">
          {level.description}
        </p>
      </div>

      {/* Lock icon for locked levels */}
      {!isUnlocked && (
        <Lock className="h-4 w-4 text-zinc-600 shrink-0" />
      )}
    </button>
  );
}
