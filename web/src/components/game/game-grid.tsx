import { useRef, useEffect } from "react";
import type { Grid, BotState, Direction } from "@/types/game";

interface GameGridProps {
  grid: Grid;
  botState: BotState;
  tileSize?: number;
  className?: string;
}

const COLORS = {
  floor: "#0f0f14", // soft dark
  wall: "#4a4a55", // muted slate
  goal: "#86efac", // soft mint green
  goalGlow: "#bbf7d0", // lighter mint
  gem: "#d8b4fe", // soft lavender
  gemGlow: "#e9d5ff", // lighter lavender
  bot: "#a5b4fc", // soft periwinkle
  botOutline: "#c7d2fe", // lighter periwinkle
  gridLine: "#3b3b47", // subtle grid
  pit: "#1a1a22", // dark pit
  pitBorder: "#fca5a5", // soft coral
  lockedDoor: "#c4b5fd", // soft purple
  lockedDoorGlow: "#ddd6fe", // lighter purple
  gemVault: "#f9a8d4", // soft pink
  gemVaultGlow: "#fbcfe8", // lighter pink
  diamondVault: "#7dd3fc", // soft sky
  diamondVaultGlow: "#bae6fd", // lighter sky
  key: "#fde68a", // soft gold
  keyGlow: "#fef9c3", // lighter gold
  diamond: "#7dd3fc", // soft sky
  diamondGlow: "#bae6fd", // lighter sky
};

export function GameGrid({
  grid,
  botState,
  tileSize = 48,
  className = "",
}: GameGridProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const width = grid.width * tileSize;
    const height = grid.height * tileSize;

    // Set canvas size
    canvas.width = width;
    canvas.height = height;

    // Clear canvas
    ctx.fillStyle = COLORS.floor;
    ctx.fillRect(0, 0, width, height);

    // Draw tiles
    for (let y = 0; y < grid.height; y++) {
      for (let x = 0; x < grid.width; x++) {
        const row = grid.tiles[y];
        if (!row) continue;
        const tile = row[x];
        if (!tile) continue;

        const px = x * tileSize;
        const py = y * tileSize;

        // Draw tile background
        if (tile.tile_type === "wall") {
          ctx.fillStyle = COLORS.wall;
          ctx.fillRect(px, py, tileSize, tileSize);
        } else if (tile.tile_type === "goal") {
          ctx.fillStyle = COLORS.goal;
          ctx.fillRect(px + 2, py + 2, tileSize - 4, tileSize - 4);
          ctx.strokeStyle = COLORS.goalGlow;
          ctx.lineWidth = 2;
          ctx.strokeRect(px + 4, py + 4, tileSize - 8, tileSize - 8);
        } else if (tile.tile_type === "pit") {
          ctx.fillStyle = COLORS.pit;
          ctx.fillRect(px + 1, py + 1, tileSize - 2, tileSize - 2);
          ctx.strokeStyle = COLORS.pitBorder;
          ctx.lineWidth = 2;
          ctx.strokeRect(px + 2, py + 2, tileSize - 4, tileSize - 4);
        } else if (tile.tile_type === "locked_door") {
          ctx.fillStyle = COLORS.lockedDoor;
          ctx.fillRect(px + 2, py + 2, tileSize - 4, tileSize - 4);
          ctx.strokeStyle = COLORS.lockedDoorGlow;
          ctx.lineWidth = 1;
          ctx.strokeRect(px + 4, py + 4, tileSize - 8, tileSize - 8);
          // Keyhole indicator
          const kcx = px + tileSize / 2;
          const kcy = py + tileSize / 2;
          ctx.beginPath();
          ctx.arc(kcx, kcy - tileSize * 0.05, tileSize * 0.08, 0, Math.PI * 2);
          ctx.fillStyle = COLORS.key;
          ctx.fill();
          ctx.fillRect(kcx - tileSize * 0.03, kcy, tileSize * 0.06, tileSize * 0.12);
        } else if (tile.tile_type === "gem_vault") {
          ctx.fillStyle = COLORS.gemVault;
          const pad = tileSize * 0.15;
          ctx.beginPath();
          ctx.arc(px + tileSize / 2, py + tileSize / 2, tileSize / 2 - pad, 0, Math.PI * 2);
          ctx.fill();
          ctx.strokeStyle = COLORS.gemVaultGlow;
          ctx.lineWidth = 2;
          ctx.stroke();
        } else if (tile.tile_type === "diamond_vault") {
          ctx.fillStyle = COLORS.diamondVault;
          const pad = tileSize * 0.15;
          ctx.beginPath();
          ctx.arc(px + tileSize / 2, py + tileSize / 2, tileSize / 2 - pad, 0, Math.PI * 2);
          ctx.fill();
          ctx.strokeStyle = COLORS.diamondVaultGlow;
          ctx.lineWidth = 2;
          ctx.stroke();
        }

        // Draw item if present
        const item = tile.item ?? null;
        if (item === "gem") {
          drawGem(ctx, px + tileSize / 2, py + tileSize / 2, tileSize * 0.3);
        } else if (item === "key") {
          drawKey(ctx, px + tileSize / 2, py + tileSize / 2, tileSize * 0.3);
        } else if (item === "diamond") {
          drawDiamond(ctx, px + tileSize / 2, py + tileSize / 2, tileSize * 0.3);
        }

        // Draw grid lines
        ctx.strokeStyle = COLORS.gridLine;
        ctx.lineWidth = 1;
        ctx.strokeRect(px, py, tileSize, tileSize);
      }
    }

    // Draw bot
    drawBot(
      ctx,
      botState.position.x * tileSize + tileSize / 2,
      botState.position.y * tileSize + tileSize / 2,
      tileSize * 0.35,
      botState.direction
    );

    // Draw speech bubble if bot has a message
    if (botState.message) {
      const onTopRow = botState.position.y === 0;
      drawSpeechBubble(
        ctx,
        botState.position.x * tileSize + tileSize / 2,
        onTopRow
          ? botState.position.y * tileSize + tileSize + 10
          : botState.position.y * tileSize - 10,
        botState.message,
        onTopRow
      );
    }
  }, [grid, botState, tileSize]);

  return (
    <div className={`inline-block rounded-lg overflow-hidden border border-zinc-300 dark:border-zinc-600/40 shadow-lg ${className}`}>
      <canvas
        ref={canvasRef}
        style={{
          display: "block",
          imageRendering: "pixelated",
        }}
      />
    </div>
  );
}

