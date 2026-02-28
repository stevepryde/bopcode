import { createFileRoute } from "@tanstack/react-router";
import { LevelEditor } from "@/components/editor/level-editor";

export const Route = createFileRoute("/editor")({
  component: LevelEditor,
});
