import { useRef, useCallback, useLayoutEffect, useState } from "react";
import { GameGrid } from "@/components/game/game-grid";
import { Tooltip } from "@/components/ui/tooltip";
import type { PuzzleConfig, TileType, TileItem, Direction } from "@/types/game";
import { resizeGrid } from "@/lib/editor-store";

export type EditorTool =
  | { kind: "tile"; tileType: TileType }
  | { kind: "item"; item: TileItem }
  | { kind: "bot" }
  | { kind: "eraser" };

const TOOL_PALETTE: { tool: EditorTool; label: string; color: string; description: string }[] = [
  { tool: { kind: "tile", tileType: "floor" }, label: "Floor", color: "#0f0f14", description: "Empty walkable tile" },
  { tool: { kind: "tile", tileType: "wall" }, label: "Wall", color: "#4a4a55", description: "Blocks bot movement" },
  { tool: { kind: "tile", tileType: "goal" }, label: "Goal", color: "#86efac", description: "The bot must reach this tile to win" },
  { tool: { kind: "tile", tileType: "pit" }, label: "Pit", color: "#1a1a22", description: "Bot falls in and fails the level" },
  { tool: { kind: "tile", tileType: "locked_door" }, label: "Door", color: "#c4b5fd", description: "Requires a key to open" },
  { tool: { kind: "tile", tileType: "gem_vault" }, label: "Gem Vault", color: "#f9a8d4", description: "Deposit gems here" },
  { tool: { kind: "tile", tileType: "diamond_vault" }, label: "Dia Vault", color: "#7dd3fc", description: "Deposit diamonds here" },
  { tool: { kind: "item", item: "gem" }, label: "Gem", color: "#d8b4fe", description: "Collectible item placed on a tile" },
  { tool: { kind: "item", item: "key" }, label: "Key", color: "#fde68a", description: "Unlocks locked doors" },
  { tool: { kind: "item", item: "diamond" }, label: "Diamond", color: "#7dd3fc", description: "Rare collectible placed on a tile" },
  { tool: { kind: "bot" }, label: "Bot", color: "#a5b4fc", description: "Set bot start position. Click again to rotate." },
  { tool: { kind: "eraser" }, label: "Eraser", color: "#71717a", description: "Reset tile to empty floor" },
];

function toolKey(tool: EditorTool): string {
  if (tool.kind === "tile") return `tile-${tool.tileType}`;
  if (tool.kind === "item") return `item-${tool.item}`;
  return tool.kind;
}

interface DesignTabProps {
  config: PuzzleConfig;
  onConfigChange: (config: PuzzleConfig) => void;
  selectedTool: EditorTool;
  onToolChange: (tool: EditorTool) => void;
}

const NEXT_DIR: Record<Direction, Direction> = {
  right: "down",
  down: "left",
  left: "up",
  up: "right",
};

