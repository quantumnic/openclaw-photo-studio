import { createSignal, Show, onMount, onCleanup } from "solid-js";
import { TopBar } from "./TopBar";
import { LeftSidebar } from "./LeftSidebar";
import { RightSidebar } from "./RightSidebar";
import { MainView } from "./MainView";
import { Filmstrip } from "./Filmstrip";
import { CommandPalette } from "../common/CommandPalette";
import { DiagnosticsView } from "../diagnostics/DiagnosticsView";
import { ToastContainer } from "../common/Toast";
import { ShortcutEngine } from "../../lib/shortcuts";

type Module = "library" | "develop" | "map" | "print";
type ViewMode = "grid" | "loupe" | "compare" | "survey";

interface AppShellProps {
  version?: any;
}

export function AppShell(props: AppShellProps) {
  const [activeModule, setActiveModule] = createSignal<Module>("library");
  const [viewMode, setViewMode] = createSignal<ViewMode>("grid");
  const [leftOpen, setLeftOpen] = createSignal(true);
  const [rightOpen, setRightOpen] = createSignal(true);
  const [filmstripOpen, setFilmstripOpen] = createSignal(true);
  const [selectedPhotoId, setSelectedPhotoId] = createSignal<string | null>(null);
  const [selectedPhotoIds, setSelectedPhotoIds] = createSignal<string[]>([]);
  const [allPhotoIds, setAllPhotoIds] = createSignal<string[]>([]);
  const [commandPaletteOpen, setCommandPaletteOpen] = createSignal(false);
  const [diagnosticsOpen, setDiagnosticsOpen] = createSignal(false);

  // Create shortcut engine
  const shortcuts = new ShortcutEngine();

  // Register shortcut handlers
  shortcuts.register("view.grid", () => {
    setActiveModule("library");
    setViewMode("grid");
  });
  shortcuts.register("view.loupe", () => {
    setActiveModule("library");
    setViewMode("loupe");
  });
  shortcuts.register("module.develop", () => setActiveModule("develop"));
  shortcuts.register("ui.toggle_panels", () => {
    setLeftOpen(o => !o);
    setRightOpen(o => !o);
  });
  shortcuts.register("ui.command_palette", () => setCommandPaletteOpen(true));
  shortcuts.register("ui.diagnostics", () => setDiagnosticsOpen(true));

  // View mode shortcuts
  const registerViewShortcuts = () => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

      // E = loupe view
      if (e.key === "e" || e.key === "E") {
        e.preventDefault();
        setViewMode("loupe");
      }
      // C = compare mode
      if (e.key === "c" || e.key === "C") {
        e.preventDefault();
        setViewMode("compare");
      }
      // N = survey mode
      if (e.key === "n" || e.key === "N") {
        e.preventDefault();
        setViewMode("survey");
      }
      // G = grid (also handled by shortcuts.register)
      if (e.key === "g" || e.key === "G") {
        e.preventDefault();
        setViewMode("grid");
        setActiveModule("library");
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    onCleanup(() => document.removeEventListener("keydown", handleKeyDown));
  };

  // Set context based on active module
  const updateContext = () => {
    shortcuts.setContext(activeModule() === "develop" ? "develop" : "library");
  };

  // Tab key: toggle side panels
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

    // Let shortcut engine handle it
    shortcuts.handleKeyDown(e);

    // Legacy filmstrip toggle
    if (e.key === "F" || (e.key === "f" && !e.metaKey && !e.ctrlKey)) {
      e.preventDefault();
      setFilmstripOpen(o => !o);
    }
  };

  onMount(() => {
    updateContext();
    registerViewShortcuts();
    document.addEventListener("keydown", handleKeyDown);
    onCleanup(() => document.removeEventListener("keydown", handleKeyDown));
  });

  return (
    <div class="flex flex-col h-full bg-[#141414]">
      {/* Top Bar */}
      <TopBar
        activeModule={activeModule()}
        onModuleChange={(mod) => {
          setActiveModule(mod);
          updateContext();
        }}
        version={props.version}
      />

      {/* Main area */}
      <div class="flex flex-1 overflow-hidden">
        {/* Left Sidebar */}
        <Show when={leftOpen()}>
          <LeftSidebar module={activeModule()} />
        </Show>

        {/* Center + Filmstrip */}
        <div class="flex flex-col flex-1 overflow-hidden">
          <MainView
            module={activeModule()}
            viewMode={viewMode()}
            selectedPhotoId={selectedPhotoId()}
            selectedPhotoIds={selectedPhotoIds()}
            onSelectPhoto={setSelectedPhotoId}
            onPhotosLoaded={setAllPhotoIds}
          />
          <Show when={filmstripOpen()}>
            <Filmstrip
              photoIds={allPhotoIds()}
              selectedId={selectedPhotoId()}
              onSelect={setSelectedPhotoId}
            />
          </Show>
        </div>

        {/* Right Sidebar */}
        <Show when={rightOpen()}>
          <RightSidebar module={activeModule()} selectedPhotoId={selectedPhotoId()} />
        </Show>
      </div>

      {/* Command Palette */}
      <CommandPalette
        open={commandPaletteOpen()}
        onClose={() => setCommandPaletteOpen(false)}
        onExecute={(action) => shortcuts.execute(action)}
      />

      {/* Diagnostics Overlay */}
      <Show when={diagnosticsOpen()}>
        <DiagnosticsView onClose={() => setDiagnosticsOpen(false)} />
      </Show>

      {/* Toast Notifications */}
      <ToastContainer />
    </div>
  );
}
