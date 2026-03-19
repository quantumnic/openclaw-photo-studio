import { createSignal, onMount, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

export function TetheringPanel() {
  const [cameraStatus, setCameraStatus] = createSignal<any>(null);

  onMount(async () => {
    const status = await invoke("check_tethered_camera");
    setCameraStatus(status);
  });

  return (
    <div class="p-6 space-y-6">
      <div class="bg-blue-900/20 border border-blue-700 rounded-lg p-6">
        <h2 class="text-xl font-semibold text-white mb-4">
          Tethered Capture — Coming in Phase 7
        </h2>

        <p class="text-zinc-300 mb-4">
          Tethered shooting allows you to capture photos directly from your camera
          to OpenClaw Photo Studio in real-time.
        </p>

        <Show when={cameraStatus()}>
          <div class="bg-zinc-800 rounded p-4 mb-4">
            <p class="text-zinc-400">
              Status: {cameraStatus().message}
            </p>
          </div>
        </Show>

        <div class="space-y-3">
          <h3 class="text-lg font-medium text-white">Planned Features:</h3>
          <ul class="list-disc list-inside text-zinc-400 space-y-2">
            <li>Live View preview during capture</li>
            <li>Instant import as you shoot</li>
            <li>Remote camera control (aperture, shutter, ISO)</li>
            <li>Auto-advance to next shot</li>
            <li>Backup to multiple locations</li>
          </ul>
        </div>

        <div class="mt-6 space-y-3">
          <h3 class="text-lg font-medium text-white">Supported Cameras:</h3>
          <div class="grid grid-cols-2 gap-4">
            <div class="bg-zinc-800 rounded p-3">
              <h4 class="text-white font-medium mb-2">Canon</h4>
              <p class="text-sm text-zinc-400">
                EOS R series, 5D/6D series, and more
              </p>
            </div>
            <div class="bg-zinc-800 rounded p-3">
              <h4 class="text-white font-medium mb-2">Nikon</h4>
              <p class="text-sm text-zinc-400">
                Z series, D850, D780, and more
              </p>
            </div>
            <div class="bg-zinc-800 rounded p-3">
              <h4 class="text-white font-medium mb-2">Sony</h4>
              <p class="text-sm text-zinc-400">
                A7 series, A9 series, and more
              </p>
            </div>
            <div class="bg-zinc-800 rounded p-3">
              <h4 class="text-white font-medium mb-2">Fujifilm</h4>
              <p class="text-sm text-zinc-400">
                X-T series, GFX series, and more
              </p>
            </div>
          </div>
        </div>

        <div class="mt-6">
          <a
            href="https://github.com/quantumnic/openclaw-photo-studio/issues"
            target="_blank"
            class="inline-block px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-white"
          >
            Track Progress on GitHub
          </a>
        </div>
      </div>
    </div>
  );
}
