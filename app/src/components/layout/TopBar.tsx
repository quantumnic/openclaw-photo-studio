type Module = "library" | "develop" | "map" | "print";

interface TopBarProps {
  activeModule: Module;
  onModuleChange: (m: Module) => void;
  version?: any;
}

const MODULES: { id: Module; label: string; shortcut: string }[] = [
  { id: "library",  label: "Library",  shortcut: "G" },
  { id: "develop",  label: "Develop",  shortcut: "D" },
  { id: "map",      label: "Map",      shortcut: "M" },
  { id: "print",    label: "Print",    shortcut: "P" },
];

export function TopBar(props: TopBarProps) {
  return (
    <header class="flex items-center justify-between h-9 bg-[#1a1a1a] border-b border-[#2a2a2a] px-3 flex-shrink-0">
      {/* Logo */}
      <div class="flex items-center gap-2">
        <span class="text-[#4a9eff] font-bold text-sm tracking-tight">🌊 OCPS</span>
        {props.version && (
          <span class="text-[#444] text-xs">v{props.version.app}</span>
        )}
      </div>

      {/* Module Tabs */}
      <nav class="flex items-center gap-1">
        {MODULES.map(m => (
          <button
            onClick={() => props.onModuleChange(m.id)}
            class={`px-4 py-1 rounded text-xs font-medium transition-colors ${
              props.activeModule === m.id
                ? "bg-[#2f2f2f] text-[#d4d4d4]"
                : "text-[#666] hover:text-[#999] hover:bg-[#222]"
            }`}
            title={`${m.label} (${m.shortcut})`}
          >
            {m.label}
          </button>
        ))}
      </nav>

      {/* Right Controls */}
      <div class="flex items-center gap-2 text-xs text-[#555]">
        <span>Tab = Toggle Panels</span>
      </div>
    </header>
  );
}
