import { Component, createSignal, For } from "solid-js";
import { DEFAULT_BINDINGS, ShortcutBinding } from "../../lib/shortcuts";

interface Props {
  onClose: () => void;
}

interface ShortcutSection {
  title: string;
  shortcuts: ShortcutBinding[];
}

const ShortcutReference: Component<Props> = (props) => {
  const [searchQuery, setSearchQuery] = createSignal("");

  // Organize shortcuts into sections
  const sections: ShortcutSection[] = [
    {
      title: "Navigation",
      shortcuts: DEFAULT_BINDINGS.filter(
        (b) =>
          b.action.startsWith("view.") ||
          b.action.startsWith("photo.") ||
          b.action === "module.develop"
      ),
    },
    {
      title: "Rating & Flagging",
      shortcuts: DEFAULT_BINDINGS.filter(
        (b) => b.action.startsWith("rate.") || b.action.startsWith("flag.")
      ),
    },
    {
      title: "Develop",
      shortcuts: DEFAULT_BINDINGS.filter((b) => b.action.startsWith("develop.")),
    },
    {
      title: "Export & Import",
      shortcuts: DEFAULT_BINDINGS.filter(
        (b) => b.action.startsWith("export.") || b.action.startsWith("import.")
      ),
    },
    {
      title: "UI",
      shortcuts: DEFAULT_BINDINGS.filter((b) => b.action.startsWith("ui.")),
    },
  ];

  // Filter shortcuts based on search query
  const filteredSections = () => {
    const query = searchQuery().toLowerCase();
    if (!query) return sections;

    return sections
      .map((section) => ({
        ...section,
        shortcuts: section.shortcuts.filter(
          (s) =>
            s.description.toLowerCase().includes(query) ||
            s.key.toLowerCase().includes(query) ||
            s.action.toLowerCase().includes(query)
        ),
      }))
      .filter((section) => section.shortcuts.length > 0);
  };

  // Format key for display (e.g., "cmd+c" -> ["Cmd", "C"])
  const formatKey = (key: string): string[] => {
    return key.split("+").map((part) => {
      if (part === "cmd") return "⌘";
      if (part === "shift") return "⇧";
      if (part === "alt") return "⌥";
      if (part === "ctrl") return "⌃";
      if (part === "ArrowLeft") return "←";
      if (part === "ArrowRight") return "→";
      if (part === "ArrowUp") return "↑";
      if (part === "ArrowDown") return "↓";
      if (part === "Tab") return "⇥";
      return part.toUpperCase();
    });
  };

  return (
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div class="bg-gray-900 border border-gray-700 rounded-lg shadow-xl max-w-4xl w-full max-h-[80vh] flex flex-col">
        {/* Header */}
        <div class="border-b border-gray-700 px-6 py-4 flex items-center justify-between">
          <h2 class="text-xl font-semibold text-white">Keyboard Shortcuts</h2>
          <button
            onClick={props.onClose}
            class="text-gray-400 hover:text-white transition-colors"
            aria-label="Close"
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

        {/* Search */}
        <div class="px-6 py-3 border-b border-gray-700">
          <input
            type="text"
            placeholder="Search shortcuts..."
            value={searchQuery()}
            onInput={(e) => setSearchQuery(e.currentTarget.value)}
            class="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded-md text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            autofocus
          />
        </div>

        {/* Shortcuts List */}
        <div class="flex-1 overflow-y-auto px-6 py-4">
          <For each={filteredSections()}>
            {(section) => (
              <div class="mb-6 last:mb-0">
                <h3 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3">
                  {section.title}
                </h3>
                <div class="space-y-2">
                  <For each={section.shortcuts}>
                    {(shortcut) => (
                      <div class="flex items-center justify-between py-2 hover:bg-gray-800 rounded px-2 -mx-2 transition-colors">
                        <span class="text-gray-200 text-sm">
                          {shortcut.description}
                        </span>
                        <div class="flex gap-1">
                          <For each={formatKey(shortcut.key)}>
                            {(keyPart) => (
                              <kbd class="px-2 py-1 bg-gray-700 text-gray-200 text-xs font-mono rounded border border-gray-600 shadow-sm min-w-[1.5rem] text-center">
                                {keyPart}
                              </kbd>
                            )}
                          </For>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
              </div>
            )}
          </For>

          {filteredSections().length === 0 && (
            <div class="text-center py-12 text-gray-400">
              No shortcuts found matching "{searchQuery()}"
            </div>
          )}
        </div>

        {/* Footer */}
        <div class="border-t border-gray-700 px-6 py-3 bg-gray-800/50">
          <p class="text-xs text-gray-400">
            Press <kbd class="px-1 py-0.5 bg-gray-700 rounded text-gray-300">?</kbd> to toggle this panel
          </p>
        </div>
      </div>
    </div>
  );
};

export default ShortcutReference;