export function DesignTab({ config, onConfigChange, selectedTool, onToolChange }: DesignTabProps) {
  const gridContainerRef = useRef<HTMLDivElement>(null);
  const [tileSize, setTileSize] = useState(48);
  const [gridOverflows, setGridOverflows] = useState(false);
  const isPaintingRef = useRef(false);

  const MIN_TILE = 32;

  useLayoutEffect(() => {
    const container = gridContainerRef.current;
    if (!container) return;

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry) return;
      const { width, height } = entry.contentRect;
      const padding = 16;
      const maxTileW = (width - padding) / config.grid.width;
      const maxTileH = (height - padding) / config.grid.height;
      const fitted = Math.floor(Math.min(maxTileW, maxTileH));
      if (fitted >= MIN_TILE) {
        setTileSize(fitted);
        setGridOverflows(false);
      } else {
        setTileSize(MIN_TILE);
        setGridOverflows(true);
      }
    });

    observer.observe(container);
    return () => observer.disconnect();
  }, [config.grid.width, config.grid.height]);

  const applyTool = useCallback(
    (gridX: number, gridY: number) => {
      const clone = structuredClone(config);
      const tile = clone.grid.tiles[gridY]?.[gridX];
      if (!tile) return;

      if (selectedTool.kind === "tile") {
        tile.tile_type = selectedTool.tileType;
        tile.item = undefined;
      } else if (selectedTool.kind === "item") {
        // Toggle item: if same item exists, remove it
        if (tile.item === selectedTool.item) {
          tile.item = undefined;
        } else {
          tile.item = selectedTool.item;
        }
      } else if (selectedTool.kind === "bot") {
        const botPos = clone.bot_start.position;
        if (botPos.x === gridX && botPos.y === gridY) {
          // Cycle direction
          clone.bot_start.direction = NEXT_DIR[clone.bot_start.direction];
        } else {
          clone.bot_start.position = { x: gridX, y: gridY };
        }
      } else if (selectedTool.kind === "eraser") {
        tile.tile_type = "floor";
        tile.item = undefined;
      }

      onConfigChange(clone);
    },
    [config, selectedTool, onConfigChange],
  );

  const getGridCoords = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      const rect = e.currentTarget.getBoundingClientRect();
      const x = Math.floor((e.clientX - rect.left) / tileSize);
      const y = Math.floor((e.clientY - rect.top) / tileSize);
      if (x < 0 || y < 0 || x >= config.grid.width || y >= config.grid.height) return null;
      return { x, y };
    },
    [tileSize, config.grid.width, config.grid.height],
  );

  const canDrag = selectedTool.kind === "tile" || selectedTool.kind === "eraser";

  const handleMouseDown = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      const coords = getGridCoords(e);
      if (!coords) return;
      isPaintingRef.current = true;
      applyTool(coords.x, coords.y);
    },
    [getGridCoords, applyTool],
  );

  const handleMouseMove = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      if (!isPaintingRef.current || !canDrag) return;
      const coords = getGridCoords(e);
      if (!coords) return;
      applyTool(coords.x, coords.y);
    },
    [getGridCoords, applyTool, canDrag],
  );

  const handleMouseUp = useCallback(() => {
    isPaintingRef.current = false;
  }, []);

  const handleWidthChange = (newW: number) => {
    const clamped = Math.max(1, Math.min(20, newW));
    onConfigChange(resizeGrid(config, clamped, config.grid.height));
  };

  const handleHeightChange = (newH: number) => {
    const clamped = Math.max(1, Math.min(20, newH));
    onConfigChange(resizeGrid(config, config.grid.width, clamped));
  };

  return (
    <div className="flex flex-col h-full gap-3">
      {/* Toolbar row: tool palette + grid size */}
      <div className="shrink-0 flex flex-wrap items-center gap-2">
        {/* Tool palette */}
        <div className="flex flex-wrap gap-1">
          {TOOL_PALETTE.map(({ tool, label, color, description }) => {
            const key = toolKey(tool);
            const isSelected = toolKey(selectedTool) === key;
            return (
              <Tooltip key={key} content={description}>
                <button
                  onClick={() => onToolChange(tool)}
                  className={`flex items-center gap-1.5 px-2 py-1 text-xs font-medium rounded-md border transition-colors cursor-pointer ${
                    isSelected
                      ? "border-[var(--theme-400)]/70 bg-[var(--theme-500)]/20 text-zinc-900 dark:text-white"
                      : "border-zinc-300 dark:border-zinc-700 bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 hover:border-zinc-400 dark:hover:border-zinc-600"
                  }`}
                >
                  <span
                    className="w-3 h-3 rounded-sm border border-zinc-600 shrink-0"
                    style={{ backgroundColor: color }}
                  />
                  {label}
                </button>
              </Tooltip>
            );
          })}
        </div>

        {/* Grid size controls */}
        <div className="flex items-center gap-2 ml-auto">
          <label className="text-xs text-zinc-400">Size:</label>
          <input
            type="number"
            min={1}
            max={20}
            value={config.grid.width}
            onChange={(e) => handleWidthChange(Number(e.target.value))}
            className="w-12 h-7 text-xs text-center bg-zinc-800 border border-zinc-700 rounded text-white"
          />
          <span className="text-xs text-zinc-500">x</span>
          <input
            type="number"
            min={1}
            max={20}
            value={config.grid.height}
            onChange={(e) => handleHeightChange(Number(e.target.value))}
            className="w-12 h-7 text-xs text-center bg-zinc-800 border border-zinc-700 rounded text-white"
          />
        </div>
      </div>

      {/* Grid area with mouse overlay */}
      <div
        ref={gridContainerRef}
        className={`flex-1 min-h-0 flex justify-center ${gridOverflows ? "overflow-auto items-start py-2" : "items-center"}`}
      >
        <div className="relative inline-block">
          <GameGrid grid={config.grid} botState={config.bot_start} tileSize={tileSize} />
          {/* Transparent mouse overlay */}
          <div
            className="absolute inset-0 cursor-crosshair"
            onMouseDown={handleMouseDown}
            onMouseMove={handleMouseMove}
            onMouseUp={handleMouseUp}
            onMouseLeave={handleMouseUp}
          />
        </div>
      </div>
    </div>
  );
}
