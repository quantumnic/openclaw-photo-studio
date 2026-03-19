import { invoke } from "@tauri-apps/api/core";
import { createSignal, onMount, onCleanup, Show, For } from "solid-js";

interface Photo {
  id: string;
  file_name: string;
  rating: number;
  flag: string;
  color_label: string;
}

interface PhotoCardProps {
  photo: Photo;
  selected: boolean;
  focused?: boolean;
  thumbnailSize: number;
  onSelect: (id: string) => void;
}

export function PhotoCard(props: PhotoCardProps) {
  const [thumbSrc, setThumbSrc] = createSignal<string | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal(false);
  let cardRef: HTMLDivElement | undefined;
  let observer: IntersectionObserver | undefined;

  // Load thumbnail when card is in viewport
  const loadThumbnail = async () => {
    if (loading() || thumbSrc()) return;

    setLoading(true);
    try {
      const result = await invoke<{ data: string; width: number; height: number }>(
        "get_thumbnail",
        { photoId: props.photo.id, maxSize: props.thumbnailSize }
      );
      setThumbSrc(`data:image/jpeg;base64,${result.data}`);
    } catch (e) {
      console.error("Failed to load thumbnail:", e);
      setError(true);
    }
    setLoading(false);
  };

  onMount(() => {
    // Use Intersection Observer for lazy loading
    observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && !thumbSrc()) {
          loadThumbnail();
        }
      },
      { threshold: 0.1 }
    );

    if (cardRef) {
      observer.observe(cardRef);
    }

    onCleanup(() => {
      if (observer) {
        observer.disconnect();
      }
    });
  });

  const colorLabelColors: Record<string, string> = {
    red: "#ff4a4a",
    yellow: "#e8b84b",
    green: "#4ade80",
    blue: "#4a9eff",
    purple: "#a855f7",
    none: "transparent",
  };

  return (
    <div
      ref={cardRef}
      data-photo-card
      class={`relative bg-[#1a1a1a] rounded overflow-hidden cursor-pointer group
        ${props.selected ? "ring-2 ring-[#4a9eff]" : props.focused ? "ring-2 ring-[#888]" : "hover:ring-1 hover:ring-[#444]"}
      `}
      style={`aspect-ratio: 3/2`}
      onClick={() => props.onSelect(props.photo.id)}
    >
      {/* Photo content */}
      <Show
        when={thumbSrc()}
        fallback={
          <Show
            when={loading()}
            fallback={
              <Show
                when={error()}
                fallback={
                  <div class="absolute inset-0 bg-gradient-to-br from-[#222] to-[#111] flex items-center justify-center">
                    <span class="text-[#333] text-2xl">📷</span>
                  </div>
                }
              >
                <div class="absolute inset-0 flex items-center justify-center text-[#333]">
                  ⚠️
                </div>
              </Show>
            }
          >
            <div class="absolute inset-0 flex items-center justify-center">
              <div class="w-4 h-4 border-2 border-[#444] border-t-[#4a9eff] rounded-full animate-spin" />
            </div>
          </Show>
        }
      >
        {(src) => (
          <img
            src={src()}
            class="absolute inset-0 w-full h-full object-cover"
            alt={props.photo.file_name}
          />
        )}
      </Show>

      {/* Bottom overlay with metadata */}
      <div class="absolute bottom-0 left-0 right-0 px-1 py-0.5 bg-gradient-to-t from-black/70 to-transparent">
        <div class="flex items-center gap-1">
          {/* Stars */}
          <div class="flex gap-0.5">
            <For each={Array.from({ length: 5 })}>
              {(_, i) => (
                <span
                  class={`text-[8px] ${
                    i() < props.photo.rating ? "text-[#e8b84b]" : "text-[#333]"
                  }`}
                >
                  ★
                </span>
              )}
            </For>
          </div>

          {/* Flag */}
          <Show when={props.photo.flag === "pick"}>
            <span class="text-[8px] text-[#4a9eff] ml-auto">P</span>
          </Show>
          <Show when={props.photo.flag === "reject"}>
            <span class="text-[8px] text-[#ff4a4a] ml-auto">X</span>
          </Show>

          {/* Color label dot */}
          <Show when={props.photo.color_label !== "none"}>
            <div
              class={`w-2 h-2 rounded-full ml-auto`}
              style={`background-color: ${colorLabelColors[props.photo.color_label]}`}
            />
          </Show>
        </div>
      </div>

      {/* Filename tooltip on hover */}
      <div class="absolute top-0 left-0 right-0 px-1 py-0.5 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity">
        <span class="text-[9px] text-[#aaa] truncate block">{props.photo.file_name}</span>
      </div>
    </div>
  );
}
