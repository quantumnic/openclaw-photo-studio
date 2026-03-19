import { createSignal, Show } from "solid-js";
import { usePreferences } from "../../lib/preferences";
import { invoke } from "@tauri-apps/api/core";

export interface PreferencesPanelProps {
  onClose: () => void;
}

export function PreferencesPanel(props: PreferencesPanelProps) {
  const { preferences, updatePreferences } = usePreferences();
  const [version, setVersion] = createSignal<any>(null);
  const [activeTab, setActiveTab] = createSignal<
    "general" | "performance" | "metadata" | "shortcuts" | "about"
  >("general");

  // Load version info
  invoke("get_version").then((v) => setVersion(v));

  return (
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
      <div class="bg-zinc-900 border border-zinc-700 rounded-lg shadow-2xl w-[800px] max-h-[80vh] overflow-hidden flex flex-col">
        <div class="p-6 border-b border-zinc-700">
          <h2 class="text-xl font-semibold text-white">Preferences</h2>
        </div>

        <div class="flex flex-1 overflow-hidden">
          {/* Sidebar */}
          <div class="w-48 border-r border-zinc-700 bg-zinc-950 p-4">
            <button
              class={`w-full text-left px-3 py-2 rounded mb-1 ${
                activeTab() === "general"
                  ? "bg-blue-600 text-white"
                  : "text-zinc-400 hover:bg-zinc-800"
              }`}
              onClick={() => setActiveTab("general")}
            >
              General
            </button>
            <button
              class={`w-full text-left px-3 py-2 rounded mb-1 ${
                activeTab() === "performance"
                  ? "bg-blue-600 text-white"
                  : "text-zinc-400 hover:bg-zinc-800"
              }`}
              onClick={() => setActiveTab("performance")}
            >
              Performance
            </button>
            <button
              class={`w-full text-left px-3 py-2 rounded mb-1 ${
                activeTab() === "metadata"
                  ? "bg-blue-600 text-white"
                  : "text-zinc-400 hover:bg-zinc-800"
              }`}
              onClick={() => setActiveTab("metadata")}
            >
              Metadata
            </button>
            <button
              class={`w-full text-left px-3 py-2 rounded mb-1 ${
                activeTab() === "shortcuts"
                  ? "bg-blue-600 text-white"
                  : "text-zinc-400 hover:bg-zinc-800"
              }`}
              onClick={() => setActiveTab("shortcuts")}
            >
              Shortcuts
            </button>
            <button
              class={`w-full text-left px-3 py-2 rounded mb-1 ${
                activeTab() === "about"
                  ? "bg-blue-600 text-white"
                  : "text-zinc-400 hover:bg-zinc-800"
              }`}
              onClick={() => setActiveTab("about")}
            >
              About
            </button>
          </div>

          {/* Content */}
          <div class="flex-1 p-6 overflow-y-auto">
            <Show when={activeTab() === "general"}>
              <div class="space-y-6">
                <h3 class="text-lg font-semibold text-white mb-4">General</h3>

                <div>
                  <label class="block text-sm font-medium text-zinc-300 mb-2">
                    App Language
                  </label>
                  <select
                    value={preferences.language}
                    onChange={(e) =>
                      updatePreferences({ language: e.currentTarget.value })
                    }
                    class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded text-white"
                  >
                    <option value="English">English</option>
                  </select>
                </div>

                <div>
                  <label class="block text-sm font-medium text-zinc-300 mb-2">
                    Date Format
                  </label>
                  <select
                    value={preferences.dateFormat}
                    onChange={(e) =>
                      updatePreferences({
                        dateFormat: e.currentTarget.value as any,
                      })
                    }
                    class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded text-white"
                  >
                    <option value="YYYY-MM-DD">YYYY-MM-DD</option>
                    <option value="MM-DD-YYYY">MM-DD-YYYY</option>
                    <option value="DD-MM-YYYY">DD-MM-YYYY</option>
                  </select>
                </div>

                <div>
                  <label class="flex items-center gap-2 text-sm font-medium text-zinc-300">
                    <input
                      type="checkbox"
                      checked={preferences.autoAdvanceAfterRating}
                      onChange={(e) =>
                        updatePreferences({
                          autoAdvanceAfterRating: e.currentTarget.checked,
                        })
                      }
                    />
                    Auto-advance after rating
                  </label>
                </div>
              </div>
            </Show>

            <Show when={activeTab() === "performance"}>
              <div class="space-y-6">
                <h3 class="text-lg font-semibold text-white mb-4">
                  Performance
                </h3>

                <div>
                  <label class="block text-sm font-medium text-zinc-300 mb-2">
                    Preview Cache Size
                  </label>
                  <select
                    value={preferences.previewCacheSize}
                    onChange={(e) =>
                      updatePreferences({
                        previewCacheSize: e.currentTarget.value as any,
                      })
                    }
                    class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded text-white"
                  >
                    <option value="512MB">512 MB</option>
                    <option value="1GB">1 GB</option>
                    <option value="2GB">2 GB</option>
                    <option value="4GB">4 GB</option>
                  </select>
                </div>

                <div>
                  <label class="block text-sm font-medium text-zinc-300 mb-2">
                    Background Threads
                  </label>
                  <select
                    value={
                      preferences.backgroundThreads === "auto"
                        ? "auto"
                        : String(preferences.backgroundThreads)
                    }
                    onChange={(e) =>
                      updatePreferences({
                        backgroundThreads:
                          e.currentTarget.value === "auto"
                            ? "auto"
                            : parseInt(e.currentTarget.value),
                      })
                    }
                    class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded text-white"
                  >
                    <option value="auto">Auto</option>
                    <option value="1">1</option>
                    <option value="2">2</option>
                    <option value="4">4</option>
                  </select>
                </div>

                <div>
                  <label class="flex items-center gap-2 text-sm font-medium text-zinc-300">
                    <input
                      type="checkbox"
                      checked={preferences.gpuAcceleration}
                      disabled
                      onChange={(e) =>
                        updatePreferences({
                          gpuAcceleration: e.currentTarget.checked,
                        })
                      }
                    />
                    GPU Acceleration (Coming in Phase 3)
                  </label>
                  <p class="text-xs text-zinc-500 mt-1 ml-6">
                    GPU rendering will be available in a future update
                  </p>
                </div>
              </div>
            </Show>

            <Show when={activeTab() === "metadata"}>
              <div class="space-y-6">
                <h3 class="text-lg font-semibold text-white mb-4">
                  Metadata & Sidecars
                </h3>

                <div>
                  <label class="block text-sm font-medium text-zinc-300 mb-2">
                    XMP Sidecar Mode
                  </label>
                  <select
                    value={preferences.xmpSidecarMode}
                    onChange={(e) =>
                      updatePreferences({
                        xmpSidecarMode: e.currentTarget.value as any,
                      })
                    }
                    class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded text-white"
                  >
                    <option value="auto">Auto (write on change)</option>
                    <option value="manual">Manual</option>
                    <option value="read-only">Read-Only</option>
                    <option value="disabled">Disabled</option>
                  </select>
                </div>

                <div>
                  <label class="block text-sm font-medium text-zinc-300 mb-2">
                    Default Copyright
                  </label>
                  <input
                    type="text"
                    value={preferences.defaultCopyright}
                    onInput={(e) =>
                      updatePreferences({
                        defaultCopyright: e.currentTarget.value,
                      })
                    }
                    class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded text-white"
                    placeholder="© 2026 Your Name"
                  />
                </div>

                <div>
                  <label class="block text-sm font-medium text-zinc-300 mb-2">
                    Default Creator
                  </label>
                  <input
                    type="text"
                    value={preferences.defaultCreator}
                    onInput={(e) =>
                      updatePreferences({
                        defaultCreator: e.currentTarget.value,
                      })
                    }
                    class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded text-white"
                    placeholder="Your Name"
                  />
                </div>
              </div>
            </Show>

            <Show when={activeTab() === "shortcuts"}>
              <div class="space-y-6">
                <h3 class="text-lg font-semibold text-white mb-4">
                  Keyboard Shortcuts
                </h3>

                <div class="bg-zinc-800 rounded p-4 border border-zinc-700">
                  <p class="text-zinc-300 mb-4">
                    Keyboard shortcuts are based on Adobe Lightroom's layout
                  </p>
                  <div class="space-y-2 text-sm">
                    <div class="flex justify-between">
                      <span class="text-zinc-400">Rate 1-5</span>
                      <span class="text-zinc-300 font-mono">1-5</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="text-zinc-400">Flag as Pick</span>
                      <span class="text-zinc-300 font-mono">P</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="text-zinc-400">Flag as Reject</span>
                      <span class="text-zinc-300 font-mono">X</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="text-zinc-400">Before/After</span>
                      <span class="text-zinc-300 font-mono">\</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="text-zinc-400">Copy Settings</span>
                      <span class="text-zinc-300 font-mono">Cmd+C</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="text-zinc-400">Paste Settings</span>
                      <span class="text-zinc-300 font-mono">Cmd+V</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="text-zinc-400">Batch Export</span>
                      <span class="text-zinc-300 font-mono">Cmd+Shift+E</span>
                    </div>
                  </div>
                </div>

                <p class="text-sm text-zinc-500">
                  Full shortcut customization will be available in Phase 5
                </p>
              </div>
            </Show>

            <Show when={activeTab() === "about"}>
              <div class="space-y-6">
                <h3 class="text-lg font-semibold text-white mb-4">
                  About OpenClaw Photo Studio
                </h3>

                <Show when={version()}>
                  <div class="bg-zinc-800 rounded p-4 border border-zinc-700">
                    <h4 class="text-white font-medium mb-2">Version</h4>
                    <div class="text-sm text-zinc-400 space-y-1">
                      <div>App: {version()?.app}</div>
                      <div>Core: {version()?.core}</div>
                      <div>Catalog: {version()?.catalog}</div>
                      <div>XMP: {version()?.xmp}</div>
                      <div>Export: {version()?.export}</div>
                    </div>
                  </div>
                </Show>

                <div class="bg-zinc-800 rounded p-4 border border-zinc-700">
                  <h4 class="text-white font-medium mb-2">License</h4>
                  <p class="text-sm text-zinc-400 mb-2">
                    PolyForm Noncommercial 1.0.0
                  </p>
                  <p class="text-xs text-zinc-500">
                    Free for personal use. Commercial use requires a separate
                    license.
                  </p>
                </div>

                <div class="bg-zinc-800 rounded p-4 border border-zinc-700">
                  <h4 class="text-white font-medium mb-2">Source Code</h4>
                  <p class="text-sm text-zinc-400">
                    Source available at{" "}
                    <a
                      href="https://github.com/quantumnic/openclaw-photo-studio"
                      target="_blank"
                      class="text-blue-400 hover:text-blue-300"
                    >
                      github.com/quantumnic/openclaw-photo-studio
                    </a>
                  </p>
                </div>

                <div class="text-xs text-zinc-600 text-center">
                  Made with 🌊 by Quantum Nic
                </div>
              </div>
            </Show>
          </div>
        </div>

        {/* Footer */}
        <div class="p-6 border-t border-zinc-700 flex justify-end">
          <button
            onClick={props.onClose}
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-white"
          >
            Done
          </button>
        </div>
      </div>
    </div>
  );
}
