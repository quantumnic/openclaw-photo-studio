import { For, Show } from "solid-js";

interface WelcomeScreenProps {
  onImportFolder: () => void;
  onOpenCatalog: () => void;
  recentCatalogs: string[];
  onOpenRecent?: (path: string) => void;
}

export function WelcomeScreen(props: WelcomeScreenProps) {
  return (
    <div class="h-full flex flex-col items-center justify-center bg-[#0f0f0f]">
      {/* Logo and Title */}
      <div class="mb-8 text-center">
        <div class="text-8xl mb-2">🌊</div>
        <div class="text-2xl font-light text-[#ccc]">OpenClaw Photo Studio</div>
        <div class="text-sm text-[#555] mt-1">v0.8.0 — source-available</div>
      </div>

      {/* Quick Actions */}
      <div class="flex gap-4 mb-8">
        <button
          onClick={props.onImportFolder}
          class="px-6 py-3 bg-[#4a9eff] text-white rounded-lg hover:bg-[#5aadff] font-medium transition-colors"
        >
          📁 Import Folder
        </button>
        <button
          onClick={props.onOpenCatalog}
          class="px-6 py-3 bg-[#2a2a2a] text-[#ccc] rounded-lg hover:bg-[#333] font-medium transition-colors"
        >
          🗂️ Open Catalog
        </button>
      </div>

      {/* Recent Catalogs */}
      <Show when={props.recentCatalogs.length > 0}>
        <div class="w-80">
          <div class="text-xs text-[#555] uppercase tracking-wider mb-2">Recent Catalogs</div>
          <div class="space-y-1">
            <For each={props.recentCatalogs.slice(0, 5)}>
              {(catalogPath) => {
                const catalogName = catalogPath.split("/").pop() || catalogPath;
                return (
                  <button
                    onClick={() => props.onOpenRecent?.(catalogPath)}
                    class="w-full text-left px-3 py-2 rounded hover:bg-[#1a1a1a] transition-colors group"
                  >
                    <div class="text-sm text-[#888] group-hover:text-[#aaa] truncate">
                      {catalogName}
                    </div>
                    <div class="text-xs text-[#444] group-hover:text-[#555] truncate mt-0.5">
                      {catalogPath}
                    </div>
                  </button>
                );
              }}
            </For>
          </div>
        </div>
      </Show>

      {/* Footer Links */}
      <div class="absolute bottom-6 flex gap-6 text-xs text-[#444]">
        <a
          href="https://github.com/quantumnic/openclaw-photo-studio"
          target="_blank"
          rel="noopener noreferrer"
          class="hover:text-[#666] transition-colors"
        >
          GitHub
        </a>
        <span>PolyForm Noncommercial</span>
        <span>© 2026 OpenClaw Photo Studio Contributors</span>
      </div>

      {/* Version info and shortcuts hint */}
      <div class="absolute top-6 right-6 text-xs text-[#444]">
        <div class="text-right">
          <div>Press <kbd class="px-2 py-1 bg-[#1a1a1a] rounded border border-[#2a2a2a]">Cmd+K</kbd> for commands</div>
          <div class="mt-1">Press <kbd class="px-2 py-1 bg-[#1a1a1a] rounded border border-[#2a2a2a]">Cmd+,</kbd> for preferences</div>
        </div>
      </div>
    </div>
  );
}
