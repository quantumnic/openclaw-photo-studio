import { createSignal, createEffect, onMount, onCleanup, Show } from "solid-js";
import { createStore } from "solid-js/store";
import { invoke } from "@tauri-apps/api/core";

type Module = "library" | "develop" | "map" | "print";

interface RightSidebarProps {
  module: Module;
  selectedPhotoId?: string;
}

interface WhiteBalance {
  temperature: number;
  tint: number;
}

interface SharpeningSettings {
  amount: number;
  radius: number;
  detail: number;
  masking: number;
}

interface NoiseReductionSettings {
  luminance: number;
  luminance_detail: number;
  color: number;
  color_detail: number;
}

interface CropSettings {
  left: number;
  top: number;
  right: number;
  bottom: number;
  angle: number;
}

interface ColorGradingSettings {
  shadows_hue: number;
  shadows_sat: number;
  midtones_hue: number;
  midtones_sat: number;
  highlights_hue: number;
  highlights_sat: number;
  global_hue: number;
  global_sat: number;
  blending: number;
}

interface EditRecipe {
  white_balance: WhiteBalance;
  exposure: number;
  contrast: number;
  highlights: number;
  shadows: number;
  whites: number;
  blacks: number;
  clarity: number;
  dehaze: number;
  vibrance: number;
  saturation: number;
  sharpening: SharpeningSettings;
  noise_reduction: NoiseReductionSettings;
  crop: CropSettings;
  color_grading: ColorGradingSettings;
}

function defaultRecipe(): EditRecipe {
  return {
    white_balance: { temperature: 5500, tint: 0 },
    exposure: 0,
    contrast: 0,
    highlights: 0,
    shadows: 0,
    whites: 0,
    blacks: 0,
    clarity: 0,
    dehaze: 0,
    vibrance: 0,
    saturation: 0,
    sharpening: { amount: 0, radius: 1.0, detail: 25, masking: 0 },
    noise_reduction: { luminance: 0, luminance_detail: 50, color: 0, color_detail: 50 },
    crop: { left: 0, top: 0, right: 1, bottom: 1, angle: 0 },
    color_grading: {
      shadows_hue: 0,
      shadows_sat: 0,
      midtones_hue: 0,
      midtones_sat: 0,
      highlights_hue: 0,
      highlights_sat: 0,
      global_hue: 0,
      global_sat: 0,
      blending: 50,
    },
  };
}