function drawGem(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  radius: number
) {
  // Draw gem as a hexagon with glow
  ctx.beginPath();
  for (let i = 0; i < 6; i++) {
    const angle = (i * Math.PI) / 3 - Math.PI / 2;
    const x = cx + radius * Math.cos(angle);
    const y = cy + radius * Math.sin(angle);
    if (i === 0) {
      ctx.moveTo(x, y);
    } else {
      ctx.lineTo(x, y);
    }
  }
  ctx.closePath();

  // Glow effect
  ctx.shadowColor = COLORS.gemGlow;
  ctx.shadowBlur = 8;
  ctx.fillStyle = COLORS.gem;
  ctx.fill();

  // Inner highlight
  ctx.shadowBlur = 0;
  ctx.beginPath();
  for (let i = 0; i < 6; i++) {
    const angle = (i * Math.PI) / 3 - Math.PI / 2;
    const x = cx + radius * 0.5 * Math.cos(angle);
    const y = cy + radius * 0.5 * Math.sin(angle);
    if (i === 0) {
      ctx.moveTo(x, y);
    } else {
      ctx.lineTo(x, y);
    }
  }
  ctx.closePath();
  ctx.fillStyle = COLORS.gemGlow;
  ctx.globalAlpha = 0.5;
  ctx.fill();
  ctx.globalAlpha = 1;
}

