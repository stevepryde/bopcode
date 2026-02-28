import { Check, ChevronDown, ChevronRight, Star } from "lucide-react";
import type { BotState, Grid, PuzzleObjective } from "@/types/game";

interface PuzzleObjectivePanelProps {
  title: string;
  description: string;
  completion: PuzzleObjective;
  starObjectives: PuzzleObjective[];
  starsMet: boolean[];
  puzzleCompleted: boolean;
  hint?: string | null;
  showHint?: boolean;
  goalContext?: GoalEvaluationContext | null;
  collapsed?: boolean;
  onToggle?: () => void;
}

export interface GoalEvaluationContext {
  botState: BotState;
  grid: Grid;
  instructions: number;
  steps: number;
}

function tileHasGem(tile: { item?: string | null }): boolean {
  return tile.item === "gem";
}

function tileHasDiamond(tile: { item?: string | null }): boolean {
  return tile.item === "diamond";
}

function flattenGoals(objective: PuzzleObjective): PuzzleObjective[] {
  if (objective.type === "all") {
    return objective.conditions.flatMap(flattenGoals);
  }
  return [objective];
}

function isObjectiveMet(objective: PuzzleObjective, context: GoalEvaluationContext): boolean {
  const { botState, grid, instructions, steps } = context;
  switch (objective.type) {
    case "reach_position":
      return botState.position.x === objective.x && botState.position.y === objective.y;
    case "collect_all_gems":
      return !grid.tiles.some((row) => row.some((tile) => tileHasGem(tile)));
    case "deposit_all_gems":
      return (
        !grid.tiles.some((row) => row.some((tile) => tileHasGem(tile))) &&
        botState.gems === 0
      );
    case "deposit_all_diamonds":
      return (
        !grid.tiles.some((row) => row.some((tile) => tileHasDiamond(tile))) &&
        botState.diamonds === 0
      );
    case "max_instructions":
      return instructions <= objective.instructions;
    case "max_steps":
      return steps <= objective.steps;
    case "all":
      return objective.conditions.every((condition) =>
        isObjectiveMet(condition, context),
      );
    default:
      return false;
  }
}

function getObjectiveText(objective: PuzzleObjective): string {
  const joinNatural = (parts: string[]): string => {
    if (parts.length === 0) return "Unknown objective";
    if (parts.length === 1) return parts[0] ?? "Unknown objective";
    if (parts.length === 2) return `${parts[0]} and ${parts[1]}`;
    const head = parts.slice(0, -1).join(", ");
    const tail = parts[parts.length - 1] ?? "Unknown objective";
    return `${head}, and ${tail}`;
  };

  switch (objective.type) {
    case "reach_position":
      return "Reach the finish";
    case "collect_all_gems":
      return "Collect all gems";
    case "deposit_all_gems":
      return "Deposit all gems at gem vaults";
    case "deposit_all_diamonds":
      return "Deposit all diamonds at diamond vaults";
    case "max_instructions":
      return `Use ${objective.instructions} instructions or fewer`;
    case "max_steps":
      return `Complete in ${objective.steps} steps or fewer`;
    case "all": {
      const parts = flattenGoals(objective).map(getObjectiveText);
      if (
        parts.length === 2 &&
        parts.includes("Collect all gems") &&
        parts.includes("Reach the finish")
      ) {
        return "Collect all gems and reach the finish";
      }
      return joinNatural(parts);
    }
    default:
      return "Unknown objective";
  }
}

