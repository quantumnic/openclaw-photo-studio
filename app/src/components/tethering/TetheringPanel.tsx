import { createSignal, onMount, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface Camera {
  id: string;
  name: string;
  provider: string;
  connected: boolean;
}

export function TetheringPanel() {
  const [cameras, setCameras] = createSignal<Camera[]>([]);
  const [connectedCamera, setConnectedCamera] = createSignal<Camera | null>(null);
  const [shotCount, setShotCount] = createSignal(0);
  const [message, setMessage] = createSignal("");

  const discoverCameras = async () => {
    try {
      const discovered = (await invoke("discover_cameras")) as Camera[];
      setCameras(discovered);
      setMessage(`Found ${discovered.length} camera(s)`);
    } catch (err) {
      setMessage(`Error: ${err}`);
    }
  };

  const connectCamera = async (cameraId: string) => {
    try {
      await invoke("connect_camera", { cameraId });
      const camera = cameras().find((c) => c.id === cameraId);
      if (camera) {
        setConnectedCamera({ ...camera, connected: true });
        setMessage(`Connected to ${camera.name}`);
      }
    } catch (err) {
      setMessage(`Connection failed: ${err}`);
    }
  };

  const disconnectCamera = async () => {
    try {
      await invoke("disconnect_camera");
      setConnectedCamera(null);
      setMessage("Disconnected");
    } catch (err) {
      setMessage(`Disconnect failed: ${err}`);
    }
  };

  const capture = async () => {
    try {
      const result = (await invoke("tether_capture")) as any;
      setShotCount((prev) => prev + 1);
      setMessage(result.message || "Captured!");
    } catch (err) {
      setMessage(`Capture failed: ${err}`);
    }
  };

  onMount(async () => {
    await discoverCameras();
  });

  return (
    <div class="p-6 space-y-6">
      <div class="bg-blue-900/20 border border-blue-700 rounded-lg p-6">
        <h2 class="text-xl font-semibold text-white mb-4">
          Tethered Capture (v0.4.0 — Mock Provider)
        </h2>

        <p class="text-zinc-300 mb-4">
          Tethered shooting allows you to capture photos directly from your camera
          to OpenClaw Photo Studio in real-time.
        </p>

        <Show when={message()}>
          <div class="bg-zinc-800 rounded p-4 mb-4">
            <p class="text-zinc-400">{message()}</p>
          </div>
        </Show>

        <div class="space-y-4">
          <div class="flex gap-4">
            <button
              onClick={discoverCameras}
              class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-white"
            >
              Discover Cameras
            </button>

            <Show when={connectedCamera()}>
              <button
                onClick={disconnectCamera}
                class="px-4 py-2 bg-red-600 hover:bg-red-700 rounded text-white"
              >
                Disconnect
              </button>
            </Show>
          </div>

          <Show when={cameras().length > 0}>
            <div class="space-y-2">
              <h3 class="text-lg font-medium text-white">Available Cameras:</h3>
              <For each={cameras()}>
                {(camera) => (
                  <div class="bg-zinc-800 rounded p-4 flex justify-between items-center">
                    <div>
                      <p class="text-white font-medium">{camera.name}</p>
                      <p class="text-sm text-zinc-400">
                        Provider: {camera.provider} | ID: {camera.id}
                      </p>
                    </div>
                    <Show
                      when={!connectedCamera()}
                      fallback={
                        <Show when={connectedCamera()?.id === camera.id}>
                          <span class="text-green-400 font-medium">Connected</span>
                        </Show>
                      }
                    >
                      <button
                        onClick={() => connectCamera(camera.id)}
                        class="px-4 py-2 bg-green-600 hover:bg-green-700 rounded text-white"
                      >
                        Connect
                      </button>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </Show>

          <Show when={connectedCamera()}>
            <div class="space-y-4">
              <div class="bg-green-900/20 border border-green-700 rounded-lg p-4">
                <p class="text-green-400">
                  Connected to: {connectedCamera()?.name}
                </p>
                <p class="text-zinc-400 text-sm">Shots taken: {shotCount()}</p>
              </div>

              <button
                onClick={capture}
                class="px-6 py-3 bg-green-600 hover:bg-green-700 rounded text-white font-semibold text-lg"
              >
                📷 Capture
              </button>
            </div>
          </Show>
        </div>

        <div class="mt-6 space-y-3">
          <h3 class="text-lg font-medium text-white">Future Features:</h3>
          <ul class="list-disc list-inside text-zinc-400 space-y-2">
            <li>Live View preview during capture</li>
            <li>Instant import as you shoot</li>
            <li>Remote camera control (aperture, shutter, ISO)</li>
            <li>Auto-advance to next shot</li>
            <li>Real camera support (gphoto2, Canon/Nikon SDKs)</li>
          </ul>
        </div>
      </div>
    </div>
  );
}
