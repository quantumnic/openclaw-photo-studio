import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { createSignal, onMount, Show, For, onCleanup, createEffect } from "solid-js";
import { createStore } from "solid-js/store";
import { FilterBar, FilterState } from "../library/FilterBar";
import { CompareView } from "../library/CompareView";
import { SurveyView } from "../library/SurveyView";
import { MapView } from "../map/MapView";
import { PrintView } from "../print/PrintView";
import { PhotoCard } from "../library/PhotoCard";
import { showToast } from "../common/Toast";

type Module = "library" | "develop" | "map" | "print";
type ViewMode = "grid" | "loupe" | "compare" | "survey";

interface MainViewProps {
  module: Module;
  viewMode: ViewMode;
  selectedPhotoId: string | null;
  selectedPhotoIds: string[];
  onSelectPhoto: (id: string | null) => void;
  onPhotosLoaded?: (photoIds: string[]) => void;
}

export function MainView(props: MainViewProps) {
  return (
    <main id="main-content" class="flex-1 overflow-hidden bg-[#141414] relative">
      {props.module === "library" && (
        <LibraryView
          viewMode={props.viewMode}
          selectedPhotoId={props.selectedPhotoId}
          onSelectPhoto={props.onSelectPhoto}
          onPhotosLoaded={props.onPhotosLoaded}
        />
      )}
      {props.module === "develop" && <DevelopView selectedPhotoId={props.selectedPhotoId} />}
      {props.module === "map" && <MapView />}
      {props.module === "print" && <PrintView selectedPhotoIds={props.selectedPhotoIds} />}
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
  viewMode: ViewMode;
  selectedPhotoId: string | null;
  onSelectPhoto: (id: string | null) => void;
}

function LibraryView(props: LibraryViewProps & { onPhotosLoaded?: (photoIds: string[]) => void }) {
  const [comparePhotoIds, setComparePhotoIds] = createSignal<[string, string]>(["photo1", "photo2"]);
  const [surveyPhotoIds, setSurveyPhotoIds] = createSignal<string[]>(["p1", "p2", "p3", "p4", "p5", "p6"]);

  const handleSwap = () => {
    setComparePhotoIds(([a, b]) => [b, a]);
  };

  // Show compare/survey modes if active
  if (props.viewMode === "compare") {
    return <CompareView photoIds={comparePhotoIds()} onSwap={handleSwap} />;
  }

  if (props.viewMode === "survey") {
    return <SurveyView photoIds={surveyPhotoIds()} />;
  }

  // Show loupe view if selected and in loupe mode
  if (props.viewMode === "loupe" && props.selectedPhotoId) {
    return (
      <LoupeView
        photoId={props.selectedPhotoId}
        onBack={() => props.onSelectPhoto(null)}
      />
    );
  }

  // Otherwise show grid
  return <LibraryGridView {...props} onPhotosLoaded={props.onPhotosLoaded} />;
}

interface LoupeViewProps {
  photoId: string;
  onBack: () => void;
}

function LoupeView(props: LoupeViewProps) {
  const [previewUri, setPreviewUri] = createSignal<string | null>(null);
  const [photo, setPhoto] = createSignal<Photo | null>(null);
  const [loading, setLoading] = createSignal(true);

  onMount(async () => {
    try {
      // Load full photo info
      const p = await invoke<Photo>("get_photo", { photoId: props.photoId });
      setPhoto(p);

      // Load large preview
      const result = await invoke<{ data_uri: string }>("render_preview", {
        photoId: props.photoId,
        recipe: null,
        maxWidth: 1920,
        maxHeight: 1080,
      });
      setPreviewUri(result.data_uri);
    } catch (e) {
      console.error("Failed to load loupe view:", e);
    } finally {
      setLoading(false);
    }
  });

  return (
    <div class="h-full flex flex-col bg-[#0f0f0f]">
      <div class="h-8 bg-[#1a1a1a] border-b border-[#2a2a2a] flex items-center px-3 gap-2 flex-shrink-0">
        <button
          onClick={props.onBack}
          class="text-xs text-[#666] hover:text-[#aaa]"
        >
          ← Grid (G)
        </button>
        <Show when={photo()}>
          {(p) => (
            <span class="text-xs text-[#555]">{p().file_name}</span>
          )}
        </Show>
      </div>
      <div class="flex-1 flex items-center justify-center">
        <Show
          when={!loading() && previewUri()}
          fallback={
            <div class="flex flex-col items-center gap-3 text-[#444]">
              <div class="w-8 h-8 border-2 border-[#333] border-t-[#4a9eff] rounded-full animate-spin" />
              <span class="text-xs">Loading photo...</span>
            </div>
          }
        >
          {(uri) => (
            <img
              src={uri()}
              class="max-w-full max-h-full object-contain"
              alt={photo()?.file_name || "Photo"}
            />
          )}
        </Show>
      </div>
    </div>
  );
}

interface LibraryGridViewProps {
  selectedPhotoId: string | null;
  onSelectPhoto: (id: string | null) => void;
  onPhotosLoaded?: (photoIds: string[]) => void;
}

function LibraryGridView(props: LibraryGridViewProps) {
  const [photos, setPhotos] = createSignal<Photo[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [stats, setStats] = createSignal<CatalogStats | null>(null);
  const [thumbnailSize, setThumbnailSize] = createSignal(160);
  const [importResult, setImportResult] = createSignal<any>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [focusedIndex, setFocusedIndex] = createSignal<number>(-1);

  // Filter state
  const [filter, setFilter] = createStore<FilterState>({
    ratingMin: 0,
    flag: "all",
    colorLabel: "all",
    searchQuery: "",
  });

  // Load photos from catalog
  const loadPhotos = async () => {
    try {
      setLoading(true);

      // Build filter object
      const filterObj: any = {};
      if (filter.ratingMin > 0) {
        filterObj.rating_min = filter.ratingMin;
      }
      if (filter.flag !== "all") {
        filterObj.flag = filter.flag;
      }
      if (filter.colorLabel !== "all") {
        filterObj.color_label = filter.colorLabel;
      }
      if (filter.searchQuery.trim()) {
        filterObj.search = filter.searchQuery.trim();
      }

      const result = await invoke<Photo[]>("get_photos", {
        filter: filterObj,
        limit: 500,
        offset: 0,
      });
      setPhotos(result);

      // Notify parent of loaded photos
      if (props.onPhotosLoaded) {
        props.onPhotosLoaded(result.map(p => p.id));
      }

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

      const startTime = Date.now();
      const result = await invoke<any>("import_folder", { path });
      const duration = ((Date.now() - startTime) / 1000).toFixed(1);

      setImportResult(result);

      // Store in recent folders (localStorage)
      const recentKey = "recent_import_folders";
      const recent = JSON.parse(localStorage.getItem(recentKey) || "[]");
      const updated = [path, ...recent.filter((p: string) => p !== path)].slice(0, 5);
      localStorage.setItem(recentKey, JSON.stringify(updated));

      // Reload photos after import
      await loadPhotos();

      // Show success toast
      showToast(`Imported ${result.inserted} photos in ${duration}s`, "success", 3000);
    } catch (err) {
      setError(String(err));
      showToast(`Import failed: ${err}`, "error", 5000);
    } finally {
      setLoading(false);
    }
  };

  // Calculate grid columns for arrow key navigation
  const getGridColumns = () => {
    const containerWidth = 1200; // Approximate, could be measured
    return Math.floor(containerWidth / (thumbnailSize() + 4)); // 4 = gap
  };

  // Scroll element into view
  const scrollIntoView = (index: number) => {
    const photoCards = document.querySelectorAll("[data-photo-card]");
    if (photoCards[index]) {
      photoCards[index].scrollIntoView({ behavior: "smooth", block: "nearest" });
    }
  };

  // Keyboard shortcuts
  const handleKeyDown = async (e: KeyboardEvent) => {
    // Navigation keys work with focused index
    const currentFocus = focusedIndex();
    const currentPhotos = photos();
    const gridColumns = getGridColumns();

    // Arrow keys for navigation
    if (e.key === "ArrowRight") {
      e.preventDefault();
      const newIndex = Math.min(currentFocus + 1, currentPhotos.length - 1);
      setFocusedIndex(newIndex);
      props.onSelectPhoto(currentPhotos[newIndex]?.id || null);
      scrollIntoView(newIndex);
      return;
    }

    if (e.key === "ArrowLeft") {
      e.preventDefault();
      const newIndex = Math.max(currentFocus - 1, 0);
      setFocusedIndex(newIndex);
      props.onSelectPhoto(currentPhotos[newIndex]?.id || null);
      scrollIntoView(newIndex);
      return;
    }

    if (e.key === "ArrowDown") {
      e.preventDefault();
      const newIndex = Math.min(currentFocus + gridColumns, currentPhotos.length - 1);
      setFocusedIndex(newIndex);
      props.onSelectPhoto(currentPhotos[newIndex]?.id || null);
      scrollIntoView(newIndex);
      return;
    }

    if (e.key === "ArrowUp") {
      e.preventDefault();
      const newIndex = Math.max(currentFocus - gridColumns, 0);
      setFocusedIndex(newIndex);
      props.onSelectPhoto(currentPhotos[newIndex]?.id || null);
      scrollIntoView(newIndex);
      return;
    }

    if (e.key === "Home") {
      e.preventDefault();
      setFocusedIndex(0);
      props.onSelectPhoto(currentPhotos[0]?.id || null);
      scrollIntoView(0);
      return;
    }

    if (e.key === "End") {
      e.preventDefault();
      const lastIndex = currentPhotos.length - 1;
      setFocusedIndex(lastIndex);
      props.onSelectPhoto(currentPhotos[lastIndex]?.id || null);
      scrollIntoView(lastIndex);
      return;
    }

    if (e.key === "PageDown") {
      e.preventDefault();
      const newIndex = Math.min(currentFocus + gridColumns * 5, currentPhotos.length - 1);
      setFocusedIndex(newIndex);
      props.onSelectPhoto(currentPhotos[newIndex]?.id || null);
      scrollIntoView(newIndex);
      return;
    }

    if (e.key === "PageUp") {
      e.preventDefault();
      const newIndex = Math.max(currentFocus - gridColumns * 5, 0);
      setFocusedIndex(newIndex);
      props.onSelectPhoto(currentPhotos[newIndex]?.id || null);
      scrollIntoView(newIndex);
      return;
    }

    if (e.key === "Enter") {
      e.preventDefault();
      // Enter opens loupe view - handled by AppShell
      return;
    }

    // All remaining shortcuts require a selected photo
    const selected = props.selectedPhotoId;
    if (!selected) return;

    // Rating: 0-5 (auto-advance to next after rating)
    if (e.key >= "0" && e.key <= "5") {
      e.preventDefault();
      const rating = parseInt(e.key);
      try {
        await invoke("update_rating", { photoId: selected, rating });
        setPhotos((prev) =>
          prev.map((p) => (p.id === selected ? { ...p, rating } : p))
        );
        // Auto-advance to next photo
        const newIndex = Math.min(currentFocus + 1, currentPhotos.length - 1);
        if (newIndex > currentFocus) {
          setFocusedIndex(newIndex);
          props.onSelectPhoto(currentPhotos[newIndex]?.id || null);
          scrollIntoView(newIndex);
        }
      } catch (err) {
        console.error("Failed to update rating:", err);
      }
    }

    // Flag: P = pick, X = reject, U = unflag (auto-advance)
    if (e.key === "p" || e.key === "P") {
      e.preventDefault();
      try {
        await invoke("update_flag", { photoId: selected, flag: "pick" });
        setPhotos((prev) =>
          prev.map((p) => (p.id === selected ? { ...p, flag: "pick" } : p))
        );
        const newIndex = Math.min(currentFocus + 1, currentPhotos.length - 1);
        if (newIndex > currentFocus) {
          setFocusedIndex(newIndex);
          props.onSelectPhoto(currentPhotos[newIndex]?.id || null);
          scrollIntoView(newIndex);
        }
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
        const newIndex = Math.min(currentFocus + 1, currentPhotos.length - 1);
        if (newIndex > currentFocus) {
          setFocusedIndex(newIndex);
          props.onSelectPhoto(currentPhotos[newIndex]?.id || null);
          scrollIntoView(newIndex);
        }
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
  };

  onMount(() => {
    loadPhotos();
    document.addEventListener("keydown", handleKeyDown);
    onCleanup(() => document.removeEventListener("keydown", handleKeyDown));
  });

  // Reload photos when filter changes
  createEffect(() => {
    // Track all filter properties
    const _ = [filter.ratingMin, filter.flag, filter.colorLabel, filter.searchQuery];
    loadPhotos();
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

      {/* Filter Bar */}
      <FilterBar
        filter={filter}
        onFilterChange={setFilter}
        totalCount={stats()?.total || 0}
        filteredCount={photos().length}
      />

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
              {(photo, index) => (
                <PhotoCard
                  photo={photo}
                  selected={props.selectedPhotoId === photo.id}
                  focused={focusedIndex() === index()}
                  thumbnailSize={thumbnailSize()}
                  onSelect={(id) => {
                    props.onSelectPhoto(id);
                    setFocusedIndex(index());
                  }}
                />
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
  const [beforeAfter, setBeforeAfter] = createSignal(false);
  const [previewUri, setPreviewUri] = createSignal<string | null>(null);
  const [rendering, setRendering] = createSignal(false);
  const [previewDims, setPreviewDims] = createSignal({ w: 0, h: 0 });
  const [zoom, setZoom] = createSignal(0); // 0 = fit, 1.0 = 100%, 2.0 = 200%
  const [pan, setPan] = createStore({ x: 0, y: 0 });
  const [isPanning, setIsPanning] = createSignal(false);
  const [panStart, setPanStart] = createStore({ x: 0, y: 0, panX: 0, panY: 0 });

  let renderTimer: ReturnType<typeof setTimeout> | null = null;
  let imgContainerRef: HTMLDivElement | undefined;

  // Update preview when photo changes or recipe changes
  async function updatePreview(photoId: string, recipe?: any) {
    setRendering(true);
    try {
      const result = await invoke<{
        data_uri: string;
        width: number;
        height: number;
        duration_ms: number;
      }>("render_preview", {
        photoId,
        recipe: beforeAfter() ? null : recipe, // null = before (default recipe)
        maxWidth: 1200,
        maxHeight: 800,
      });
      setPreviewUri(result.data_uri);
      setPreviewDims({ w: result.width, h: result.height });
    } catch (e) {
      console.error("Render failed:", e);
    } finally {
      setRendering(false);
    }
  }

  // Load preview when photo changes
  createEffect(async () => {
    const id = props.selectedPhotoId;
    if (!id) {
      setPreviewUri(null);
      return;
    }
    await updatePreview(id);
  });

  // Zoom functions
  function zoomIn() {
    const current = zoom();
    if (current === 0) setZoom(1.0); // fit -> 100%
    else if (current === 1.0) setZoom(2.0); // 100% -> 200%
    else if (current < 4.0) setZoom(Math.min(4.0, current + 0.5));
  }

  function zoomOut() {
    const current = zoom();
    if (current > 1.0) setZoom(Math.max(1.0, current - 0.5));
    else setZoom(0); // -> fit
  }

  function zoomFit() {
    setZoom(0);
    setPan({ x: 0, y: 0 });
  }

  function zoom100() {
    setZoom(1.0);
    setPan({ x: 0, y: 0 });
  }

  function toggleFit100() {
    if (zoom() === 0) zoom100();
    else zoomFit();
  }

  // Mouse pan handling
  function handleMouseDown(e: MouseEvent) {
    if (zoom() > 0 && e.button === 0 && e.spaceKey !== undefined ? e.spaceKey : false) {
      setIsPanning(true);
      setPanStart({ x: e.clientX, y: e.clientY, panX: pan.x, panY: pan.y });
      e.preventDefault();
    }
  }

  function handleMouseMove(e: MouseEvent) {
    if (isPanning()) {
      const dx = e.clientX - panStart.x;
      const dy = e.clientY - panStart.y;
      setPan({ x: panStart.panX + dx, y: panStart.panY + dy });
      e.preventDefault();
    }
  }

  function handleMouseUp() {
    setIsPanning(false);
  }

  // Keyboard shortcut for before/after toggle and zoom
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

    if (e.key === "\\" && props.selectedPhotoId) {
      e.preventDefault();
      const newMode = !beforeAfter();
      setBeforeAfter(newMode);
      updatePreview(props.selectedPhotoId);
    }

    const mod = e.metaKey || e.ctrlKey;

    // Zoom shortcuts
    if (mod && (e.key === "=" || e.key === "+")) {
      e.preventDefault();
      zoomIn();
    }
    if (mod && e.key === "-") {
      e.preventDefault();
      zoomOut();
    }
    if (mod && e.key === "0") {
      e.preventDefault();
      if (e.altKey) zoom100(); // Cmd+Alt+0 = 1:1
      else zoomFit(); // Cmd+0 = fit
    }
    if (e.key === "z" || e.key === "Z") {
      e.preventDefault();
      toggleFit100();
    }
  };

  onMount(() => {
    document.addEventListener("keydown", handleKeyDown);
    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
    onCleanup(() => {
      document.removeEventListener("keydown", handleKeyDown);
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
      if (renderTimer) clearTimeout(renderTimer);
    });
  });

  const zoomLabel = () => {
    if (zoom() === 0) return "Fit";
    return `${Math.round(zoom() * 100)}%`;
  };

  const containerStyle = () => {
    if (zoom() === 0) return { overflow: "hidden" };
    return { overflow: "auto", cursor: isPanning() ? "grabbing" : "grab" };
  };

  const imageStyle = () => {
    if (zoom() === 0) {
      return {
        "max-width": "100%",
        "max-height": "100%",
        "object-fit": "contain",
        opacity: rendering() ? "0.7" : "1",
        transition: "opacity 0.1s",
      };
    }
    return {
      width: `${zoom() * 100}%`,
      "max-width": "none",
      "max-height": "none",
      transform: `translate(${pan.x}px, ${pan.y}px)`,
      opacity: rendering() ? "0.7" : "1",
      transition: "opacity 0.1s",
      cursor: "inherit",
    };
  };

  return (
    <div class="h-full flex flex-col">
      {/* Toolbar */}
      <div class="h-8 bg-[#1c1c1c] border-b border-[#2a2a2a] flex items-center px-3 gap-3 flex-shrink-0">
        <button
          onClick={() => {
            setBeforeAfter((b) => !b);
            if (props.selectedPhotoId) updatePreview(props.selectedPhotoId);
          }}
          class={`text-xs px-2 py-0.5 rounded transition-colors ${
            beforeAfter()
              ? "bg-[#3a3a3a] text-white"
              : "text-[#666] hover:text-[#999]"
          }`}
        >
          {beforeAfter() ? "◀ Before" : "After ▶"} (\)
        </button>

        {/* Zoom controls */}
        <span class="text-[#333]">|</span>
        <button
          onClick={zoomOut}
          class="text-xs px-2 py-0.5 text-[#666] hover:text-[#999] rounded hover:bg-[#2a2a2a]"
          title="Zoom Out (Cmd+-)"
          aria-label="Zoom Out"
        >
          −
        </button>
        <span
          class="text-xs text-[#888] min-w-[2.5rem] text-center cursor-pointer hover:text-[#aaa]"
          onClick={zoomFit}
          title="Click to fit (Cmd+0)"
        >
          {zoomLabel()}
        </span>
        <button
          onClick={zoomIn}
          class="text-xs px-2 py-0.5 text-[#666] hover:text-[#999] rounded hover:bg-[#2a2a2a]"
          title="Zoom In (Cmd+=)"
          aria-label="Zoom In"
        >
          +
        </button>

        <Show when={rendering()}>
          <div class="w-3 h-3 border border-[#333] border-t-[#4a9eff] rounded-full animate-spin" />
        </Show>
        <Show when={previewDims().w > 0}>
          <span class="text-xs text-[#444] ml-auto">
            {previewDims().w} × {previewDims().h}
          </span>
        </Show>
      </div>

      {/* Image area */}
      <div
        ref={imgContainerRef}
        class="flex-1 flex items-center justify-center bg-[#0f0f0f]"
        style={containerStyle()}
        onMouseDown={handleMouseDown}
      >
        <Show
          when={props.selectedPhotoId}
          fallback={
            <div class="text-center text-[#333]">
              <div class="text-5xl mb-3">📷</div>
              <div class="text-sm">Select a photo in Library</div>
            </div>
          }
        >
          <Show
            when={previewUri()}
            fallback={
              <div class="flex flex-col items-center gap-3 text-[#444]">
                <div class="w-8 h-8 border-2 border-[#333] border-t-[#4a9eff] rounded-full animate-spin" />
                <span class="text-xs">Loading preview...</span>
              </div>
            }
          >
            {(uri) => (
              <img
                src={uri()}
                class={zoom() === 0 ? "" : "pointer-events-none"}
                style={imageStyle()}
              />
            )}
          </Show>
        </Show>
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
