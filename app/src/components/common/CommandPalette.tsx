import { createSignal, For, onMount, Show, onCleanup } from "solid-js";
import { DEFAULT_BINDINGS, type ShortcutBinding } from "../../lib/shortcuts";

interface CommandPaletteProps {
  open: boolean;
  onClose: () => void;
  onExecute: (action: string) => void;
}

export function CommandPalette(props: CommandPaletteProps) {
  const [query, setQuery] = createSignal("");
  const [selectedIndex, setSelectedIndex] = createSignal(0);
  let inputRef: HTMLInputElement | undefined;

  const filtered = () => {
    const q = query().toLowerCase();
    if (!q) return DEFAULT_BINDINGS;

    return DEFAULT_BINDINGS.filter(
      (b) =>
        b.action.toLowerCase().includes(q) ||
        b.description.toLowerCase().includes(q) ||
        b.key.toLowerCase().includes(q)
    );
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      props.onClose();
      return;
    }

    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((i) => Math.min(i + 1, filtered().length - 1));
      return;
    }

    if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((i) => Math.max(i - 1, 0));
      return;
    }

    if (e.key === "Enter") {
      e.preventDefault();
      const selected = filtered()[selectedIndex()];
      if (selected) {
        props.onExecute(selected.action);
        props.onClose();
      }
      return;
    }
  };

  onMount(() => {
    if (props.open) {
      inputRef?.focus();
      document.addEventListener("keydown", handleKeyDown);
      onCleanup(() => document.removeEventListener("keydown", handleKeyDown));
    }
  });

  return (
    <Show when={props.open}>
      <div
        class="fixed inset-0 bg-black/60 flex items-start justify-center pt-32 z-50"
        onClick={props.onClose}
      >
        <div
          class="bg-[#1a1a1a] border border-[#333] rounded-lg shadow-2xl w-[600px] max-h-[500px] flex flex-col"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Search Input */}
          <div class="p-4 border-b border-[#2a2a2a]">
            <input
              ref={inputRef}
              type="text"
              value={query()}
              onInput={(e) => {
                setQuery(e.currentTarget.value);
                setSelectedIndex(0);
              }}
              placeholder="Search commands..."
              class="w-full bg-[#111] border border-[#333] rounded px-3 py-2 text-sm text-[#ddd] placeholder-[#555] focus:outline-none focus:ring-1 focus:ring-[#4a9eff]"
            />
          </div>

          {/* Command List */}
          <div class="overflow-y-auto flex-1 p-2">
            <Show
              when={filtered().length > 0}
              fallback={
                <div class="text-center py-8 text-[#555] text-sm">
                  No commands found
                </div>
              }
            >
              <For each={filtered()}>
                {(binding, index) => (
                  <div
                    class={`px-3 py-2 rounded cursor-pointer flex items-center justify-between ${
                      index() === selectedIndex()
                        ? "bg-[#4a9eff] text-white"
                        : "hover:bg-[#252525] text-[#ddd]"
                    }`}
                    onClick={() => {
                      props.onExecute(binding.action);
                      props.onClose();
                    }}
                  >
                    <div class="flex-1">
                      <div class="text-sm font-medium">{binding.description}</div>
                      <div
                        class={`text-xs ${
                          index() === selectedIndex() ? "text-white/70" : "text-[#666]"
                        }`}
                      >
                        {binding.action}
                      </div>
                    </div>
                    <div
                      class={`text-xs font-mono px-2 py-1 rounded ${
                        index() === selectedIndex()
                          ? "bg-white/20"
                          : "bg-[#2a2a2a] text-[#777]"
                      }`}
                    >
                      {binding.key}
                    </div>
                  </div>
                )}
              </For>
            </Show>
          </div>

          {/* Footer */}
          <div class="px-4 py-2 border-t border-[#2a2a2a] flex items-center gap-3 text-xs text-[#666]">
            <span>↑↓ Navigate</span>
            <span>Enter Execute</span>
            <span>Esc Close</span>
          </div>
        </div>
      </div>
    </Show>
  );
}