function Slider(props: {
  label: string;
  value: number;
  onChange: (v: number) => void;
  min?: number;
  max?: number;
  step?: number;
  unit?: string;
}) {
  const min = props.min ?? -100;
  const max = props.max ?? 100;
  const step = props.step ?? 1;
  const defaultVal = min === -100 && max === 100 ? 0 : min;

  return (
    <div class="flex items-center gap-2 py-1">
      <span class="w-20 text-xs text-[#777] shrink-0">{props.label}</span>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={props.value}
        onInput={e => props.onChange(Number(e.currentTarget.value))}
        class="flex-1"
      />
      <span
        class={`w-12 text-right text-xs font-mono ${props.value !== defaultVal ? "text-[#4a9eff]" : "text-[#555]"}`}
        onDblClick={() => props.onChange(defaultVal)}
        title="Double-click to reset"
        style="cursor: pointer; user-select: none;"
      >
        {props.unit ? `${props.value}${props.unit}` : props.value}
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
  const [recipe, setRecipe] = createStore<EditRecipe>(defaultRecipe());
  const [dirty, setDirty] = createSignal(false);
  const [hasClipboard, setHasClipboard] = createSignal(false);
  const [saving, setSaving] = createSignal(false);

  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  // Load recipe when selected photo changes
  createEffect(async () => {
    const id = props.selectedPhotoId;
    if (!id || props.module !== "develop") {
      setRecipe(defaultRecipe());
      return;
    }

    try {
      const saved = await invoke<EditRecipe>("load_edit_recipe", { photoId: id });
      setRecipe(saved ?? defaultRecipe());
      setDirty(false);
    } catch (e) {
      console.error("Failed to load recipe:", e);
      setRecipe(defaultRecipe());
    }
  });

  // Debounced auto-save
  function onValueChange(updater: () => void) {
    updater();
    setDirty(true);
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(async () => {
      if (!props.selectedPhotoId || props.module !== "develop") return;
      setSaving(true);
      try {
        await invoke("save_edit_recipe", { photoId: props.selectedPhotoId, recipe });
        setDirty(false);
      } catch (e) {
        console.error("Save failed:", e);
      }
      setSaving(false);
    }, 500);
  }

  // Keyboard shortcuts
  function handleKey(e: KeyboardEvent) {
    if (!props.selectedPhotoId || props.module !== "develop") return;
    const mod = e.metaKey || e.ctrlKey;

    if (mod && e.key === "c" && !e.shiftKey) {
      e.preventDefault();
      copyEdit();
    }
    if (mod && e.key === "v" && !e.shiftKey) {
      e.preventDefault();
      pasteEdit();
    }
    // Cmd+Z for undo can be added here later
  }

  async function copyEdit() {
    if (!props.selectedPhotoId) return;
    try {
      await invoke("copy_edit", { photoId: props.selectedPhotoId, modules: [] });
      setHasClipboard(true);
    } catch (e) {
      console.error("Copy failed:", e);
    }
  }

  async function pasteEdit() {
    if (!props.selectedPhotoId) return;
    try {
      await invoke("paste_edit", { photoIds: [props.selectedPhotoId], modules: [] });
      // Reload recipe
      const saved = await invoke<EditRecipe>("load_edit_recipe", { photoId: props.selectedPhotoId });
      setRecipe(saved ?? defaultRecipe());
      setDirty(false);
    } catch (e) {
      console.error("Paste failed:", e);
    }
  }

  async function resetEdit() {
    if (!props.selectedPhotoId) return;
    if (!confirm("Reset all edits for this photo?")) return;
    try {
      await invoke("reset_edit", { photoId: props.selectedPhotoId });
      setRecipe(defaultRecipe());
      setDirty(false);
    } catch (e) {
      console.error("Reset failed:", e);
    }
  }

  onMount(() => {
    document.addEventListener("keydown", handleKey);
  });

  onCleanup(() => {
    document.removeEventListener("keydown", handleKey);
    if (saveTimer) clearTimeout(saveTimer);
  });

  return (
    <aside class="w-64 flex-shrink-0 bg-[#1a1a1a] border-l border-[#2a2a2a] overflow-y-auto flex flex-col">
      {props.module === "develop" && (
        <>
          {/* Toolbar */}
          <div class="flex items-center gap-1 px-2 py-1 border-b border-[#2a2a2a] shrink-0">
            <button
              onClick={copyEdit}
              title="Copy Edit (Cmd+C)"
              class="px-2 py-1 text-xs bg-[#2a2a2a] hover:bg-[#333] rounded"
            >
              Copy
            </button>
            <button
              onClick={pasteEdit}
              disabled={!hasClipboard()}
              title="Paste Edit (Cmd+V)"
              class="px-2 py-1 text-xs bg-[#2a2a2a] hover:bg-[#333] rounded disabled:opacity-30 disabled:cursor-not-allowed"
            >
              Paste
            </button>
            <button
              onClick={resetEdit}
              title="Reset All"
              class="px-2 py-1 text-xs bg-[#2a2a2a] hover:bg-[#333] rounded"
            >
              Reset
            </button>
            <div class="flex-1" />
            <Show when={dirty()}>
              <span class="text-xs text-yellow-500">●</span>
            </Show>
            <Show when={saving()}>
              <span class="text-xs text-[#555]">saving...</span>
            </Show>
          </div>

          {/* Panels */}
          <div class="flex-1 overflow-y-auto">
            <PanelSection title="Basic">
              <div class="mb-2">
                <div class="text-xs text-[#666] mb-1">White Balance</div>
                <Slider
                  label="Temp"
                  value={recipe.white_balance.temperature}
                  onChange={v => onValueChange(() => setRecipe("white_balance", "temperature", v))}
                  min={2000}
                  max={50000}
                  step={50}
                  unit="K"
                />
                <Slider
                  label="Tint"
                  value={recipe.white_balance.tint}
                  onChange={v => onValueChange(() => setRecipe("white_balance", "tint", v))}
                  min={-150}
                  max={150}
                />
              </div>
              <Slider
                label="Exposure"
                value={recipe.exposure}
                onChange={v => onValueChange(() => setRecipe("exposure", v))}
                min={-5}
                max={5}
                step={0.1}
              />
              <Slider
                label="Contrast"
                value={recipe.contrast}
                onChange={v => onValueChange(() => setRecipe("contrast", v))}
              />
              <Slider
                label="Highlights"
                value={recipe.highlights}
                onChange={v => onValueChange(() => setRecipe("highlights", v))}
              />
              <Slider
                label="Shadows"
                value={recipe.shadows}
                onChange={v => onValueChange(() => setRecipe("shadows", v))}
              />
              <Slider
                label="Whites"
                value={recipe.whites}
                onChange={v => onValueChange(() => setRecipe("whites", v))}
              />
              <Slider
                label="Blacks"
                value={recipe.blacks}
                onChange={v => onValueChange(() => setRecipe("blacks", v))}
              />
              <div class="border-t border-[#2a2a2a] mt-2 pt-2">
                <Slider
                  label="Clarity"
                  value={recipe.clarity}
                  onChange={v => onValueChange(() => setRecipe("clarity", v))}
                />
                <Slider
                  label="Dehaze"
                  value={recipe.dehaze}
                  onChange={v => onValueChange(() => setRecipe("dehaze", v))}
                />
                <Slider
                  label="Vibrance"
                  value={recipe.vibrance}
                  onChange={v => onValueChange(() => setRecipe("vibrance", v))}
                />
                <Slider
                  label="Saturation"
                  value={recipe.saturation}
                  onChange={v => onValueChange(() => setRecipe("saturation", v))}
                />
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
              <Slider
                label="Amount"
                value={recipe.sharpening.amount}
                onChange={v => onValueChange(() => setRecipe("sharpening", "amount", v))}
                min={0}
                max={150}
              />
              <Slider
                label="Radius"
                value={recipe.sharpening.radius}
                onChange={v => onValueChange(() => setRecipe("sharpening", "radius", v))}
                min={0.5}
                max={3}
                step={0.1}
              />
              <Slider
                label="Detail"
                value={recipe.sharpening.detail}
                onChange={v => onValueChange(() => setRecipe("sharpening", "detail", v))}
                min={0}
                max={100}
              />
              <Slider
                label="Masking"
                value={recipe.sharpening.masking}
                onChange={v => onValueChange(() => setRecipe("sharpening", "masking", v))}
                min={0}
                max={100}
              />
              <div class="text-xs text-[#777] mt-2 mb-1 font-medium">Noise Reduction</div>
              <Slider
                label="Luminance"
                value={recipe.noise_reduction.luminance}
                onChange={v => onValueChange(() => setRecipe("noise_reduction", "luminance", v))}
                min={0}
                max={100}
              />
              <Slider
                label="Color"
                value={recipe.noise_reduction.color}
                onChange={v => onValueChange(() => setRecipe("noise_reduction", "color", v))}
                min={0}
                max={100}
              />
            </PanelSection>
          </div>
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
              {[
                ["Camera", "—"],
                ["Lens", "—"],
                ["ISO", "—"],
                ["f/", "—"],
                ["Shutter", "—"],
              ].map(([k, v]) => (
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
