import { createSignal, For, Show, createMemo } from "solid-js";
import { createStore } from "solid-js/store";

interface ShortcutBinding {
  action: string;
  description: string;
  key: string;
  category: string;
}

const DEFAULT_BINDINGS: ShortcutBinding[] = [
  // Navigation
  { action: "grid_view", description: "Switch to Grid View", key: "g", category: "Navigation" },
  { action: "loupe_view", description: "Switch to Loupe View", key: "e", category: "Navigation" },
  { action: "compare_view", description: "Switch to Compare View", key: "c", category: "Navigation" },
  { action: "develop_view", description: "Switch to Develop Module", key: "d", category: "Navigation" },
  { action: "next_photo", description: "Next Photo", key: "ArrowRight", category: "Navigation" },
  { action: "prev_photo", description: "Previous Photo", key: "ArrowLeft", category: "Navigation" },

  // Rating
  { action: "rate_0", description: "Set Rating 0", key: "0", category: "Rating" },
  { action: "rate_1", description: "Set Rating 1 Star", key: "1", category: "Rating" },
  { action: "rate_2", description: "Set Rating 2 Stars", key: "2", category: "Rating" },
  { action: "rate_3", description: "Set Rating 3 Stars", key: "3", category: "Rating" },
  { action: "rate_4", description: "Set Rating 4 Stars", key: "4", category: "Rating" },
  { action: "rate_5", description: "Set Rating 5 Stars", key: "5", category: "Rating" },

  // Flags
  { action: "flag_pick", description: "Flag as Pick", key: "p", category: "Rating" },
  { action: "flag_reject", description: "Flag as Rejected", key: "x", category: "Rating" },
  { action: "flag_none", description: "Remove Flag", key: "u", category: "Rating" },

  // Develop
  { action: "toggle_before_after", description: "Toggle Before/After", key: "\\", category: "Develop" },
  { action: "reset_all", description: "Reset All Adjustments", key: "cmd+shift+r", category: "Develop" },
  { action: "copy_settings", description: "Copy Settings", key: "cmd+c", category: "Develop" },
  { action: "paste_settings", description: "Paste Settings", key: "cmd+v", category: "Develop" },

  // Tools
  { action: "crop_tool", description: "Crop Tool", key: "r", category: "Develop" },
  { action: "brush_tool", description: "Adjustment Brush", key: "k", category: "Develop" },
  { action: "command_palette", description: "Command Palette", key: "cmd+k", category: "General" },
  { action: "preferences", description: "Preferences", key: "cmd+,", category: "General" },

  // Export
  { action: "export", description: "Export Photos", key: "cmd+shift+e", category: "Export" },
  { action: "quick_export", description: "Quick Export", key: "cmd+e", category: "Export" },
];

function loadBindings(): ShortcutBinding[] {
  const stored = localStorage.getItem("ocps_keybindings");
  if (stored) {
    try {
      return JSON.parse(stored);
    } catch {
      return DEFAULT_BINDINGS;
    }
  }
  return DEFAULT_BINDINGS;
}

function saveBindings(bindings: ShortcutBinding[]) {
  localStorage.setItem("ocps_keybindings", JSON.stringify(bindings));
}

