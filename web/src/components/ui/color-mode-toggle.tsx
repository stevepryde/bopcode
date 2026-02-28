import { Sun, Moon } from "lucide-react";
import { useColorMode, setColorMode } from "@/lib/theme";

export function ColorModeToggle() {
  const mode = useColorMode();

  const toggle = () => {
    setColorMode(mode === "dark" ? "light" : "dark");
  };

  return (
    <button
      onClick={toggle}
      className="p-1.5 text-zinc-400 hover:text-zinc-900 dark:hover:text-white transition-colors cursor-pointer rounded-md hover:bg-zinc-200 dark:hover:bg-zinc-800"
      title={`Switch to ${mode === "dark" ? "light" : "dark"} mode`}
    >
      {mode === "dark" ? (
        <Sun className="h-4 w-4" />
      ) : (
        <Moon className="h-4 w-4" />
      )}
    </button>
  );
}
