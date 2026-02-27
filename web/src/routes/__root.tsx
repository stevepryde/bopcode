import { createRootRoute, Outlet } from "@tanstack/react-router";

export const Route = createRootRoute({
  component: () => (
    <div className="min-h-screen bg-zinc-900 text-zinc-100">
      <Outlet />
    </div>
  ),
});
