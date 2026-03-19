import { createSignal, createResource, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

type Module = "library" | "develop" | "map" | "print";

interface LeftSidebarProps {
  module: Module;
}

function PanelSection(props: { title: string; children: any }) {
  const [open, setOpen] = createSignal(true);
  return (
    <div class="border-b border-[#2a2a2a]">
      <button
        onClick={() => setOpen(o => !o)}
        class="w-full flex items-center justify-between px-3 py-2 text-xs font-semibold text-[#888] hover:text-[#aaa] uppercase tracking-wider"
      >
        <span>{props.title}</span>
        <span class="text-[#555]">{open() ? "▾" : "▸"}</span>
      </button>
      {open() && <div class="px-2 pb-2">{props.children}</div>}
    </div>
  );
}

interface Keyword {
  id: string;
  name: string;
  count: number;
}

export function LeftSidebar(props: LeftSidebarProps) {
  const folders = ["📁 2026", "📁 2025", "📁 2024", "📁 Archive"];
  const collections = ["⭐ Best of 2026", "📷 Weddings", "🏔 Landscapes", "Quick Collection"];

  // Load keywords from catalog
  const [keywords] = createResource<Keyword[]>(async () => {
    try {
      return await invoke<Keyword[]>("get_keywords");
    } catch (e) {
      console.error("Failed to load keywords:", e);
      return [];
    }
  });

  const [newKeyword, setNewKeyword] = createSignal("");

  const handleAddKeyword = async () => {
    const keyword = newKeyword().trim();
    if (!keyword) return;

    // This would add to selected photos - for now just log
    console.log("Add keyword:", keyword);
    setNewKeyword("");

    // Refresh keywords
    keywords.refetch?.();
  };

  return (
    <aside class="w-56 flex-shrink-0 bg-[#1a1a1a] border-r border-[#2a2a2a] overflow-y-auto flex flex-col">
      {props.module === "library" && (
        <>
          <PanelSection title="Navigator">
            <div class="bg-[#111] rounded aspect-video flex items-center justify-center text-[#333] text-xs">
              No Preview
            </div>
          </PanelSection>

          <PanelSection title="Folders">
            <ul class="space-y-0.5">
              {folders.map(f => (
                <li>
                  <button class="w-full text-left px-2 py-1 rounded text-xs text-[#888] hover:bg-[#252525] hover:text-[#bbb] transition-colors">
                    {f}
                  </button>
                </li>
              ))}
            </ul>
          </PanelSection>

          <PanelSection title="Collections">
            <ul class="space-y-0.5">
              {collections.map(c => (
                <li>
                  <button class="w-full text-left px-2 py-1 rounded text-xs text-[#888] hover:bg-[#252525] hover:text-[#bbb] transition-colors">
                    {c}
                  </button>
                </li>
              ))}
            </ul>
          </PanelSection>

          <PanelSection title="Keywords">
            <div class="space-y-2">
              {/* Add keyword input */}
              <div class="flex gap-1">
                <input
                  type="text"
                  value={newKeyword()}
                  onInput={(e) => setNewKeyword(e.currentTarget.value)}
                  onKeyPress={(e) => e.key === "Enter" && handleAddKeyword()}
                  placeholder="Add keyword..."
                  class="flex-1 px-2 py-1 text-xs bg-[#252525] border border-[#333] rounded text-[#bbb] placeholder-[#555] focus:border-blue-500 focus:outline-none"
                />
                <button
                  onClick={handleAddKeyword}
                  class="px-2 py-1 bg-blue-600 hover:bg-blue-700 text-white text-xs rounded transition-colors"
                  disabled={!newKeyword().trim()}
                >
                  +
                </button>
              </div>

              {/* Keywords list */}
              <ul class="space-y-0.5 max-h-48 overflow-y-auto">
                <Show when={keywords()} fallback={<li class="px-2 py-1 text-xs text-[#555]">Loading...</li>}>
                  <For each={keywords()}>
                    {(keyword) => (
                      <li>
                        <button class="w-full text-left px-2 py-1 rounded text-xs text-[#888] hover:bg-[#252525] hover:text-[#bbb] transition-colors flex justify-between items-center">
                          <span>{keyword.name}</span>
                          <span class="text-[#555] text-xs">({keyword.count})</span>
                        </button>
                      </li>
                    )}
                  </For>
                </Show>
              </ul>
            </div>
          </PanelSection>
        </>
      )}

      {props.module === "develop" && (
        <PanelSection title="Presets">
          {["Warm Tone", "Cool Mist", "B&W Film", "High Contrast", "Soft Portrait"].map(p => (
            <button class="w-full text-left px-2 py-1 rounded text-xs text-[#888] hover:bg-[#252525] hover:text-[#bbb] transition-colors">
              {p}
            </button>
          ))}
        </PanelSection>
      )}
    </aside>
  );
}
