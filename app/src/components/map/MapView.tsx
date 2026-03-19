import { invoke } from "@tauri-apps/api/core";
import { createSignal, onMount, Show, For } from "solid-js";

interface GeoPhoto {
  id: string;
  file_name: string;
  lat: f64;
  lon: number;
  rating: number;
}

export function MapView() {
  const [geoPhotos, setGeoPhotos] = createSignal<GeoPhoto[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);

  // Load photos with GPS coordinates
  const loadGeoPhotos = async () => {
    try {
      setLoading(true);
      setError(null);
      const photos = await invoke<GeoPhoto[]>("get_geo_photos");
      setGeoPhotos(photos);
    } catch (err) {
      console.error("Failed to load geo photos:", err);
      setError(String(err));
      setGeoPhotos([]);
    } finally {
      setLoading(false);
    }
  };

  onMount(() => {
    loadGeoPhotos();
  });

  // Calculate center and bounds for map
  const calculateMapBounds = () => {
    const photos = geoPhotos();
    if (photos.length === 0) {
      return { lat: 46.8182, lon: 8.2275, zoom: 8 }; // Switzerland default
    }

    const lats = photos.map((p) => p.lat);
    const lons = photos.map((p) => p.lon);

    const minLat = Math.min(...lats);
    const maxLat = Math.max(...lats);
    const minLon = Math.min(...lons);
    const maxLon = Math.max(...lons);

    const centerLat = (minLat + maxLat) / 2;
    const centerLon = (minLon + maxLon) / 2;

    return {
      lat: centerLat,
      lon: centerLon,
      bbox: `${minLon},${minLat},${maxLon},${maxLat}`,
    };
  };

  const mapBounds = () => calculateMapBounds();

  return (
    <div class="h-full flex flex-col bg-[#141414]">
      {/* Toolbar */}
      <div class="h-10 bg-[#1c1c1c] border-b border-[#2a2a2a] flex items-center px-3 gap-3 text-xs text-[#aaa] flex-shrink-0">
        <div class="text-[#666] font-medium">🗺️ Map Module</div>
        <span class="text-[#333]">|</span>
        <button
          onClick={loadGeoPhotos}
          disabled={loading()}
          class="px-2 py-0.5 bg-[#2a2a2a] hover:bg-[#333] disabled:bg-[#1a1a1a] text-[#aaa] rounded text-xs transition-colors"
        >
          {loading() ? "Loading..." : "Refresh"}
        </button>
        <div class="ml-auto text-[#666]">
          {geoPhotos().length} photo{geoPhotos().length !== 1 ? "s" : ""} with GPS location
        </div>
      </div>

      {/* Error message */}
      <Show when={error()}>
        {(err) => (
          <div class="px-4 py-2 bg-[#2a1a1a] border-b border-[#3a2a2a] text-xs text-[#ff6b6b]">
            ✗ {err()}
          </div>
        )}
      </Show>

      {/* Main content */}
      <div class="flex-1 overflow-hidden flex flex-col">
        <Show
          when={geoPhotos().length > 0}
          fallback={
            <div class="h-full flex flex-col items-center justify-center text-[#444]">
              <div class="text-6xl mb-4">📍</div>
              <div class="text-[#666] font-medium mb-2">No photos with GPS data yet</div>
              <div class="text-xs text-[#444] mb-4">
                Import photos that contain GPS coordinates (EXIF)
              </div>
              <div class="text-xs text-[#333] space-y-1 text-center">
                <div>Photos taken with smartphones and GPS-enabled cameras</div>
                <div>automatically include location data</div>
              </div>
            </div>
          }
        >
          {/* Map Container */}
          <div class="flex-1 overflow-hidden p-4">
            <div class="h-full bg-[#1a1a1a] rounded border border-[#2a2a2a] overflow-hidden">
              {/* OpenStreetMap iframe embed */}
              <iframe
                src={`https://www.openstreetmap.org/export/embed.html?bbox=${
                  mapBounds().bbox || "7.0,46.0,9.0,48.0"
                }&layer=mapnik&marker=${mapBounds().lat},${mapBounds().lon}`}
                style="border: none; width: 100%; height: 100%;"
                title="OpenStreetMap"
              />
            </div>
          </div>

          {/* Photo List */}
          <div class="h-48 border-t border-[#2a2a2a] bg-[#1a1a1a] overflow-auto">
            <div class="px-4 py-2 bg-[#1c1c1c] border-b border-[#2a2a2a] text-xs text-[#666] font-medium">
              Photos in this region
            </div>
            <div class="p-2 space-y-1">
              <For each={geoPhotos()}>
                {(photo) => (
                  <div class="px-3 py-2 bg-[#1c1c1c] hover:bg-[#222] rounded text-xs text-[#aaa] flex items-center gap-3 cursor-pointer transition-colors">
                    <div class="text-lg">📷</div>
                    <div class="flex-1">
                      <div class="font-medium text-[#ccc]">{photo.file_name}</div>
                      <div class="text-[#666] text-[10px] font-mono">
                        {photo.lat.toFixed(5)}, {photo.lon.toFixed(5)}
                      </div>
                    </div>
                    <div class="flex gap-0.5">
                      <For each={Array.from({ length: 5 })}>
                        {(_, i) => (
                          <span
                            class={`text-[10px] ${
                              i() < photo.rating ? "text-[#e8b84b]" : "text-[#333]"
                            }`}
                          >
                            ★
                          </span>
                        )}
                      </For>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
}
