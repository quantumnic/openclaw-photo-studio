import { invoke } from "@tauri-apps/api/core";
import { createSignal, onMount, Show, For } from "solid-js";
import { createStore } from "solid-js/store";

interface FilmstripProps {
  photoIds: string[];
  selectedId: string | null;
  onSelect: (id: string) => void;
}

export function Filmstrip(props: FilmstripProps) {
  const [thumbnails, setThumbnails] = createStore<Record<string, string>>({});

  // Load thumbnails for visible photos (±5 around selection)
  const loadThumbnails = async () => {
    if (props.photoIds.length === 0) return;

    // Find selected index
    const selectedIndex = props.selectedId ? props.photoIds.indexOf(props.selectedId) : 0;
    const startIndex = Math.max(0, selectedIndex - 5);
    const endIndex = Math.min(props.photoIds.length, selectedIndex + 15);

    // Load thumbnails for this range
    const idsToLoad = props.photoIds.slice(startIndex, endIndex).filter(id => !thumbnails[id]);

    for (const photoId of idsToLoad) {
      try {
        const dataUri = await invoke<string>("render_thumbnail", { photoId });
        setThumbnails(photoId, dataUri);
      } catch (e) {
        console.error("Failed to load filmstrip thumbnail:", e);
      }
    }
  };

  onMount(() => {
    loadThumbnails();
  });

  // Reload thumbnails when selection changes
  onMount(() => {
    const interval = setInterval(() => {
      if (props.selectedId) {
        loadThumbnails();
      }
    }, 500);
    return () => clearInterval(interval);
  });

  return (
    <div class="h-20 bg-[#111] border-t border-[#2a2a2a] flex items-center gap-1 px-2 overflow-x-auto flex-shrink-0">
      <Show when={props.photoIds.length > 0} fallback={<span class="text-[#333] text-xs">No photos</span>}>
        <For each={props.photoIds}>
          {(photoId) => (
            <div
              onClick={() => props.onSelect(photoId)}
              class={`h-16 w-24 flex-shrink-0 rounded overflow-hidden cursor-pointer relative
                ${photoId === props.selectedId ? "ring-2 ring-[#4a9eff]" : "hover:ring-1 hover:ring-[#555]"}`}
            >
              <Show
                when={thumbnails[photoId]}
                fallback={
                  <div class="absolute inset-0 bg-[#1a1a1a] flex items-center justify-center">
                    <span class="text-[#2a2a2a] text-sm">📷</span>
                  </div>
                }
              >
                {(src) => (
                  <img src={src()} class="w-full h-full object-cover" alt="" />
                )}
              </Show>
            </div>
          )}
        </For>
      </Show>
    </div>
  );
}
