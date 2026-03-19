import { createSignal, For, createStore } from "solid-js";

interface PrintLayout {
  paperSize: "A4" | "A3" | "Letter" | "4x6" | "5x7";
  orientation: "portrait" | "landscape";
  columns: number;
  rows: number;
  margin: number;
  showFilename: boolean;
}

interface PrintViewProps {
  selectedPhotoIds: string[];
}

export function PrintView(props: PrintViewProps) {
  const [layout, setLayout] = createStore<PrintLayout>({
    paperSize: "A4",
    orientation: "portrait",
    columns: 3,
    rows: 4,
    margin: 10,
    showFilename: false,
  });

  const handlePrint = () => {
    window.print();
  };

  const handleContactSheet = () => {
    // Future: export as single JPEG grid
    console.log("Contact sheet export - coming soon");
  };

  return (
    <div class="flex h-full">
      {/* Preview Area */}
      <div class="flex-1 overflow-auto bg-gray-100 dark:bg-gray-900 p-8">
        <div
          class="bg-white dark:bg-gray-800 shadow-lg mx-auto"
          style={{
            width: layout.orientation === "portrait" ? "210mm" : "297mm",
            height: layout.orientation === "portrait" ? "297mm" : "210mm",
            padding: `${layout.margin}mm`,
          }}
        >
          <div
            class="grid h-full gap-2"
            style={{
              "grid-template-columns": `repeat(${layout.columns}, 1fr)`,
              "grid-template-rows": `repeat(${layout.rows}, 1fr)`,
            }}
          >
            <For each={Array.from({ length: layout.columns * layout.rows })}>
              {(_, index) => {
                const photoIndex = index();
                const hasPhoto = photoIndex < props.selectedPhotoIds.length;
                return (
                  <div class="border border-gray-300 dark:border-gray-600 flex flex-col items-center justify-center bg-gray-50 dark:bg-gray-700">
                    {hasPhoto ? (
                      <>
                        <div class="flex-1 w-full bg-gray-200 dark:bg-gray-600" />
                        {layout.showFilename && (
                          <div class="text-xs text-center py-1 text-gray-600 dark:text-gray-300">
                            Photo {photoIndex + 1}
                          </div>
                        )}
                      </>
                    ) : (
                      <div class="text-gray-400 text-sm">Empty</div>
                    )}
                  </div>
                );
              }}
            </For>
          </div>
        </div>
      </div>

      {/* Controls Panel */}
      <div class="w-80 bg-white dark:bg-gray-800 border-l border-gray-200 dark:border-gray-700 p-4 overflow-auto">
        <h2 class="text-xl font-semibold mb-4 text-gray-900 dark:text-white">
          Print Settings
        </h2>

        <div class="space-y-4">
          {/* Paper Size */}
          <div>
            <label class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
              Paper Size
            </label>
            <select
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              value={layout.paperSize}
              onChange={(e) =>
                setLayout(
                  "paperSize",
                  e.currentTarget.value as PrintLayout["paperSize"]
                )
              }
            >
              <option value="A4">A4 (210 × 297 mm)</option>
              <option value="A3">A3 (297 × 420 mm)</option>
              <option value="Letter">Letter (8.5 × 11 in)</option>
              <option value="4x6">4×6 in</option>
              <option value="5x7">5×7 in</option>
            </select>
          </div>

          {/* Orientation */}
          <div>
            <label class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
              Orientation
            </label>
            <div class="flex gap-2">
              <button
                class={`flex-1 px-4 py-2 rounded ${
                  layout.orientation === "portrait"
                    ? "bg-blue-600 text-white"
                    : "bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white"
                }`}
                onClick={() => setLayout("orientation", "portrait")}
              >
                Portrait
              </button>
              <button
                class={`flex-1 px-4 py-2 rounded ${
                  layout.orientation === "landscape"
                    ? "bg-blue-600 text-white"
                    : "bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white"
                }`}
                onClick={() => setLayout("orientation", "landscape")}
              >
                Landscape
              </button>
            </div>
          </div>

          {/* Columns */}
          <div>
            <label class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
              Columns: {layout.columns}
            </label>
            <input
              type="range"
              min="1"
              max="6"
              value={layout.columns}
              class="w-full"
              onInput={(e) =>
                setLayout("columns", parseInt(e.currentTarget.value))
              }
            />
          </div>

          {/* Rows */}
          <div>
            <label class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
              Rows: {layout.rows}
            </label>
            <input
              type="range"
              min="1"
              max="6"
              value={layout.rows}
              class="w-full"
              onInput={(e) =>
                setLayout("rows", parseInt(e.currentTarget.value))
              }
            />
          </div>

          {/* Margin */}
          <div>
            <label class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
              Margin: {layout.margin}mm
            </label>
            <input
              type="range"
              min="0"
              max="50"
              value={layout.margin}
              class="w-full"
              onInput={(e) =>
                setLayout("margin", parseInt(e.currentTarget.value))
              }
            />
          </div>

          {/* Show Filename */}
          <div class="flex items-center gap-2">
            <input
              type="checkbox"
              id="show-filename"
              checked={layout.showFilename}
              onChange={(e) => setLayout("showFilename", e.currentTarget.checked)}
              class="w-4 h-4"
            />
            <label
              for="show-filename"
              class="text-sm font-medium text-gray-700 dark:text-gray-300"
            >
              Show filename
            </label>
          </div>

          {/* Actions */}
          <div class="space-y-2 pt-4 border-t border-gray-200 dark:border-gray-700">
            <button
              onClick={handlePrint}
              class="w-full px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
            >
              Print
            </button>
            <button
              onClick={handleContactSheet}
              class="w-full px-4 py-2 bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white rounded hover:bg-gray-300 dark:hover:bg-gray-600"
            >
              Export Contact Sheet (Coming Soon)
            </button>
          </div>

          {/* Info */}
          <div class="text-xs text-gray-500 dark:text-gray-400 pt-4">
            <p>{props.selectedPhotoIds.length} photo(s) selected</p>
            <p>
              Grid capacity: {layout.columns * layout.rows} photos
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
