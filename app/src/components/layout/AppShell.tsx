import { createSignal, Show } from "solid-js";
import { TopBar } from "./TopBar";
import { LeftSidebar } from "./LeftSidebar";
import { RightSidebar } from "./RightSidebar";
import { MainView } from "./MainView";
import { Filmstrip } from "./Filmstrip";

type Module = "library" | "develop" | "map" | "print";

interface AppShellProps {
  version?: any;
}

export function AppShell(props: AppShellProps) {
  const [activeModule, setActiveModule] = createSignal<Module>("library");
  const [leftOpen, setLeftOpen] = createSignal(true);
  const [rightOpen, setRightOpen] = createSignal(true);
  const [filmstripOpen, setFilmstripOpen] = createSignal(true);

  // Tab key: toggle side panels
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.target instanceof HTMLInputElement) return;
    if (e.key === "Tab") {
      e.preventDefault();
      setLeftOpen(o => !o);
      setRightOpen(o => !o);
    }
    if (e.key === "F" || (e.key === "f" && !e.metaKey && !e.ctrlKey)) {
      setFilmstripOpen(o => !o);
    }
  };

  document.addEventListener("keydown", handleKeyDown);

  return (
    <div class="flex flex-col h-full bg-[#141414]">
      {/* Top Bar */}
      <TopBar
        activeModule={activeModule()}
        onModuleChange={setActiveModule}
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
          <MainView module={activeModule()} />
          <Show when={filmstripOpen()}>
            <Filmstrip />
          </Show>
        </div>

        {/* Right Sidebar */}
        <Show when={rightOpen()}>
          <RightSidebar module={activeModule()} />
        </Show>
      </div>
    </div>
  );
}
