import { createSignal, onMount, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface DiagnosticsData {
  version: string;
  photoCount: number;
  pendingJobs: number;
  supportedFormats: string[];
  pluginCount: number;
  cacheSize: string;
  gpuInfo: string;
}

interface DiagnosticsViewProps {
  onClose: () => void;
}

export function DiagnosticsView(props: DiagnosticsViewProps) {
  const [data, setData] = createSignal<DiagnosticsData | null>(null);
  const [loading, setLoading] = createSignal(true);

  onMount(async () => {
    try {
      const [version, stats, formats, plugins] = await Promise.all([
        invoke<string>("get_version"),
        invoke<{ total: number }>("get_catalog_stats"),
        invoke<string[]>("get_supported_formats"),
        invoke<any[]>("get_plugins"),
      ]);

      setData({
        version,
        photoCount: stats.total,
        pendingJobs: 0,
        supportedFormats: formats,
        pluginCount: plugins.length,
        cacheSize: "N/A (Phase 3)",
        gpuInfo: "N/A (Phase 3)",
      });
    } catch (error) {
      console.error("Failed to load diagnostics:", error);
    } finally {
      setLoading(false);
    }
  });

  return (
    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-2xl mx-4">
        {/* Header */}
        <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
            Diagnostics
          </h2>
          <button
            onClick={props.onClose}
            class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
          >
            <svg
              class="w-6 h-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>

        {/* Content */}
        <div class="p-6 max-h-[70vh] overflow-auto">
          <Show when={!loading()} fallback={<div class="text-center py-8 text-gray-500">Loading...</div>}>
            <Show when={data()}>
              {(diagnostics) => (
                <div class="space-y-4">
                  {/* App Version */}
                  <div class="flex justify-between py-2 border-b border-gray-200 dark:border-gray-700">
                    <span class="font-medium text-gray-700 dark:text-gray-300">
                      App Version
                    </span>
                    <span class="text-gray-900 dark:text-white">
                      {diagnostics().version}
                    </span>
                  </div>

                  {/* Photo Count */}
                  <div class="flex justify-between py-2 border-b border-gray-200 dark:border-gray-700">
                    <span class="font-medium text-gray-700 dark:text-gray-300">
                      Photos in Catalog
                    </span>
                    <span class="text-gray-900 dark:text-white">
                      {diagnostics().photoCount.toLocaleString()}
                    </span>
                  </div>

                  {/* Pending Jobs */}
                  <div class="flex justify-between py-2 border-b border-gray-200 dark:border-gray-700">
                    <span class="font-medium text-gray-700 dark:text-gray-300">
                      Pending Jobs
                    </span>
                    <span class="text-gray-900 dark:text-white">
                      {diagnostics().pendingJobs}
                    </span>
                  </div>

                  {/* Plugin Count */}
                  <div class="flex justify-between py-2 border-b border-gray-200 dark:border-gray-700">
                    <span class="font-medium text-gray-700 dark:text-gray-300">
                      Installed Plugins
                    </span>
                    <span class="text-gray-900 dark:text-white">
                      {diagnostics().pluginCount}
                    </span>
                  </div>

                  {/* RAW Formats */}
                  <div class="py-2 border-b border-gray-200 dark:border-gray-700">
                    <div class="font-medium text-gray-700 dark:text-gray-300 mb-2">
                      Supported RAW Formats
                    </div>
                    <div class="flex flex-wrap gap-2">
                      {diagnostics().supportedFormats.map((format) => (
                        <span class="px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded text-sm">
                          {format.toUpperCase()}
                        </span>
                      ))}
                    </div>
                  </div>

                  {/* Cache Size */}
                  <div class="flex justify-between py-2 border-b border-gray-200 dark:border-gray-700">
                    <span class="font-medium text-gray-700 dark:text-gray-300">
                      Cache Size
                    </span>
                    <span class="text-gray-500 dark:text-gray-400 italic">
                      {diagnostics().cacheSize}
                    </span>
                  </div>

                  {/* GPU Info */}
                  <div class="flex justify-between py-2">
                    <span class="font-medium text-gray-700 dark:text-gray-300">
                      GPU Info
                    </span>
                    <span class="text-gray-500 dark:text-gray-400 italic">
                      {diagnostics().gpuInfo}
                    </span>
                  </div>
                </div>
              )}
            </Show>
          </Show>
        </div>

        {/* Footer */}
        <div class="p-4 border-t border-gray-200 dark:border-gray-700 flex justify-end">
          <button
            onClick={props.onClose}
            class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
