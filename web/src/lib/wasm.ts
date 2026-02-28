import type {
  SimulationResult,
  WorldInfo,
  LevelSummary,
  PuzzleConfig,
} from "@/types/game";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
let wasmModule: any = null;

export async function initWasm(): Promise<void> {
  if (wasmModule) return;
  const wasm = await import("@/wasm/bopcode_wasm");
  await wasm.default();
  wasmModule = wasm;
}

export function runSimulation(
  puzzleId: string,
  code: string,
): SimulationResult {
  if (!wasmModule) throw new Error("WASM not initialized");
  return wasmModule.run_simulation(puzzleId, code) as SimulationResult;
}

export function runSimulationWithConfig(
  config: PuzzleConfig,
  code: string,
): SimulationResult {
  if (!wasmModule) throw new Error("WASM not initialized");
  return wasmModule.run_simulation_with_config(config, code) as SimulationResult;
}

export function getWorlds(): WorldInfo[] {
  if (!wasmModule) throw new Error("WASM not initialized");
  return wasmModule.get_worlds() as WorldInfo[];
}

export function getWorldLevels(worldId: string): LevelSummary[] {
  if (!wasmModule) throw new Error("WASM not initialized");
  return wasmModule.get_world_levels(worldId) as LevelSummary[];
}

export function getLevel(levelId: string): PuzzleConfig | null {
  if (!wasmModule) throw new Error("WASM not initialized");
  return wasmModule.get_level(levelId) as PuzzleConfig | null;
}
