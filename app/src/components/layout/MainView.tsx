import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { createSignal, onMount, Show } from "solid-js";

type Module = "library" | "develop" | "map" | "print";

interface MainViewProps {
  module: Module;
}

export function MainView(props: MainViewProps) {
  return (
    <main class="flex-1 overflow-hidden bg-[#141414] relative">
      {props.module === "library" && <LibraryView />}
      {props.module === "develop" && <DevelopView />}
      {props.module === "map" && <PlaceholderView label="Map" emoji="🗺️" phase="Phase 5" />}
      {props.module === "print" && <PlaceholderView label="Print / Contact Sheet" emoji="🖨️" phase="Phase 7" />}
    </main>
  );
}

interface RawMetadata {
  success: boolean;
  camera_make?: string;
  camera_model?: string;
  width: number;
  height: number;
  megapixels: number;
  cfa_pattern: string;
  iso?: number;
  exposure_time?: string;
  aperture?: string;
  decode_time_ms: number;
}

function LibraryView() {
  const [greeting, setGreeting] = createSignal("");
  const [rawMetadata, setRawMetadata] = createSignal<RawMetadata | null>(null);
  const [rawError, setRawError] = createSignal<string | null>(null);
  const [isDecoding, setIsDecoding] = createSignal(false);

  onMount(async () => {
    try {
      const msg = await invoke<string>("greet", { name: "Nicola" });
      setGreeting(msg);
    } catch {
      setGreeting("OpenClaw Photo Studio — running in browser mode");
    }
  });

  const handleOpenRAW = async () => {
    setRawError(null);
    setRawMetadata(null);

    try {
      const selected = await open({
        title: "Open RAW File (Test)",
        multiple: false,
        filters: [
          {
            name: "RAW Files",
            extensions: ["dng", "arw", "nef", "raf", "orf", "rw2", "cr2", "cr3"],
          },
        ],
      });

      if (!selected) {
        return; // User cancelled
      }

      const path = typeof selected === "string" ? selected : selected.path;

      setIsDecoding(true);

      try {
        const result = await invoke<RawMetadata>("decode_raw_info", { path });
        setRawMetadata(result);
      } catch (err) {
        setRawError(String(err));
      } finally {
        setIsDecoding(false);
      }
    } catch (err) {
      setRawError("Failed to open file dialog: " + String(err));
    }
  };

  // Placeholder photos for the grid
  const placeholderPhotos = Array.from({ length: 24 }, (_, i) => ({
    id: i,
    rating: Math.floor(Math.random() * 6),
    flag: ["none", "pick", "reject"][Math.floor(Math.random() * 3)],
  }));

  return (
    <div class="h-full flex flex-col">
      {/* Toolbar */}
      <div class="h-8 bg-[#1c1c1c] border-b border-[#2a2a2a] flex items-center px-3 gap-3 text-xs text-[#666] flex-shrink-0">
        <span>Grid</span>
        <span class="text-[#333]">|</span>
        <span>Loupe</span>
        <span class="text-[#333]">|</span>
        <span>Compare</span>
        <span class="ml-auto">24 photos</span>
        <span class="text-[#333]">|</span>
        <span>Sort: Date ▾</span>
      </div>

      {/* Backend greeting */}
      {greeting() && (
        <div class="px-4 py-2 bg-[#1c2a1c] border-b border-[#2a3a2a] text-xs text-[#4a9eff]">
          ✓ {greeting()}
        </div>
      )}

      {/* RAW Decode Test Section */}
      <div class="px-4 py-3 bg-[#1a1a1a] border-b border-[#2a2a2a]">
        <div class="flex items-center gap-3">
          <button
            onClick={handleOpenRAW}
            disabled={isDecoding()}
            class="px-4 py-1.5 bg-[#4a9eff] hover:bg-[#5aa9ff] disabled:bg-[#2a4a7f] disabled:cursor-not-allowed text-white text-xs font-medium rounded transition-colors"
          >
            {isDecoding() ? "Decoding..." : "📷 Open RAW File (Test)"}
          </button>

          <Show when={rawMetadata()}>
            {(metadata) => (
              <div class="text-xs text-[#aaa]">
                <span class="font-medium text-[#4a9eff]">
                  {metadata().camera_make || "Unknown"} {metadata().camera_model || ""}
                </span>
                <span class="text-[#666] mx-2">•</span>
                <span>{metadata().width} × {metadata().height} px</span>
                <span class="text-[#666] mx-2">•</span>
                <span>{metadata().megapixels.toFixed(1)} MP</span>
                <span class="text-[#666] mx-2">•</span>
                <span class="text-[#666]">{metadata().decode_time_ms.toFixed(0)}ms</span>
              </div>
            )}
          </Show>

          <Show when={rawError()}>
            {(error) => (
              <div class="text-xs text-[#ff6b6b]">
                ✗ {error()}
              </div>
            )}
          </Show>
        </div>

        {/* Detailed metadata display */}
        <Show when={rawMetadata()}>
          {(metadata) => (
            <div class="mt-3 p-3 bg-[#0f0f0f] rounded border border-[#2a2a2a] text-xs">
              <div class="grid grid-cols-2 gap-x-6 gap-y-1.5 text-[#aaa]">
                <div>
                  <span class="text-[#666]">CFA Pattern:</span>{" "}
                  <span class="font-mono">{metadata().cfa_pattern}</span>
                </div>
                <div>
                  <Show when={metadata().iso}>
                    <span class="text-[#666]">ISO:</span> {metadata().iso}
                  </Show>
                </div>
                <div>
                  <Show when={metadata().exposure_time}>
                    <span class="text-[#666]">Exposure:</span> {metadata().exposure_time}
                  </Show>
                </div>
                <div>
                  <Show when={metadata().aperture}>
                    <span class="text-[#666]">Aperture:</span> {metadata().aperture}
                  </Show>
                </div>
              </div>
              <div class="mt-2 pt-2 border-t border-[#2a2a2a] text-[#4a9eff]">
                ✓ RAW decode pipeline working — Phase 1, Day 15-35 complete
              </div>
            </div>
          )}
        </Show>
      </div>

      {/* Photo Grid — placeholder */}
      <div class="flex-1 overflow-auto p-3">
        <div class="grid gap-1" style="grid-template-columns: repeat(auto-fill, minmax(160px, 1fr))">
          {placeholderPhotos.map(photo => (
            <div class="relative bg-[#1a1a1a] rounded overflow-hidden cursor-pointer group hover:ring-1 hover:ring-[#4a9eff] transition-all aspect-[3/2]">
              {/* Placeholder image */}
              <div class="absolute inset-0 bg-gradient-to-br from-[#222] to-[#111] flex items-center justify-center">
                <span class="text-[#333] text-2xl">📷</span>
              </div>
              {/* Rating */}
              <div class="absolute bottom-0 left-0 right-0 px-1 py-0.5 bg-black/60 flex items-center gap-1">
                <div class="flex gap-0.5">
                  {Array.from({ length: 5 }, (_, i) => (
                    <span class={`text-[8px] ${i < photo.rating ? "text-[#e8b84b]" : "text-[#333]"}`}>★</span>
                  ))}
                </div>
                {photo.flag === "pick" && <span class="text-[8px] text-[#4a9eff] ml-auto">P</span>}
                {photo.flag === "reject" && <span class="text-[8px] text-[#ff4a4a] ml-auto">X</span>}
              </div>
            </div>
          ))}
        </div>

        {/* Welcome message */}
        <div class="mt-8 text-center text-[#444] text-sm">
          <div class="text-4xl mb-3">🌊</div>
          <div class="text-[#666] font-medium mb-1">OpenClaw Photo Studio</div>
          <div class="text-[#444] text-xs mb-4">Skeleton v0.1.0 — Phase 1 of 9</div>
          <div class="text-[#333] text-xs">
            Import • Develop • Export — Coming Phase 2–3
          </div>
          <div class="mt-4 text-[#333] text-xs space-y-1">
            <div>Tab = Toggle Panels</div>
            <div>D = Develop Module</div>
            <div>G = Library Module</div>
          </div>
        </div>
      </div>
    </div>
  );
}

function DevelopView() {
  return (
    <div class="h-full flex flex-col items-center justify-center text-[#444]">
      <div class="text-6xl mb-4">🎛️</div>
      <div class="text-[#666] font-medium mb-1">Develop Module</div>
      <div class="text-xs text-[#444] mb-2">GPU-accelerated RAW development</div>
      <div class="text-xs text-[#333]">
        Full pipeline coming in Phase 3 — Sliders are live on the right →
      </div>
    </div>
  );
}

function PlaceholderView(props: { label: string; emoji: string; phase: string }) {
  return (
    <div class="h-full flex flex-col items-center justify-center text-[#444]">
      <div class="text-6xl mb-4">{props.emoji}</div>
      <div class="text-[#666] font-medium mb-1">{props.label}</div>
      <div class="text-xs text-[#333]">Coming in {props.phase}</div>
    </div>
  );
}
