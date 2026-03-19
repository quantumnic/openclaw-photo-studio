import { createSignal, For, onMount, Show } from "solid-js";

interface Toast {
  id: string;
  message: string;
  type: "success" | "error" | "info";
  duration: number;
}

const [toasts, setToasts] = createSignal<Toast[]>([]);

export function showToast(message: string, type: "success" | "error" | "info" = "info", duration = 3000) {
  const id = `toast-${Date.now()}-${Math.random()}`;
  const toast: Toast = { id, message, type, duration };

  setToasts(prev => [...prev, toast]);

  if (duration > 0) {
    setTimeout(() => {
      setToasts(prev => prev.filter(t => t.id !== id));
    }, duration);
  }
}

export function ToastContainer() {
  const bgColors = {
    success: "bg-[#1a3a1a] border-[#2a5a2a]",
    error: "bg-[#3a1a1a] border-[#5a2a2a]",
    info: "bg-[#1a2a3a] border-[#2a3a5a]",
  };

  const textColors = {
    success: "text-[#4ade80]",
    error: "text-[#ff6b6b]",
    info: "text-[#4a9eff]",
  };

  return (
    <div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none">
      <For each={toasts()}>
        {(toast) => (
          <div
            class={`
              px-4 py-3 rounded-lg border shadow-lg pointer-events-auto
              animate-[slideInRight_0.3s_ease-out]
              ${bgColors[toast.type]}
            `}
            style={{
              "animation-fill-mode": "forwards",
            }}
          >
            <div class="flex items-center gap-2">
              <Show when={toast.type === "success"}>
                <span class={textColors.success}>✓</span>
              </Show>
              <Show when={toast.type === "error"}>
                <span class={textColors.error}>✗</span>
              </Show>
              <Show when={toast.type === "info"}>
                <span class={textColors.info}>ℹ</span>
              </Show>
              <span class={`text-sm ${textColors[toast.type]}`}>{toast.message}</span>
            </div>
          </div>
        )}
      </For>
    </div>
  );
}
