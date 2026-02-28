import { useState } from "react";
import { Play, Pause, SkipForward, RotateCcw, AlertTriangle, MessageSquareText, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { PlaybackSpeed } from "@/types/game";

interface PlaybackControlsProps {
  isPlaying: boolean;
  onPlay: () => void;
  onPause: () => void;
  onStep: () => void;
  onReset: () => void;
  speed: PlaybackSpeed;
  onSpeedChange: (speed: PlaybackSpeed) => void;
  currentAction: number;
  totalActions: number;
  disabled?: boolean;
  warnings?: string[];
  outputs?: string[];
}

const SPEEDS: PlaybackSpeed[] = [0.5, 1, 2, 4];

export function PlaybackControls({
  isPlaying,
  onPlay,
  onPause,
  onStep,
  onReset,
  speed,
  onSpeedChange,
  currentAction,
  totalActions,
  disabled = false,
  warnings = [],
  outputs = [],
}: PlaybackControlsProps) {
  const [showWarnings, setShowWarnings] = useState(false);
  const [showOutputs, setShowOutputs] = useState(false);

  return (
    <>
      <div className="flex items-center gap-1.5 px-2 py-1 bg-zinc-100 dark:bg-zinc-900 rounded-lg border border-zinc-200 dark:border-zinc-800">
        {/* Play/Pause Button */}
        <Button
          variant="ghost"
          size="sm"
          onClick={isPlaying ? onPause : onPlay}
          disabled={disabled || totalActions === 0}
          className="h-7 w-7 p-0"
        >
          {isPlaying ? (
            <Pause className="h-3.5 w-3.5" />
          ) : (
            <Play className="h-3.5 w-3.5" />
          )}
        </Button>

        {/* Step Button */}
        <Button
          variant="ghost"
          size="sm"
          onClick={onStep}
          disabled={disabled || isPlaying || currentAction >= totalActions}
          className="h-7 w-7 p-0"
          title="Step forward"
        >
          <SkipForward className="h-3.5 w-3.5" />
        </Button>

        {/* Reset Button */}
        <Button
          variant="ghost"
          size="sm"
          onClick={onReset}
          disabled={disabled}
          className="h-7 w-7 p-0"
          title="Reset"
        >
          <RotateCcw className="h-3.5 w-3.5" />
        </Button>

        {/* Divider */}
        <div className="w-px h-5 bg-zinc-300 dark:bg-zinc-700 mx-0.5" />

        {/* Speed Selector */}
        <div className="flex items-center gap-0.5">
          {SPEEDS.map((s) => (
            <Button
              key={s}
              variant={speed === s ? "secondary" : "ghost"}
              size="sm"
              onClick={() => onSpeedChange(s)}
              disabled={disabled}
              className="h-6 px-1.5 text-xs"
            >
              {s}x
            </Button>
          ))}
        </div>

        {/* Divider */}
        <div className="w-px h-5 bg-zinc-300 dark:bg-zinc-700 mx-0.5" />

        {/* Progress Display */}
        <div className="text-xs text-zinc-500 dark:text-zinc-400 min-w-[60px] text-center">
          {totalActions > 0 ? (
            <>
              {currentAction} / {totalActions}
            </>
          ) : (
            "No actions"
          )}
        </div>

        {/* Output & warning indicators */}
        {(outputs.length > 0 || warnings.length > 0) && (
          <>
            <div className="w-px h-5 bg-zinc-300 dark:bg-zinc-700 mx-0.5" />
            {outputs.length > 0 && (
              <button
                onClick={() => setShowOutputs(true)}
                className="relative flex items-center gap-1 px-1.5 py-0.5 text-xs font-medium text-[var(--theme-600)] dark:text-[var(--theme-400)] hover:bg-[var(--theme-500)]/10 rounded transition-colors cursor-pointer"
                title="View output"
              >
                <MessageSquareText className="h-3.5 w-3.5" />
                <span>{outputs.length}</span>
              </button>
            )}
            {warnings.length > 0 && (
              <button
                onClick={() => setShowWarnings(true)}
                className="relative flex items-center gap-1 px-1.5 py-0.5 text-xs font-medium text-amber-600 dark:text-amber-400 hover:bg-amber-100 dark:hover:bg-amber-900/30 rounded transition-colors cursor-pointer"
                title="View warnings"
              >
                <AlertTriangle className="h-3.5 w-3.5" />
                <span>{warnings.length}</span>
              </button>
            )}
          </>
        )}
      </div>

      {/* Output modal */}
      {showOutputs && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={() => setShowOutputs(false)}>
          <div
            className="bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 rounded-xl shadow-xl max-w-md w-full mx-4 overflow-hidden"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-200 dark:border-zinc-800">
              <div className="flex items-center gap-2">
                <MessageSquareText className="h-4 w-4 text-[var(--theme-500)]" />
                <h3 className="text-sm font-semibold text-zinc-900 dark:text-white">
                  Output ({outputs.length})
                </h3>
              </div>
              <button
                onClick={() => setShowOutputs(false)}
                className="p-1 text-zinc-400 hover:text-zinc-900 dark:hover:text-white transition-colors cursor-pointer"
              >
                <X className="h-4 w-4" />
              </button>
            </div>
            <div className="px-4 py-3 max-h-60 overflow-y-auto space-y-1">
              {outputs.map((msg, i) => (
                <div
                  key={i}
                  className="font-mono text-sm text-zinc-700 dark:text-zinc-300"
                >
                  {msg}
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Warnings modal */}
      {showWarnings && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={() => setShowWarnings(false)}>
          <div
            className="bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 rounded-xl shadow-xl max-w-md w-full mx-4 overflow-hidden"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-200 dark:border-zinc-800">
              <div className="flex items-center gap-2">
                <AlertTriangle className="h-4 w-4 text-amber-500" />
                <h3 className="text-sm font-semibold text-zinc-900 dark:text-white">
                  Warnings ({warnings.length})
                </h3>
              </div>
              <button
                onClick={() => setShowWarnings(false)}
                className="p-1 text-zinc-400 hover:text-zinc-900 dark:hover:text-white transition-colors cursor-pointer"
              >
                <X className="h-4 w-4" />
              </button>
            </div>
            <div className="px-4 py-3 max-h-60 overflow-y-auto space-y-2">
              {warnings.map((msg, i) => (
                <div
                  key={i}
                  className="flex items-start gap-2 text-sm text-zinc-700 dark:text-zinc-300"
                >
                  <span className="shrink-0 text-amber-500 dark:text-amber-400 font-medium text-xs mt-0.5">
                    {i + 1}.
                  </span>
                  <span>{msg}</span>
                </div>
              ))}
            </div>
            <div className="px-4 py-3 border-t border-zinc-200 dark:border-zinc-800">
              <p className="text-xs text-zinc-500 dark:text-zinc-400">
                Use <code className="px-1 py-0.5 bg-zinc-100 dark:bg-zinc-800 rounded text-[var(--theme-700)] dark:text-[var(--theme-300)]">wall_ahead()</code> to check before moving.
              </p>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
