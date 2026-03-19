import { createSignal, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export interface BatchExportSettings {
  format: "jpeg" | "png" | "tiff";
  quality: number;
  resizeLongEdge?: number;
  outputFolder: string;
  namingTemplate: string;
  includeMetadata: boolean;
}

export interface BatchExportDialogProps {
  selectedPhotoIds: string[];
  onClose: () => void;
}

export function BatchExportDialog(props: BatchExportDialogProps) {
  const [format, setFormat] = createSignal<"jpeg" | "png" | "tiff">("jpeg");
  const [quality, setQuality] = createSignal(85);
  const [resizeEnabled, setResizeEnabled] = createSignal(false);
  const [resizeLongEdge, setResizeLongEdge] = createSignal(2048);
  const [outputFolder, setOutputFolder] = createSignal("");
  const [namingTemplate, setNamingTemplate] = createSignal("{original}");
  const [includeMetadata, setIncludeMetadata] = createSignal(true);
  const [isExporting, setIsExporting] = createSignal(false);
  const [progress, setProgress] = createSignal(0);
  const [exportResult, setExportResult] = createSignal<any>(null);
  const [errors, setErrors] = createSignal<string[]>([]);

  const selectOutputFolder = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (selected && typeof selected === "string") {
      setOutputFolder(selected);
    }
  };

  const startExport = async () => {
    if (!outputFolder()) {
      alert("Please select an output folder");
      return;
    }

    setIsExporting(true);
    setProgress(0);
    setErrors([]);

    const startTime = Date.now();

    try {
      const result = await invoke("export_photos_batch", {
        photoIds: props.selectedPhotoIds,
        outputFolder: outputFolder(),
        format: format(),
        quality: quality(),
        resizeLongEdge: resizeEnabled() ? resizeLongEdge() : undefined,
        namingTemplate: namingTemplate(),
      });

      const duration = (Date.now() - startTime) / 1000;
      setExportResult({ ...result, duration });
      setProgress(100);

      // Extract errors if any
      if (result.errors && result.errors.length > 0) {
        setErrors(result.errors);
      }
    } catch (error) {
      console.error("Export failed:", error);
      setErrors([String(error)]);
    } finally {
      setIsExporting(false);
    }
  };

  const namingPreview = () => {
    const template = namingTemplate();
    if (template === "{original}") return "DSC_4523.jpg";
    if (template === "{date}_{original}") return "2024-03-19_DSC_4523.jpg";
    if (template === "{seq}_{original}") return "001_DSC_4523.jpg";
    return template;
  };

  return (
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
      <div class="bg-zinc-900 border border-zinc-700 rounded-lg shadow-2xl w-[600px] max-h-[80vh] overflow-y-auto">
        <div class="p-6 border-b border-zinc-700">
          <h2 class="text-xl font-semibold text-white">
            Batch Export — {props.selectedPhotoIds.length} photos
          </h2>
        </div>

        <div class="p-6 space-y-6">
          {/* Format selector */}
          <div>
            <label class="block text-sm font-medium text-zinc-300 mb-2">
              Format
            </label>
            <div class="flex gap-2">
              <button
                class={`px-4 py-2 rounded ${
                  format() === "jpeg"
                    ? "bg-blue-600 text-white"
                    : "bg-zinc-800 text-zinc-400"
                }`}
                onClick={() => setFormat("jpeg")}
              >
                JPEG
              </button>
              <button
                class={`px-4 py-2 rounded ${
                  format() === "png"
                    ? "bg-blue-600 text-white"
                    : "bg-zinc-800 text-zinc-400"
                }`}
                onClick={() => setFormat("png")}
              >
                PNG
              </button>
              <button
                class={`px-4 py-2 rounded ${
                  format() === "tiff"
                    ? "bg-blue-600 text-white"
                    : "bg-zinc-800 text-zinc-400"
                }`}
                onClick={() => setFormat("tiff")}
              >
                TIFF
              </button>
            </div>
          </div>

          {/* Quality slider (JPEG only) */}
          <Show when={format() === "jpeg"}>
            <div>
              <label class="block text-sm font-medium text-zinc-300 mb-2">
                Quality: {quality()}
              </label>
              <input
                type="range"
                min="60"
                max="100"
                value={quality()}
                onInput={(e) => setQuality(parseInt(e.currentTarget.value))}
                class="w-full"
              />
            </div>
          </Show>

          {/* Resize option */}
          <div>
            <label class="flex items-center gap-2 text-sm font-medium text-zinc-300 mb-2">
              <input
                type="checkbox"
                checked={resizeEnabled()}
                onChange={(e) => setResizeEnabled(e.currentTarget.checked)}
              />
              Resize long edge
            </label>
            <Show when={resizeEnabled()}>
              <input
                type="number"
                value={resizeLongEdge()}
                onInput={(e) => setResizeLongEdge(parseInt(e.currentTarget.value))}
                class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded text-white"
                placeholder="2048"
              />
            </Show>
          </div>

          {/* Output folder */}
          <div>
            <label class="block text-sm font-medium text-zinc-300 mb-2">
              Output Folder
            </label>
            <div class="flex gap-2">
              <input
                type="text"
                value={outputFolder()}
                readOnly
                class="flex-1 px-3 py-2 bg-zinc-800 border border-zinc-700 rounded text-white"
                placeholder="Select output folder..."
              />
              <button
                onClick={selectOutputFolder}
                class="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded text-white"
              >
                Browse
              </button>
            </div>
          </div>

          {/* Naming template */}
          <div>
            <label class="block text-sm font-medium text-zinc-300 mb-2">
              Naming Template
            </label>
            <select
              value={namingTemplate()}
              onChange={(e) => setNamingTemplate(e.currentTarget.value)}
              class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded text-white"
            >
              <option value="{original}">Original filename</option>
              <option value="{date}_{original}">Date + original</option>
              <option value="{seq}_{original}">Sequence + original</option>
            </select>
            <p class="text-sm text-zinc-500 mt-1">Preview: {namingPreview()}</p>
          </div>

          {/* Include metadata */}
          <div>
            <label class="flex items-center gap-2 text-sm font-medium text-zinc-300">
              <input
                type="checkbox"
                checked={includeMetadata()}
                onChange={(e) => setIncludeMetadata(e.currentTarget.checked)}
              />
              Include metadata (EXIF)
            </label>
          </div>

          {/* Progress bar */}
          <Show when={isExporting()}>
            <div>
              <div class="w-full bg-zinc-800 rounded-full h-2">
                <div
                  class="bg-blue-600 h-2 rounded-full transition-all"
                  style={{ width: `${progress()}%` }}
                />
              </div>
              <p class="text-sm text-zinc-400 mt-2">Exporting...</p>
            </div>
          </Show>

          {/* Export result */}
          <Show when={exportResult()}>
            <div class="bg-green-900/20 border border-green-700 rounded p-4">
              <p class="text-green-400 font-medium">
                Exported {exportResult().succeeded}/{exportResult().total} photos
                in {exportResult().duration.toFixed(1)}s
              </p>
              <Show when={exportResult().failed > 0}>
                <p class="text-yellow-400 mt-1">
                  {exportResult().failed} failed
                </p>
              </Show>
            </div>
          </Show>

          {/* Errors */}
          <Show when={errors().length > 0}>
            <div class="bg-red-900/20 border border-red-700 rounded p-4 max-h-32 overflow-y-auto">
              <p class="text-red-400 font-medium mb-2">Errors:</p>
              <For each={errors()}>
                {(error) => <p class="text-sm text-red-300">{error}</p>}
              </For>
            </div>
          </Show>
        </div>

        {/* Actions */}
        <div class="p-6 border-t border-zinc-700 flex justify-end gap-3">
          <button
            onClick={props.onClose}
            class="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded text-white"
            disabled={isExporting()}
          >
            {exportResult() ? "Close" : "Cancel"}
          </button>
          <Show when={!exportResult()}>
            <button
              onClick={startExport}
              class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-white"
              disabled={isExporting() || !outputFolder()}
            >
              {isExporting() ? "Exporting..." : "Export"}
            </button>
          </Show>
        </div>
      </div>
    </div>
  );
}
