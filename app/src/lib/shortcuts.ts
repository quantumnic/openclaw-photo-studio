export type ShortcutContext = "global" | "library" | "develop" | "dialog";

export interface ShortcutBinding {
  key: string;
  action: string;
  context: ShortcutContext;
  description: string;
}

export const DEFAULT_BINDINGS: ShortcutBinding[] = [
  { key: "g", action: "view.grid", context: "global", description: "Grid view" },
  { key: "e", action: "view.loupe", context: "global", description: "Loupe view" },
  { key: "d", action: "module.develop", context: "global", description: "Develop module" },
  { key: "ArrowLeft", action: "photo.prev", context: "global", description: "Previous photo" },
  { key: "ArrowRight", action: "photo.next", context: "global", description: "Next photo" },
  { key: "0", action: "rate.0", context: "global", description: "Rating: none" },
  { key: "1", action: "rate.1", context: "global", description: "Rating: 1 star" },
  { key: "2", action: "rate.2", context: "global", description: "Rating: 2 stars" },
  { key: "3", action: "rate.3", context: "global", description: "Rating: 3 stars" },
  { key: "4", action: "rate.4", context: "global", description: "Rating: 4 stars" },
  { key: "5", action: "rate.5", context: "global", description: "Rating: 5 stars" },
  { key: "p", action: "flag.pick", context: "global", description: "Flag: Pick" },
  { key: "x", action: "flag.reject", context: "global", description: "Flag: Reject" },
  { key: "u", action: "flag.none", context: "global", description: "Flag: None" },
  { key: "\\", action: "develop.before_after", context: "develop", description: "Before/After toggle" },
  { key: "cmd+c", action: "develop.copy", context: "develop", description: "Copy settings" },
  { key: "cmd+v", action: "develop.paste", context: "develop", description: "Paste settings" },
  { key: "cmd+shift+r", action: "develop.reset", context: "develop", description: "Reset settings" },
  { key: "Tab", action: "ui.toggle_panels", context: "global", description: "Toggle panels" },
  { key: "cmd+shift+e", action: "export.open", context: "global", description: "Export" },
  { key: "cmd+shift+i", action: "import.open", context: "global", description: "Import folder" },
  { key: "cmd+k", action: "ui.command_palette", context: "global", description: "Command palette" },
];

export class ShortcutEngine {
  private handlers = new Map<string, () => void>();
  private context: ShortcutContext = "global";

  setContext(ctx: ShortcutContext): void {
    this.context = ctx;
  }

  register(action: string, handler: () => void): void {
    this.handlers.set(action, handler);
  }

  execute(action: string): void {
    this.handlers.get(action)?.();
  }

  handleKeyDown(e: KeyboardEvent): void {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

    const key = this.normalizeKey(e);

    for (const binding of DEFAULT_BINDINGS) {
      if (binding.key === key) {
        if (binding.context === "global" || binding.context === this.context) {
          e.preventDefault();
          this.execute(binding.action);
          return;
        }
      }
    }
  }

  private normalizeKey(e: KeyboardEvent): string {
    const parts: string[] = [];
    if (e.metaKey || e.ctrlKey) parts.push("cmd");
    if (e.shiftKey) parts.push("shift");
    if (e.altKey) parts.push("alt");
    parts.push(e.key);
    return parts.join("+");
  }
}
