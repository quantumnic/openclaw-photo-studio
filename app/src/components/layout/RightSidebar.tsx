import { createSignal } from "solid-js";

type Module = "library" | "develop" | "map" | "print";

interface RightSidebarProps {
  module: Module;
}

function Slider(props: { label: string; min?: number; max?: number; default?: number; unit?: string }) {
  const [value, setValue] = createSignal(props.default ?? 0);
  const min = props.min ?? -100;
  const max = props.max ?? 100;

  return (
    <div class="flex items-center gap-2 py-1">
      <span class="w-20 text-xs text-[#777] shrink-0">{props.label}</span>
      <input
        type="range"
        min={min}
        max={max}
        value={value()}
        onInput={e => setValue(Number(e.currentTarget.value))}
        class="flex-1"
      />
      <span
        class={`w-8 text-right text-xs font-mono ${value() !== 0 ? "text-[#4a9eff]" : "text-[#555]"}`}
        onDblClick={() => setValue(props.default ?? 0)}
        title="Double-click to reset"
        style="cursor: pointer"
      >
        {value()}
      </span>
    </div>
  );
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
      {open() && <div class="px-3 pb-3">{props.children}</div>}
    </div>
  );
}

export function RightSidebar(props: RightSidebarProps) {
  return (
    <aside class="w-64 flex-shrink-0 bg-[#1a1a1a] border-l border-[#2a2a2a] overflow-y-auto">
      {props.module === "develop" && (
        <>
          <PanelSection title="Basic">
            <div class="mb-2">
              <div class="text-xs text-[#666] mb-1">White Balance</div>
              <Slider label="Temp" min={2000} max={50000} default={5500} />
              <Slider label="Tint" min={-150} max={150} default={0} />
            </div>
            <Slider label="Exposure" min={-5} max={5} default={0} />
            <Slider label="Contrast" />
            <Slider label="Highlights" />
            <Slider label="Shadows" />
            <Slider label="Whites" />
            <Slider label="Blacks" />
            <div class="border-t border-[#2a2a2a] mt-2 pt-2">
              <Slider label="Clarity" />
              <Slider label="Dehaze" />
              <Slider label="Vibrance" />
              <Slider label="Saturation" />
            </div>
          </PanelSection>

          <PanelSection title="Tone Curve">
            <div class="bg-[#111] rounded aspect-square flex items-center justify-center text-[#333] text-xs">
              Curve Editor — Phase 3
            </div>
          </PanelSection>

          <PanelSection title="HSL / Color">
            <div class="text-xs text-[#555] italic">HSL Mixer — Phase 3</div>
          </PanelSection>

          <PanelSection title="Detail">
            <div class="text-xs text-[#777] mb-1 font-medium">Sharpening</div>
            <Slider label="Amount" min={0} max={150} default={40} />
            <Slider label="Radius" min={5} max={30} default={10} />
            <Slider label="Detail" min={0} max={100} default={25} />
            <Slider label="Masking" min={0} max={100} default={0} />
            <div class="text-xs text-[#777] mt-2 mb-1 font-medium">Noise Reduction</div>
            <Slider label="Luminance" min={0} max={100} default={0} />
            <Slider label="Color" min={0} max={100} default={25} />
          </PanelSection>
        </>
      )}

      {props.module === "library" && (
        <>
          <PanelSection title="Histogram">
            <div class="bg-[#111] rounded h-20 flex items-center justify-center text-[#333] text-xs">
              Histogram — Phase 2
            </div>
          </PanelSection>
          <PanelSection title="Quick Develop">
            <div class="text-xs text-[#555] italic">Quick adjustments — Phase 2</div>
          </PanelSection>
          <PanelSection title="Keywords">
            <div class="text-xs text-[#555] italic">Keyword tagging — Phase 2</div>
          </PanelSection>
          <PanelSection title="Metadata">
            <div class="space-y-1">
              {[["Camera", "—"], ["Lens", "—"], ["ISO", "—"], ["f/", "—"], ["Shutter", "—"]].map(([k, v]) => (
                <div class="flex justify-between text-xs">
                  <span class="text-[#666]">{k}</span>
                  <span class="text-[#888]">{v}</span>
                </div>
              ))}
            </div>
          </PanelSection>
        </>
      )}
    </aside>
  );
}