export function PuzzleObjectivePanel({
  title,
  description,
  completion,
  starObjectives,
  starsMet,
  puzzleCompleted,
  hint,
  showHint = false,
  goalContext = null,
  collapsed = false,
  onToggle,
}: PuzzleObjectivePanelProps) {
  const goals = flattenGoals(completion);
  const goalStatuses = goals.map((goal) =>
    goalContext ? isObjectiveMet(goal, goalContext) : puzzleCompleted,
  );
  return (
    <div className="bg-zinc-50 dark:bg-zinc-900 rounded-lg p-4 space-y-4 border border-zinc-200 dark:border-zinc-800 border-l-2 border-l-[var(--theme-400)]/35">
      {/* Title -- clickable to collapse */}
      <button
        onClick={onToggle}
        className="flex items-center gap-2 w-full text-left cursor-pointer"
        title={collapsed ? description : undefined}
      >
        {onToggle &&
          (collapsed ? (
            <ChevronRight className="h-5 w-5 text-zinc-400 shrink-0" />
          ) : (
            <ChevronDown className="h-5 w-5 text-zinc-400 shrink-0" />
          ))}
        <h2 className="text-xl font-bold text-zinc-900 dark:text-white">{title}</h2>

        {/* Collapsed inline summary: goal + stars */}
        {collapsed && (
          <div className="flex items-center gap-2 ml-auto shrink-0">
            {/* Star icons (completion star + bonus stars) */}
            <div className="flex gap-0.5">
              <Star
                className={`h-4 w-4 ${
                  puzzleCompleted
                    ? "text-yellow-300 fill-yellow-300"
                    : "text-zinc-600"
                }`}
              />
              {starObjectives.map((_, index) => {
                const isMet = starsMet[index] ?? false;
                return (
                  <Star
                    key={index}
                    className={`h-4 w-4 ${
                      isMet
                        ? "text-yellow-300 fill-yellow-300"
                        : "text-zinc-600"
                    }`}
                  />
                );
              })}
            </div>
          </div>
        )}
      </button>

      {!collapsed && (
        <>
          <p className="text-sm text-zinc-500 dark:text-zinc-400">{description}</p>

          {/* Completion objective */}
          <div className="space-y-2">
            <h3 className="text-sm font-semibold text-zinc-500 dark:text-zinc-300 uppercase tracking-wide">
              {goals.length > 1 ? "Objectives" : "Objective"}
            </h3>
            <ul className="space-y-2">
              {goals.map((goal, index) => {
                const isMet = goalStatuses[index] ?? false;
                return (
                  <li
                    key={`${goal.type}-${index}`}
                    className={`flex items-center gap-2 text-sm ${
                      isMet ? "text-emerald-600 dark:text-emerald-300" : "text-zinc-600 dark:text-zinc-300"
                    }`}
                  >
                    {isMet ? (
                      <Check className="h-4 w-4 text-emerald-600 dark:text-emerald-300" />
                    ) : (
                      <div className="h-4 w-4 rounded-full border-2 border-zinc-500" />
                    )}
                    <span className={isMet ? "line-through opacity-70" : ""}>
                      {getObjectiveText(goal)}
                    </span>
                  </li>
                );
              })}
            </ul>
          </div>

          {/* Star objectives */}
          <div className="space-y-2">
            <h3 className="text-sm font-semibold text-zinc-500 dark:text-zinc-300 uppercase tracking-wide">
              Stars
            </h3>
            <ul className="space-y-2">
              {/* Star 1: Complete all objectives */}
              <li
                className={`flex items-center gap-2 text-sm ${
                  puzzleCompleted ? "text-yellow-600 dark:text-yellow-300" : "text-zinc-600 dark:text-zinc-300"
                }`}
              >
                <Star
                  className={`h-4 w-4 ${
                    puzzleCompleted
                      ? "text-yellow-300 fill-yellow-300"
                      : "text-zinc-500"
                  }`}
                />
                <span className={puzzleCompleted ? "line-through opacity-70" : ""}>
                  Complete all objectives
                </span>
              </li>
              {/* Bonus stars from star_objectives */}
              {starObjectives.map((objective, index) => {
                const isMet = starsMet[index] ?? false;
                return (
                  <li
                    key={index}
                    className={`flex items-center gap-2 text-sm ${
                      isMet ? "text-yellow-600 dark:text-yellow-300" : "text-zinc-600 dark:text-zinc-300"
                    }`}
                  >
                    <Star
                      className={`h-4 w-4 ${
                        isMet
                          ? "text-yellow-300 fill-yellow-300"
                          : "text-zinc-500"
                      }`}
                    />
                    <span className={isMet ? "line-through opacity-70" : ""}>
                      {getObjectiveText(objective)}
                    </span>
                  </li>
                );
              })}
            </ul>
          </div>

          {/* Hint */}
          {showHint && hint && (
            <div className="pt-2 border-t border-zinc-200 dark:border-zinc-800">
              <p className="text-xs text-amber-600 dark:text-amber-300">
                <strong>Hint:</strong> {hint}
              </p>
            </div>
          )}
        </>
      )}
    </div>
  );
}
