import { useState, useEffect, useRef, useCallback } from "react";
import { useNavigate } from "@tanstack/react-router";
import { ArrowLeft, Download, Upload, Check } from "lucide-react";
import type { PuzzleConfig } from "@/types/game";
import {
  createDefaultConfig,
  saveEditorState,
  loadEditorState,
  downloadConfigAsJson,
  importConfigFromJson,
} from "@/lib/editor-store";
import { ConfirmModal } from "@/components/ui/confirm-modal";
import { ColorModeToggle } from "@/components/ui/color-mode-toggle";
import { BopIcon } from "@/components/ui/bop-icon";
import { WORLD_THEME_CLASS } from "@/lib/theme";
import { DesignTab, type EditorTool } from "./design-tab";
import { ConfigTab } from "./config-tab";
import { TestTab } from "./test-tab";

type TabId = "design" | "config" | "test";
type PendingAction = "new" | "import" | null;

export function LevelEditor() {
  const navigate = useNavigate();
  const [config, setConfig] = useState<PuzzleConfig>(() => loadEditorState() ?? createDefaultConfig());
  const [activeTab, setActiveTab] = useState<TabId>("config");
  const [selectedTool, setSelectedTool] = useState<EditorTool>({ kind: "tile", tileType: "wall" });
  const [saved, setSaved] = useState(true);
  const saveTimerRef = useRef<number | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [pendingAction, setPendingAction] = useState<PendingAction>(null);

  // Auto-save with 1s debounce
  const handleConfigChange = useCallback((newConfig: PuzzleConfig) => {
    setConfig(newConfig);
    setSaved(false);
  }, []);

  useEffect(() => {
    if (saved) return;
    if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
    saveTimerRef.current = window.setTimeout(() => {
      saveEditorState(config);
      setSaved(true);
    }, 1000);
    return () => {
      if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
    };
  }, [config, saved]);

  const doImport = async (file: File) => {
    try {
      const imported = await importConfigFromJson(file);
      setConfig(imported);
      saveEditorState(imported);
      setSaved(true);
    } catch (err) {
      alert(err instanceof Error ? err.message : "Failed to import file");
    }
  };

  const handleImportFile = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    await doImport(file);
    e.target.value = "";
  };

  const handleImportClick = () => {
    setPendingAction("import");
  };

  const handleNewClick = () => {
    setPendingAction("new");
  };

  const handleConfirm = () => {
    if (pendingAction === "new") {
      const fresh = createDefaultConfig();
      setConfig(fresh);
      saveEditorState(fresh);
      setSaved(true);
      setPendingAction(null);
    } else if (pendingAction === "import") {
      setPendingAction(null);
      fileInputRef.current?.click();
    }
  };

  const handleDownload = () => {
    downloadConfigAsJson(config);
  };

  const tabs: { id: TabId; label: string }[] = [
    { id: "config", label: "Config" },
    { id: "design", label: "Design" },
    { id: "test", label: "Test" },
  ];

  return (
    <div className={`flex flex-col h-screen bg-white dark:bg-zinc-950 text-zinc-900 dark:text-white ${WORLD_THEME_CLASS[config.theme ?? "cloud_city"] ?? ""}`}>
      {/* Header */}
      <header className="shrink-0 flex items-center gap-3 px-4 py-3 bg-zinc-50 dark:bg-zinc-900 border-b border-zinc-200 dark:border-zinc-800">
        <button
          onClick={() => navigate({ to: "/" })}
          className="p-1.5 text-zinc-400 hover:text-zinc-900 dark:hover:text-white transition-colors cursor-pointer"
          title="Back to home"
        >
          <ArrowLeft className="h-5 w-5" />
        </button>

        <h1 className="flex items-center gap-2 text-lg font-bold text-indigo-400">
          <BopIcon className="h-5 w-5" />
          Level Editor
        </h1>

        {/* Save indicator */}
        <span className="text-xs text-zinc-500 flex items-center gap-1">
          {saved ? (
            <>
              <Check className="h-3 w-3 text-emerald-400" />
              Saved
            </>
          ) : (
            "Unsaved"
          )}
        </span>

        <div className="ml-auto flex items-center gap-2">
          <button
            onClick={handleNewClick}
            className="px-3 py-1.5 text-xs font-medium text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-white bg-zinc-100 dark:bg-zinc-800 border border-zinc-300 dark:border-zinc-700 rounded-md transition-colors cursor-pointer"
          >
            New
          </button>
          <button
            onClick={handleImportClick}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-white bg-zinc-100 dark:bg-zinc-800 border border-zinc-300 dark:border-zinc-700 rounded-md transition-colors cursor-pointer"
          >
            <Upload className="h-3.5 w-3.5" />
            Import
          </button>
          <input
            ref={fileInputRef}
            type="file"
            accept=".json"
            onChange={handleImportFile}
            className="hidden"
          />
          <button
            onClick={handleDownload}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-white bg-zinc-100 dark:bg-zinc-800 border border-zinc-300 dark:border-zinc-700 rounded-md transition-colors cursor-pointer"
          >
            <Download className="h-3.5 w-3.5" />
            Download
          </button>
          <ColorModeToggle />
        </div>
      </header>

      {/* Tab bar */}
      <div className="shrink-0 flex gap-1 px-4 pt-2 bg-white dark:bg-zinc-950">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`px-4 py-2 text-sm font-medium rounded-t-md transition-colors cursor-pointer ${
              activeTab === tab.id
                ? "bg-[var(--theme-500)]/20 text-[var(--theme-700)] dark:text-[var(--theme-300)] border-b-2 border-[var(--theme-500)] dark:border-[var(--theme-400)]/70"
                : "text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200"
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      <div className="flex-1 min-h-0 p-4">
        {activeTab === "design" && (
          <DesignTab
            config={config}
            onConfigChange={handleConfigChange}
            selectedTool={selectedTool}
            onToolChange={setSelectedTool}
          />
        )}
        {activeTab === "config" && (
          <ConfigTab config={config} onConfigChange={handleConfigChange} />
        )}
        {activeTab === "test" && (
          <TestTab config={config} onConfigChange={handleConfigChange} />
        )}
      </div>

      {/* Confirmation modal */}
      {pendingAction && (
        <ConfirmModal
          title={pendingAction === "new" ? "Create new puzzle?" : "Import puzzle?"}
          description="This will replace your current puzzle. Any unsaved changes will be lost."
          confirmLabel={pendingAction === "new" ? "Create New" : "Import"}
          onConfirm={handleConfirm}
          onCancel={() => setPendingAction(null)}
        />
      )}
    </div>
  );
}