export function ShortcutEditor() {
  const [bindings, setBindings] = createStore<ShortcutBinding[]>(loadBindings());
  const [editingAction, setEditingAction] = createSignal<string | null>(null);
  const [conflictWarning, setConflictWarning] = createSignal<string | null>(null);

  const groupedBindings = createMemo(() => {
    const groups: Record<string, ShortcutBinding[]> = {};
    bindings.forEach((binding) => {
      if (!groups[binding.category]) {
        groups[binding.category] = [];
      }
      groups[binding.category].push(binding);
    });
    return groups;
  });

  function formatKeyCombo(e: KeyboardEvent): string {
    const parts: string[] = [];

    if (e.metaKey || e.ctrlKey) parts.push("cmd");
    if (e.shiftKey) parts.push("shift");
    if (e.altKey) parts.push("alt");

    if (!["Meta", "Control", "Shift", "Alt"].includes(e.key)) {
      parts.push(e.key.toLowerCase());
    }

    return parts.join("+");
  }

  function findConflict(key: string, action: string): string | null {
    const existing = bindings.find((b) => b.key === key && b.action !== action);
    return existing ? existing.description : null;
  }

  function startEdit(action: string) {
    setEditingAction(action);
    setConflictWarning(null);

    const handler = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      const key = formatKeyCombo(e);

      // Check for conflicts
      const conflict = findConflict(key, action);
      if (conflict) {
        setConflictWarning(`Conflict: "${key}" is already used for "${conflict}"`);
        return;
      }

      // Update binding
      const index = bindings.findIndex((b) => b.action === action);
      if (index !== -1) {
        setBindings(index, "key", key);
        saveBindings([...bindings]);
      }

      setEditingAction(null);
      document.removeEventListener("keydown", handler);
    };

    document.addEventListener("keydown", handler);
  }

  function resetToDefault(action: string) {
    const defaultBinding = DEFAULT_BINDINGS.find((b) => b.action === action);
    if (defaultBinding) {
      const index = bindings.findIndex((b) => b.action === action);
      if (index !== -1) {
        setBindings(index, "key", defaultBinding.key);
        saveBindings([...bindings]);
      }
    }
  }

  function resetAll() {
    if (confirm("Reset all keyboard shortcuts to defaults?")) {
      setBindings([...DEFAULT_BINDINGS]);
      saveBindings(DEFAULT_BINDINGS);
    }
  }

  function exportKeymap() {
    const json = JSON.stringify(bindings, null, 2);
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "ocps-keymap.json";
    a.click();
    URL.revokeObjectURL(url);
  }

  function importKeymap() {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        const reader = new FileReader();
        reader.onload = (event) => {
          try {
            const imported = JSON.parse(event.target?.result as string);
            setBindings(imported);
            saveBindings(imported);
          } catch (err) {
            alert("Failed to import keymap: Invalid JSON");
          }
        };
        reader.readAsText(file);
      }
    };
    input.click();
  }

  return (
    <div class="p-6 bg-[#1a1a1a] text-[#ccc] overflow-y-auto h-full">
      <div class="flex justify-between items-center mb-6">
        <div>
          <h2 class="text-2xl font-light mb-1">Keyboard Shortcuts</h2>
          <p class="text-sm text-[#666]">Customize keyboard shortcuts for all actions</p>
        </div>
        <div class="flex gap-2">
          <button
            onClick={importKeymap}
            class="px-4 py-2 bg-[#2a2a2a] hover:bg-[#333] rounded text-sm"
          >
            Import
          </button>
          <button
            onClick={exportKeymap}
            class="px-4 py-2 bg-[#2a2a2a] hover:bg-[#333] rounded text-sm"
          >
            Export
          </button>
          <button
            onClick={resetAll}
            class="px-4 py-2 bg-[#2a2a2a] hover:bg-[#333] rounded text-sm"
          >
            Reset All
          </button>
        </div>
      </div>

      <Show when={conflictWarning()}>
        <div class="mb-4 p-3 bg-[#ff4444]/10 border border-[#ff4444]/30 rounded text-[#ff6666] text-sm">
          ⚠️ {conflictWarning()}
        </div>
      </Show>

      <For each={Object.entries(groupedBindings())}>
        {([category, categoryBindings]) => (
          <div class="mb-8">
            <h3 class="text-xs uppercase tracking-wider text-[#888] mb-3">{category}</h3>
            <div class="space-y-2">
              <For each={categoryBindings}>
                {(binding) => (
                  <div class="flex items-center justify-between p-3 bg-[#0f0f0f] rounded hover:bg-[#222]">
                    <div class="flex-1">
                      <div class="text-sm">{binding.description}</div>
                      <div class="text-xs text-[#555] mt-0.5">{binding.action}</div>
                    </div>
                    <div class="flex items-center gap-2">
                      <Show
                        when={editingAction() === binding.action}
                        fallback={
                          <div class="px-3 py-1 bg-[#2a2a2a] rounded font-mono text-xs min-w-[80px] text-center">
                            {binding.key}
                          </div>
                        }
                      >
                        <div class="px-3 py-1 bg-[#4a9eff] rounded font-mono text-xs min-w-[80px] text-center animate-pulse">
                          Press key...
                        </div>
                      </Show>
                      <button
                        onClick={() => startEdit(binding.action)}
                        class="px-3 py-1 bg-[#2a2a2a] hover:bg-[#333] rounded text-xs"
                        disabled={editingAction() !== null}
                      >
                        Edit
                      </button>
                      <button
                        onClick={() => resetToDefault(binding.action)}
                        class="px-3 py-1 bg-[#2a2a2a] hover:bg-[#333] rounded text-xs"
                      >
                        Reset
                      </button>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </div>
        )}
      </For>
    </div>
  );
}
