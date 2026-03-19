import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { createSignal, onMount, Show, For, onCleanup } from "solid-js";

type Module = "library" | "develop" | "map" | "print";

interface MainViewProps {
  module: Module;
  selectedPhotoId: string | null;
  onSelectPhoto: (id: string | null) => void;
}

export function MainView(props: MainViewProps) {
  return (
    <main class="flex-1 overflow-hidden bg-[#141414] relative">
      {props.module === "library" && (
        <LibraryView
          selectedPhotoId={props.selectedPhotoId}
          onSelectPhoto={props.onSelectPhoto}
        />
      )}
      {props.module === "develop" && <DevelopView selectedPhotoId={props.selectedPhotoId} />}
      {props.module === "map" && <PlaceholderView label="Map" emoji="🗺️" phase="Phase 5" />}
      {props.module === "print" && <PlaceholderView label="Print / Contact Sheet" emoji="🖨️" phase="Phase 7" />}
    </main>
  );
}

interface Photo {
  id: string;
  file_path: string;
  file_name: string;
  file_size: number;
  width: number | null;
  height: number | null;
  date_taken: string | null;
  date_imported: string;
  camera_make: string | null;
  camera_model: string | null;
  rating: number;
  color_label: string;
  flag: string;
  has_edits: boolean;
}

interface CatalogStats {
  total: number;
  rated: number;
  picks: number;
  rejects: number;
}

interface LibraryViewProps {
  selectedPhotoId: string | null;
  onSelectPhoto: (id: string | null) => void;
}

