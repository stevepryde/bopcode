import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { Star } from "lucide-react";
import { useMemo } from "react";
import { getWorlds, getWorldLevels } from "@/lib/wasm";
import { getProgress } from "@/lib/progress";
import type { WorldInfo } from "@/types/game";

export const Route = createFileRoute("/")({
  component: HomePage,
});

function HomePage() {
  const navigate = useNavigate();
  const worlds = useMemo(() => getWorlds(), []);
  const progress = useMemo(() => getProgress(), []);

  const worldLevelCounts = useMemo(() => {
    const counts: Record<string, string[]> = {};
    for (const w of worlds) {
      counts[w.world_id] = getWorldLevels(w.world_id).map((l) => l.puzzle_id);
    }
    return counts;
  }, [worlds]);

  function worldProgress(world: WorldInfo) {
    const levelIds = worldLevelCounts[world.world_id] ?? [];
    const total = levelIds.length;
    if (total === 0) return { completed: 0, total, stars: 0, maxStars: 0 };

    let completed = 0;
    let stars = 0;
    for (const levelId of levelIds) {
      const lp = progress.levels[levelId];
      if (lp?.completed) {
        completed++;
        stars += lp.stars;
      }
    }

    return { completed, total, stars, maxStars: total * 3 };
  }

  return (
    <div className="min-h-screen w-full bg-zinc-950 text-white">
      {/* Header */}
      <header className="bg-gradient-to-r from-violet-500/20 via-violet-950/30 to-zinc-950 border-b border-violet-500/20 px-6 py-4">
        <div className="max-w-5xl mx-auto flex items-center">
          <h1 className="text-xl font-bold tracking-tight bg-gradient-to-r from-violet-400 to-fuchsia-400 bg-clip-text text-transparent">
            bopcode
          </h1>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-5xl mx-auto px-6 py-10">
        {/* Hero */}
        <div className="mb-12">
          <h2 className="text-3xl font-bold mb-2">
            Welcome to bopcode
          </h2>
          <p className="text-zinc-400 text-lg">
            Learn to code by guiding a bot through puzzles.
          </p>
        </div>

        {/* Worlds Grid */}
        <section>
          <h3 className="text-sm font-semibold text-zinc-500 uppercase tracking-wider mb-4">
            Worlds
          </h3>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {worlds.map((world) => {
              const wp = worldProgress(world);
              const themeColors: Record<string, string> = {
                grassy_plains: "from-violet-600 to-purple-800",
                crystal_caves: "from-cyan-600 to-blue-800",
                abandoned_station: "from-zinc-600 to-slate-800",
                volcanic_islands: "from-orange-600 to-red-800",
                cloud_city: "from-sky-500 to-indigo-700",
              };
              const color =
                themeColors[world.theme] ?? "from-violet-600 to-purple-800";

              return (
                <button
                  key={world.world_id}
                  onClick={() =>
                    navigate({
                      to: "/play/$worldId",
                      params: { worldId: world.world_id },
                    })
                  }
                  className={`group relative overflow-hidden rounded-xl border border-violet-500/15 bg-gradient-to-br ${color} p-6 text-left transition-all hover:scale-[1.02] hover:shadow-lg hover:shadow-violet-500/15 cursor-pointer`}
                >
                  <h4 className="text-lg font-bold text-white mb-1">
                    {world.title}
                  </h4>
                  <p className="text-sm text-white/70 leading-relaxed">
                    {world.description}
                  </p>

                  {wp.total > 0 && (
                    <div className="mt-4 space-y-1.5">
                      <div className="flex items-center gap-2">
                        <div className="flex-1 h-1.5 bg-black/20 rounded-full overflow-hidden">
                          <div
                            className="h-full bg-white/60 rounded-full transition-all"
                            style={{
                              width: `${Math.min(100, (wp.completed / wp.total) * 100)}%`,
                            }}
                          />
                        </div>
                        <span className="text-xs text-white/60 font-medium">
                          {wp.completed}/{wp.total}
                        </span>
                      </div>
                      {wp.stars > 0 && (
                        <div className="flex items-center gap-1">
                          <Star className="h-3 w-3 text-yellow-400 fill-yellow-400" />
                          <span className="text-xs text-white/60 font-medium">
                            {wp.stars}/{wp.maxStars}
                          </span>
                        </div>
                      )}
                    </div>
                  )}
                </button>
              );
            })}
          </div>
        </section>
      </main>
    </div>
  );
}
