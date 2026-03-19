import { createSignal, Show, onMount, onCleanup } from "solid-js";
import { TopBar } from "./TopBar";
import { LeftSidebar } from "./LeftSidebar";
import { RightSidebar } from "./RightSidebar";
import { MainView } from "./MainView";
import { Filmstrip } from "./Filmstrip";
import { CommandPalette } from "../common/CommandPalette";
import { ShortcutEngine } from "../../lib/shortcuts";

type Module = "library" | "develop" | "map" | "print";

interface AppShellProps {
  version?: any;
}

export function AppShell(props: AppShellProps) {
  const [activeModule, setActiveModule] = createSignal<Module>("library");
  const [leftOpen, setLeftOpen] = createSignal(true);
  const [rightOpen, setRightOpen] = createSignal(true);
  const [filmstripOpen, setFilmstripOpen] = createSignal(true);
  const [selectedPhotoId, setSelectedPhotoId] = createSignal<string | null>(null);
  const [commandPaletteOpen, setCommandPaletteOpen] = createSignal(false);

  // Create shortcut engine
  const shortcuts = new ShortcutEngine();

  // Register shortcut handlers
  shortcuts.register("view.grid", () => setActiveModule("library"));
  shortcuts.register("view.loupe", () => setActiveModule("library"));
  shortcuts.register("module.develop", () => setActiveModule("develop"));
  shortcuts.register("ui.toggle_panels", () => {
    setLeftOpen(o => !o);
    setRightOpen(o => !o);
  });
  shortcuts.register("ui.command_palette", () => setCommandPaletteOpen(true));

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
            selectedPhotoId={selectedPhotoId()}
            onSelectPhoto={setSelectedPhotoId}
          />
          <Show when={filmstripOpen()}>
            <Filmstrip />
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
    </div>
  );
}
