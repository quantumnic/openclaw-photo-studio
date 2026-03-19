import { createSignal, For } from "solid-js";

interface CompareViewProps {
  photoIds: [string, string]; // exactly 2 photos
  onSwap: () => void;
}

interface PhotoMetadata {
  id: string;
  file_name: string;
  rating: number;
  flag: string;
  camera_model?: string;
  date_taken?: string;
}

export function CompareView(props: CompareViewProps) {
  const [syncZoom, setSyncZoom] = createSignal(true);
  const [zoomLevel, setZoomLevel] = createSignal(1.0);

  // Mock metadata (in reality, would fetch from catalog)
  const getMetadata = (photoId: string): PhotoMetadata => {
    return {
      id: photoId,
      file_name: `Photo_${photoId.slice(0, 8)}.ARW`,
      rating: 3,
      flag: "none",
      camera_model: "Sony A7IV",
      date_taken: "2026-03-19",
    };
  };

  return (
    <div class="h-full flex flex-col bg-[#0a0a0a]">
      {/* Toolbar */}
      <div class="h-10 bg-[#1c1c1c] border-b border-[#2a2a2a] flex items-center px-3 gap-3 text-xs text-[#aaa] flex-shrink-0">
        <span class="text-[#666] font-medium">Compare Mode</span>
        <span class="text-[#333]">|</span>

        <button
          onClick={props.onSwap}
          class="px-3 py-1 bg-[#2a2a2a] hover:bg-[#3a3a3a] text-[#aaa] rounded text-xs transition-colors"
        >
          ⇄ Swap
        </button>

        <button
          onClick={() => setSyncZoom(!syncZoom())}
          class={`px-3 py-1 rounded text-xs transition-colors ${
            syncZoom()
              ? "bg-[#4a9eff] text-white"
              : "bg-[#2a2a2a] hover:bg-[#3a3a3a] text-[#aaa]"
          }`}
        >
          🔗 Sync Zoom
        </button>

        <div class="ml-auto flex items-center gap-2">
          <span class="text-[#444]">Zoom:</span>
          <span class="text-[#666] font-mono">{(zoomLevel() * 100).toFixed(0)}%</span>
        </div>

        <span class="text-[#333] text-[10px]">Press C to exit compare mode</span>
      </div>

      {/* Two-panel layout */}
      <div class="flex-1 flex overflow-hidden">
        {/* Left Panel */}
        <ComparePanel
          photoId={props.photoIds[0]}
          metadata={getMetadata(props.photoIds[0])}
          position="left"
        />

        {/* Divider */}
        <div class="w-px bg-[#2a2a2a] flex-shrink-0" />

        {/* Right Panel */}
        <ComparePanel
          photoId={props.photoIds[1]}
          metadata={getMetadata(props.photoIds[1])}
          position="right"
        />
      </div>

      {/* Instructions */}
      <div class="h-8 bg-[#1c1c1c] border-t border-[#2a2a2a] flex items-center justify-center gap-4 text-[10px] text-[#444] flex-shrink-0">
        <span>← → Change right photo</span>
        <span>•</span>
        <span>⇄ Swap panels</span>
        <span>•</span>
        <span>0-5 Rate selected</span>
      </div>
    </div>
  );
}

interface ComparePanelProps {
  photoId: string;
  metadata: PhotoMetadata;
  position: "left" | "right";
}

function ComparePanel(props: ComparePanelProps) {
  const flagColors: Record<string, string> = {
    pick: "#4a9eff",
    reject: "#ff4a4a",
    none: "#444",
  };

  return (
    <div class="flex-1 flex flex-col relative">
      {/* Photo placeholder */}
      <div class="flex-1 flex items-center justify-center bg-[#111]">
        <div class="text-6xl">📷</div>
      </div>

      {/* Metadata overlay at bottom */}
      <div class="absolute bottom-0 left-0 right-0 bg-black/80 backdrop-blur-sm p-3 space-y-1">
        {/* Filename */}
        <div class="text-xs text-[#eee] font-medium truncate">
          {props.metadata.file_name}
        </div>

        {/* Rating stars */}
        <div class="flex items-center gap-2">
          <div class="flex gap-0.5">
            <For each={Array.from({ length: 5 })}>
              {(_, i) => (
                <span
                  class={`text-sm ${
                    i() < props.metadata.rating ? "text-[#e8b84b]" : "text-[#333]"
                  }`}
                >
                  ★
                </span>
              )}
            </For>
          </div>

          {/* Flag indicator */}
          {props.metadata.flag !== "none" && (
            <span
              class="text-xs font-bold px-1.5 py-0.5 rounded"
              style={`color: ${flagColors[props.metadata.flag]}`}
            >
              {props.metadata.flag === "pick" ? "P" : "X"}
            </span>
          )}
        </div>

        {/* Camera & Date */}
        <div class="text-[10px] text-[#666] flex gap-2">
          {props.metadata.camera_model && <span>{props.metadata.camera_model}</span>}
          {props.metadata.date_taken && (
            <>
              <span class="text-[#333]">•</span>
              <span>{props.metadata.date_taken}</span>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
