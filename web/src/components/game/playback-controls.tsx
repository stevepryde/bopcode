import { Play, Pause, SkipForward, RotateCcw } from "lucide-react";
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
}: PlaybackControlsProps) {
  return (
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
      <div className="w-px h-5 bg-zinc-700 mx-0.5" />

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
      <div className="w-px h-5 bg-zinc-700 mx-0.5" />

      {/* Progress Display */}
      <div className="text-xs text-zinc-400 min-w-[60px] text-center">
        {totalActions > 0 ? (
          <>
            {currentAction} / {totalActions}
          </>
        ) : (
          "No actions"
        )}
      </div>
    </div>
  );
}