function drawKey(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  radius: number
) {
  ctx.shadowColor = COLORS.keyGlow;
  ctx.shadowBlur = 6;
  ctx.fillStyle = COLORS.key;

  // Key head (circle)
  ctx.beginPath();
  ctx.arc(cx, cy - radius * 0.3, radius * 0.4, 0, Math.PI * 2);
  ctx.fill();

  // Key shaft
  const shaftW = radius * 0.15;
  ctx.fillRect(cx - shaftW, cy - radius * 0.1, shaftW * 2, radius * 0.7);

  // Key teeth
  ctx.fillRect(cx, cy + radius * 0.2, radius * 0.25, shaftW);
  ctx.fillRect(cx, cy + radius * 0.45, radius * 0.2, shaftW);

  ctx.shadowBlur = 0;
}

function drawDiamond(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  radius: number
) {
  ctx.shadowColor = COLORS.diamondGlow;
  ctx.shadowBlur = 8;
  ctx.fillStyle = COLORS.diamond;

  // Diamond shape (rotated square)
  ctx.beginPath();
  ctx.moveTo(cx, cy - radius);
  ctx.lineTo(cx + radius * 0.7, cy);
  ctx.lineTo(cx, cy + radius);
  ctx.lineTo(cx - radius * 0.7, cy);
  ctx.closePath();
  ctx.fill();

  // Inner highlight
  ctx.shadowBlur = 0;
  ctx.fillStyle = COLORS.diamondGlow;
  ctx.globalAlpha = 0.4;
  ctx.beginPath();
  ctx.moveTo(cx, cy - radius * 0.5);
  ctx.lineTo(cx + radius * 0.35, cy);
  ctx.lineTo(cx, cy + radius * 0.5);
  ctx.lineTo(cx - radius * 0.35, cy);
  ctx.closePath();
  ctx.fill();
  ctx.globalAlpha = 1;
}

function drawBot(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  radius: number,
  direction: Direction
) {
  // Rotation based on direction
  const rotations: Record<Direction, number> = {
    up: -Math.PI / 2,
    right: 0,
    down: Math.PI / 2,
    left: Math.PI,
  };
  const rotation = rotations[direction];

  ctx.save();
  ctx.translate(cx, cy);
  ctx.rotate(rotation);

  // Draw bot as a triangle pointing right
  ctx.beginPath();
  ctx.moveTo(radius, 0); // Tip
  ctx.lineTo(-radius * 0.7, -radius * 0.7); // Top-left
  ctx.lineTo(-radius * 0.7, radius * 0.7); // Bottom-left
  ctx.closePath();

  // Glow effect
  ctx.shadowColor = COLORS.botOutline;
  ctx.shadowBlur = 10;
  ctx.fillStyle = COLORS.bot;
  ctx.fill();

  // Outline
  ctx.shadowBlur = 0;
  ctx.strokeStyle = COLORS.botOutline;
  ctx.lineWidth = 2;
  ctx.stroke();

  // Eye
  ctx.beginPath();
  ctx.arc(radius * 0.2, 0, radius * 0.15, 0, Math.PI * 2);
  ctx.fillStyle = "#fff";
  ctx.fill();

  ctx.restore();
}

function drawSpeechBubble(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  message: string,
  below = false
) {
  const padding = 8;
  ctx.font = "12px sans-serif";
  const metrics = ctx.measureText(message);
  const width = metrics.width + padding * 2;
  const height = 20;

  const bubbleX = x - width / 2;
  const bubbleY = below ? y + 5 : y - height - 5;

  // Bubble background
  ctx.fillStyle = "#fff";
  ctx.beginPath();
  ctx.roundRect(bubbleX, bubbleY, width, height, 4);
  ctx.fill();

  // Bubble tail
  ctx.beginPath();
  if (below) {
    ctx.moveTo(x - 5, bubbleY);
    ctx.lineTo(x, bubbleY - 5);
    ctx.lineTo(x + 5, bubbleY);
  } else {
    ctx.moveTo(x - 5, bubbleY + height);
    ctx.lineTo(x, bubbleY + height + 5);
    ctx.lineTo(x + 5, bubbleY + height);
  }
  ctx.fill();

  // Text
  ctx.fillStyle = "#000";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(message, x, bubbleY + height / 2);
}