function LibraryView(props: LibraryViewProps) {
  const [photos, setPhotos] = createSignal<Photo[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [stats, setStats] = createSignal<CatalogStats | null>(null);
  const [thumbnailSize, setThumbnailSize] = createSignal(160);
  const [importResult, setImportResult] = createSignal<any>(null);
  const [error, setError] = createSignal<string | null>(null);

  // Load photos from catalog
  const loadPhotos = async () => {
    try {
      setLoading(true);
      const result = await invoke<Photo[]>("get_photos", {
        filter: {},
        limit: 500,
        offset: 0,
      });
      setPhotos(result);

      // Load stats
      try {
        const statsData = await invoke<CatalogStats>("get_catalog_stats");
        setStats(statsData);
      } catch (e) {
        console.warn("Failed to load stats:", e);
      }
    } catch (err) {
      console.error("Failed to load photos:", err);
      setPhotos([]);
    } finally {
      setLoading(false);
    }
  };

  // Import folder via dialog
  const handleImport = async () => {
    try {
      setError(null);
      const selected = await open({
        title: "Import Folder",
        directory: true,
        multiple: false,
      });

      if (!selected) return;

      const path = typeof selected === "string" ? selected : selected.path;
      setLoading(true);

      const result = await invoke<any>("import_folder", { path });
      setImportResult(result);

      // Reload photos after import
      await loadPhotos();
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  // Keyboard shortcuts
  const handleKeyDown = async (e: KeyboardEvent) => {
    const selected = props.selectedPhotoId;
    if (!selected) return;

    // Rating: 0-5
    if (e.key >= "0" && e.key <= "5") {
      e.preventDefault();
      const rating = parseInt(e.key);
      try {
        await invoke("update_rating", { photoId: selected, rating });
        // Update local state
        setPhotos((prev) =>
          prev.map((p) => (p.id === selected ? { ...p, rating } : p))
        );
      } catch (err) {
        console.error("Failed to update rating:", err);
      }
    }

    // Flag: P = pick, X = reject, U = unflag
    if (e.key === "p" || e.key === "P") {
      e.preventDefault();
      try {
        await invoke("update_flag", { photoId: selected, flag: "pick" });
        setPhotos((prev) =>
          prev.map((p) => (p.id === selected ? { ...p, flag: "pick" } : p))
        );
      } catch (err) {
        console.error("Failed to update flag:", err);
      }
    }

    if (e.key === "x" || e.key === "X") {
      e.preventDefault();
      try {
        await invoke("update_flag", { photoId: selected, flag: "reject" });
        setPhotos((prev) =>
          prev.map((p) => (p.id === selected ? { ...p, flag: "reject" } : p))
        );
      } catch (err) {
        console.error("Failed to update flag:", err);
      }
    }

    if (e.key === "u" || e.key === "U") {
      e.preventDefault();
      try {
        await invoke("update_flag", { photoId: selected, flag: "none" });
        setPhotos((prev) =>
          prev.map((p) => (p.id === selected ? { ...p, flag: "none" } : p))
        );
      } catch (err) {
        console.error("Failed to update flag:", err);
      }
    }

    // Color labels: 6=red, 7=yellow, 8=green, 9=blue
    const colorMap: Record<string, string> = {
      "6": "red",
      "7": "yellow",
      "8": "green",
      "9": "blue",
    };

    if (colorMap[e.key]) {
      e.preventDefault();
      try {
        await invoke("update_color_label", {
          photoId: selected,
          label: colorMap[e.key],
        });
        setPhotos((prev) =>
          prev.map((p) =>
            p.id === selected ? { ...p, color_label: colorMap[e.key] } : p
          )
        );
      } catch (err) {
        console.error("Failed to update color label:", err);
      }
    }

    // Arrow navigation
    if (e.key === "ArrowRight" || e.key === "ArrowLeft") {
      e.preventDefault();
      const currentIndex = photos().findIndex((p) => p.id === selected);
      if (currentIndex === -1) return;

      const newIndex =
        e.key === "ArrowRight" ? currentIndex + 1 : currentIndex - 1;
      if (newIndex >= 0 && newIndex < photos().length) {
        props.onSelectPhoto(photos()[newIndex].id);
      }
    }
  };

  onMount(() => {
    loadPhotos();
    document.addEventListener("keydown", handleKeyDown);
    onCleanup(() => document.removeEventListener("keydown", handleKeyDown));
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
    <div class="h-full flex flex-col">
      {/* Toolbar */}
      <div class="h-10 bg-[#1c1c1c] border-b border-[#2a2a2a] flex items-center px-3 gap-3 text-xs text-[#aaa] flex-shrink-0">
        <button
          onClick={handleImport}
          disabled={loading()}
          class="px-3 py-1 bg-[#4a9eff] hover:bg-[#5aa9ff] disabled:bg-[#2a4a7f] text-white rounded text-xs font-medium transition-colors"
        >
          {loading() ? "Importing..." : "📁 Import Folder"}
        </button>

        <Show when={stats()}>
          {(s) => (
            <>
              <span class="text-[#333]">|</span>
              <span class="text-[#666]">Total: {s().total}</span>
              <span class="text-[#333]">|</span>
              <span class="text-[#666]">Rated: {s().rated}</span>
              <span class="text-[#333]">|</span>
              <span class="text-[#4a9eff]">Picks: {s().picks}</span>
              <span class="text-[#333]">|</span>
              <span class="text-[#ff4a4a]">Rejects: {s().rejects}</span>
            </>
          )}
        </Show>

        <div class="ml-auto flex items-center gap-2">
          <span class="text-[#666]">{photos().length} photos</span>
        </div>
      </div>

      {/* Import result */}
      <Show when={importResult()}>
        {(result) => (
          <div class="px-4 py-2 bg-[#1a2a1a] border-b border-[#2a3a2a] text-xs text-[#4ade80]">
            ✓ Imported {result().inserted} photos ({result().skipped} skipped,{" "}
            {result().errors.length} errors)
          </div>
        )}
      </Show>

      {/* Error message */}
      <Show when={error()}>
        {(err) => (
          <div class="px-4 py-2 bg-[#2a1a1a] border-b border-[#3a2a2a] text-xs text-[#ff6b6b]">
            ✗ {err()}
          </div>
        )}
      </Show>

      {/* Photo Grid */}
      <div class="flex-1 overflow-auto p-3">
        <Show
          when={photos().length > 0}
          fallback={
            <div class="h-full flex flex-col items-center justify-center text-[#444]">
              <div class="text-6xl mb-4">📸</div>
              <div class="text-[#666] font-medium mb-2">No Photos</div>
              <div class="text-xs text-[#444] mb-4">
                Click "Import Folder" to get started
              </div>
              <div class="text-xs text-[#333] space-y-1 text-center">
                <div>Supported formats: ARW, NEF, RAF, DNG, CR2, CR3, ORF, RW2, JPG, TIFF, PNG</div>
                <div class="mt-4 pt-4 border-t border-[#2a2a2a]">
                  <div class="font-medium text-[#4a9eff] mb-2">Keyboard Shortcuts</div>
                  <div>0-5: Set rating • P: Pick • X: Reject • U: Unflag</div>
                  <div>6: Red • 7: Yellow • 8: Green • 9: Blue</div>
                  <div>←→: Navigate photos</div>
                </div>
              </div>
            </div>
          }
        >
          <div
            class="grid gap-1"
            style={`grid-template-columns: repeat(auto-fill, minmax(${thumbnailSize()}px, 1fr))`}
          >
            <For each={photos()}>
              {(photo) => (
                <div
                  onClick={() => props.onSelectPhoto(photo.id)}
                  class={`relative bg-[#1a1a1a] rounded overflow-hidden cursor-pointer group hover:ring-1 hover:ring-[#4a9eff] transition-all aspect-[3/2] ${
                    props.selectedPhotoId === photo.id
                      ? "ring-2 ring-[#4a9eff]"
                      : ""
                  }`}
                >
                  {/* Placeholder image (thumbnail loading comes later) */}
                  <div class="absolute inset-0 bg-gradient-to-br from-[#222] to-[#111] flex items-center justify-center">
                    <span class="text-[#333] text-2xl">📷</span>
                  </div>

                  {/* File name overlay */}
                  <div class="absolute top-0 left-0 right-0 px-1 py-0.5 bg-black/60 text-[8px] text-[#aaa] truncate opacity-0 group-hover:opacity-100 transition-opacity">
                    {photo.file_name}
                  </div>

                  {/* Info overlay */}
                  <div class="absolute bottom-0 left-0 right-0 px-1 py-0.5 bg-black/60 flex items-center gap-1">
                    {/* Rating stars */}
                    <div class="flex gap-0.5">
                      <For each={Array.from({ length: 5 })}>
                        {(_, i) => (
                          <span
                            class={`text-[8px] ${
                              i() < photo.rating
                                ? "text-[#e8b84b]"
                                : "text-[#333]"
                            }`}
                          >
                            ★
                          </span>
                        )}
                      </For>
                    </div>

                    {/* Flag indicator */}
                    {photo.flag === "pick" && (
                      <span class="text-[8px] text-[#4a9eff] ml-auto font-bold">
                        P
                      </span>
                    )}
                    {photo.flag === "reject" && (
                      <span class="text-[8px] text-[#ff4a4a] ml-auto font-bold">
                        X
                      </span>
                    )}

                    {/* Color label dot */}
                    {photo.color_label !== "none" && (
                      <div
                        class="w-1.5 h-1.5 rounded-full ml-auto"
                        style={`background-color: ${
                          colorLabelColors[photo.color_label]
                        }`}
                      />
                    )}
                  </div>
                </div>
              )}
            </For>
          </div>

          {/* Instructions */}
          <div class="mt-8 text-center text-[#444] text-xs space-y-1">
            <div class="text-[#666] mb-2">✓ Catalog + XMP + Library Grid — Phase 1 Day 36-70 Complete</div>
            <div class="text-[#333]">
              Click a photo to select • Use keyboard shortcuts to rate and flag
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
}

interface DevelopViewProps {
  selectedPhotoId: string | null;
}

function DevelopView(props: DevelopViewProps) {
  const [beforeAfterMode, setBeforeAfterMode] = createSignal(false);

  // Keyboard shortcut for before/after
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

    if (e.key === "\\" && props.selectedPhotoId) {
      e.preventDefault();
      setBeforeAfterMode((m) => !m);
    }
  };

  onMount(() => {
    document.addEventListener("keydown", handleKeyDown);
    onCleanup(() => document.removeEventListener("keydown", handleKeyDown));
  });

  return (
    <div class="h-full flex flex-col">
      <Show
        when={props.selectedPhotoId}
        fallback={
          <div class="h-full flex flex-col items-center justify-center text-[#444]">
            <div class="text-6xl mb-4">🎛️</div>
            <div class="text-[#666] font-medium mb-1">Develop Module</div>
            <div class="text-xs text-[#444] mb-2">Select a photo in Library to edit</div>
            <div class="text-xs text-[#333]">
              Sliders are live on the right →
            </div>
          </div>
        }
      >
        <div class="h-full flex flex-col items-center justify-center bg-[#0a0a0a] relative">
          {/* Before/After Toggle Indicator */}
          <Show when={beforeAfterMode()}>
            <div class="absolute top-4 left-4 px-3 py-1 bg-black/80 text-xs text-[#4a9eff] rounded font-mono">
              BEFORE
            </div>
          </Show>

          {/* Photo Preview Placeholder */}
          <div class="text-6xl mb-4">📷</div>
          <div class="text-[#666] font-medium mb-1">Photo ID: {props.selectedPhotoId.slice(0, 8)}...</div>
          <div class="text-xs text-[#444] mb-2">Image preview coming in Phase 2</div>
          <div class="text-xs text-[#333] flex items-center gap-2">
            <span>Press <kbd class="px-1.5 py-0.5 bg-[#1a1a1a] border border-[#333] rounded text-[10px] font-mono">\</kbd> for Before/After</span>
          </div>

          {/* Status */}
          <div class="absolute bottom-4 right-4 text-xs text-[#444]">
            Mode: {beforeAfterMode() ? "Before" : "After"}
          </div>
        </div>
      </Show>
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
