/**
 * Vim Mode — Modal keyboard navigation for expert users
 *
 * When enabled, provides modal editing interface:
 * - NORMAL: Navigation, rating, flags
 * - COMMAND: Ex-style commands (:export, :rate, etc.)
 * - VISUAL: Multi-select with keyboard
 */

type VimModeState = "normal" | "edit" | "command" | "visual";

export class VimMode {
  private state: VimModeState = "normal";
  private commandBuffer = "";
  private onAction: (action: string, args?: any) => void;
  public isEnabled = false;

  constructor(onAction: (action: string, args?: any) => void) {
    this.onAction = onAction;
  }

  /**
   * Handle keyboard events in vim mode
   * @returns true if the key was consumed by vim mode
   */
  handleKey(e: KeyboardEvent): boolean {
    // Don't consume if focus is on an input element
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
      return false;
    }

    switch (this.state) {
      case "normal": return this.handleNormalMode(e);
      case "command": return this.handleCommandMode(e);
      case "visual": return this.handleVisualMode(e);
      default: return false;
    }
  }

  private handleNormalMode(e: KeyboardEvent): boolean {
    // Navigation
    if (e.key === "h") { this.onAction("photo.prev"); return true; }
    if (e.key === "l") { this.onAction("photo.next"); return true; }
    if (e.key === "j") { this.onAction("photo.next_row"); return true; }
    if (e.key === "k") { this.onAction("photo.prev_row"); return true; }

    // gg → first photo
    if (e.key === "g" && this.commandBuffer === "g") {
      this.onAction("photo.first");
      this.commandBuffer = "";
      return true;
    }
    if (e.key === "g") { this.commandBuffer = "g"; return true; }

    // G → last photo
    if (e.key === "G") { this.onAction("photo.last"); return true; }

    // Rating (0-5)
    if (e.key >= "1" && e.key <= "5") {
      this.onAction("rate", parseInt(e.key));
      return true;
    }
    if (e.key === "0") { this.onAction("rate", 0); return true; }

    // Flags
    if (e.key === "p" || e.key === "P") { this.onAction("flag.pick"); return true; }
    if (e.key === "x" || e.key === "X") { this.onAction("flag.reject"); return true; }
    if (e.key === "u" || e.key === "U") { this.onAction("flag.none"); return true; }

    // Edit operations
    // yy → copy settings
    if (e.key === "y" && this.commandBuffer === "y") {
      this.onAction("develop.copy");
      this.commandBuffer = "";
      return true;
    }
    if (e.key === "y") { this.commandBuffer = "y"; return true; }

    // p → paste (Ctrl+P in vim mode to avoid conflict with flag P)
    if (e.key === "p" && e.ctrlKey) { this.onAction("develop.paste"); return true; }

    // Undo/Redo
    if (e.key === "u" && e.ctrlKey) { this.onAction("edit.undo"); return true; }
    if (e.key === "r" && e.ctrlKey) { this.onAction("edit.redo"); return true; }

    // Repeat last action
    if (e.key === ".") { this.onAction("edit.repeat"); return true; }

    // Mode switches
    if (e.key === ":") {
      this.state = "command";
      this.commandBuffer = "";
      return true;
    }
    if (e.key === "v") {
      this.state = "visual";
      return true;
    }
    if (e.key === "Escape") {
      this.commandBuffer = "";
      return true;
    }

    // Module switches
    if (e.key === "d" && !e.metaKey && !e.ctrlKey) {
      this.onAction("module.develop");
      return true;
    }

    return false;
  }

  private handleCommandMode(e: KeyboardEvent): boolean {
    if (e.key === "Escape") {
      this.state = "normal";
      this.commandBuffer = "";
      return true;
    }

    if (e.key === "Enter") {
      this.executeCommand(this.commandBuffer);
      this.state = "normal";
      this.commandBuffer = "";
      return true;
    }

    if (e.key === "Backspace") {
      this.commandBuffer = this.commandBuffer.slice(0, -1);
      return true;
    }

    if (e.key.length === 1) {
      this.commandBuffer += e.key;
      return true;
    }

    return false;
  }

  private handleVisualMode(e: KeyboardEvent): boolean {
    if (e.key === "Escape") {
      this.state = "normal";
      return true;
    }

    // Arrow keys extend selection
    if (e.key === "h") { this.onAction("select.extend_prev"); return true; }
    if (e.key === "l") { this.onAction("select.extend_next"); return true; }
    if (e.key === "j") { this.onAction("select.extend_down"); return true; }
    if (e.key === "k") { this.onAction("select.extend_up"); return true; }

    // Apply to selection
    if (e.key >= "0" && e.key <= "5") {
      this.onAction("rate.selection", parseInt(e.key));
      return true;
    }
    if (e.key === "p") {
      this.onAction("develop.paste.selection");
      return true;
    }

    return false;
  }

  private executeCommand(cmd: string): void {
    // :e or :export → open export dialog
    if (cmd === "e" || cmd === "export") {
      this.onAction("export.open");
      return;
    }

    // :i or :import → open import
    if (cmd === "i" || cmd === "import") {
      this.onAction("import.open");
      return;
    }

    // :sync → sync settings
    if (cmd === "sync") {
      this.onAction("develop.sync");
      return;
    }

    // :w → save XMP sidecar
    if (cmd === "w") {
      this.onAction("catalog.save_sidecar");
      return;
    }

    // :q → quit (with confirm)
    if (cmd === "q") {
      this.onAction("app.quit");
      return;
    }

    // :rate N → set rating
    if (cmd.startsWith("rate ")) {
      const n = parseInt(cmd.slice(5));
      if (!isNaN(n)) this.onAction("rate", n);
      return;
    }

    // :flag pick/reject/none
    if (cmd.startsWith("flag ")) {
      this.onAction("flag." + cmd.slice(5).trim());
      return;
    }

    // :preset <name> → apply preset by name
    if (cmd.startsWith("preset ")) {
      this.onAction("preset.apply_by_name", cmd.slice(7).trim());
      return;
    }
  }

  get currentState(): VimModeState {
    return this.state;
  }

  get commandBufferContent(): string {
    return this.commandBuffer;
  }
}
