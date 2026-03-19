import { createSignal } from "solid-js";

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

export function LeftSidebar(props: LeftSidebarProps) {
  const folders = ["📁 2026", "📁 2025", "📁 2024", "📁 Archive"];
  const collections = ["⭐ Best of 2026", "📷 Weddings", "🏔 Landscapes", "Quick Collection"];

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
