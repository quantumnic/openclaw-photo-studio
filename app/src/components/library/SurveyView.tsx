import { createSignal, For } from "solid-js";

interface SurveyViewProps {
  photoIds: string[]; // 2-9 photos
}

interface PhotoMetadata {
  id: string;
  file_name: string;
  rating: number;
  flag: string;
}

export function SurveyView(props: SurveyViewProps) {
  const [activePhotoId, setActivePhotoId] = createSignal<string | null>(
    props.photoIds[0] || null
  );

  // Calculate grid layout based on number of photos
  const getGridLayout = (count: number): { cols: number; rows: number } => {
    if (count <= 2) return { cols: 2, rows: 1 };
    if (count <= 4) return { cols: 2, rows: 2 };
    if (count <= 6) return { cols: 3, rows: 2 };
    return { cols: 3, rows: 3 }; // 7-9 photos
  };

  const layout = () => getGridLayout(props.photoIds.length);

  // Mock metadata
  const getMetadata = (photoId: string): PhotoMetadata => {
    return {
      id: photoId,
      file_name: `Photo_${photoId.slice(0, 8)}.ARW`,
      rating: Math.floor(Math.random() * 6),
      flag: ["none", "pick", "reject"][Math.floor(Math.random() * 3)],
    };
  };

  return (
    <div class="h-full flex flex-col bg-[#0a0a0a]">
      {/* Toolbar */}
      <div class="h-10 bg-[#1c1c1c] border-b border-[#2a2a2a] flex items-center px-3 gap-3 text-xs text-[#aaa] flex-shrink-0">
        <span class="text-[#666] font-medium">Survey Mode</span>
        <span class="text-[#333]">|</span>
        <span class="text-[#444]">
          {props.photoIds.length} photos • {layout().cols}×{layout().rows} grid
        </span>

        <div class="ml-auto">
          <span class="text-[#333] text-[10px]">Press N to exit survey mode</span>
        </div>
      </div>

      {/* Grid */}
      <div class="flex-1 p-2 overflow-auto">
        <div
          class="grid gap-2 h-full"
          style={`grid-template-columns: repeat(${layout().cols}, 1fr); grid-template-rows: repeat(${layout().rows}, 1fr);`}
        >
          <For each={props.photoIds}>
            {(photoId) => (
              <SurveyCell
                photoId={photoId}
                metadata={getMetadata(photoId)}
                isActive={activePhotoId() === photoId}
                onClick={() => setActivePhotoId(photoId)}
              />
            )}
          </For>
        </div>
      </div>

      {/* Instructions */}
      <div class="h-8 bg-[#1c1c1c] border-t border-[#2a2a2a] flex items-center justify-center gap-4 text-[10px] text-[#444] flex-shrink-0">
        <span>Click to select</span>
        <span>•</span>
        <span>0-5 Rate active photo</span>
        <span>•</span>
        <span>P/X/U Flag active photo</span>
      </div>
    </div>
  );
}

interface SurveyCellProps {
  photoId: string;
  metadata: PhotoMetadata;
  isActive: boolean;
  onClick: () => void;
}

function SurveyCell(props: SurveyCellProps) {
  const flagColors: Record<string, string> = {
    pick: "#4a9eff",
    reject: "#ff4a4a",
    none: "#444",
  };

  return (
    <div
      onClick={props.onClick}
      class={`relative bg-[#111] rounded overflow-hidden cursor-pointer group transition-all ${
        props.isActive
          ? "ring-2 ring-[#4a9eff] scale-[1.02]"
          : "hover:ring-1 hover:ring-[#4a9eff]/50"
      }`}
    >
      {/* Photo placeholder */}
      <div class="absolute inset-0 flex items-center justify-center bg-gradient-to-br from-[#1a1a1a] to-[#0a0a0a]">
        <span class="text-4xl">📷</span>
      </div>

      {/* Metadata overlay */}
      <div class="absolute bottom-0 left-0 right-0 bg-black/80 backdrop-blur-sm p-2 space-y-1">
        {/* Filename */}
        <div class="text-[10px] text-[#ccc] truncate font-medium">
          {props.metadata.file_name}
        </div>

        {/* Rating & Flag */}
        <div class="flex items-center justify-between">
          {/* Rating stars */}
          <div class="flex gap-0.5">
            <For each={Array.from({ length: 5 })}>
              {(_, i) => (
                <span
                  class={`text-[10px] ${
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
              class="text-[10px] font-bold"
              style={`color: ${flagColors[props.metadata.flag]}`}
            >
              {props.metadata.flag === "pick" ? "P" : "X"}
            </span>
          )}
        </div>
      </div>

      {/* Active indicator */}
      {props.isActive && (
        <div class="absolute top-2 right-2 w-2 h-2 bg-[#4a9eff] rounded-full animate-pulse" />
      )}
    </div>
  );
}
