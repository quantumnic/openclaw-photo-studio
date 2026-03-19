# OpenClaw Photo Studio — Shortcut-Spezifikation

> Version: 0.1.0-draft | 2026-03-19

---

## 1. Shortcut-System Architektur

### 1.1 Designziele
- Jede Aktion ist per Shortcut erreichbar
- Lightroom-kompatibles Default-Profil
- Vim-artiges Expert-Profil (optional)
- Frei konfigurierbar (JSON/TOML)
- Konflikt-Erkennung und -Auflösung
- Kontextabhängig (Library vs. Develop vs. Global)
- Chord-Support (Sequenzen wie `g` → `e`)
- Macro-Recording

### 1.2 Shortcut-Auflösung

```
Priority (highest to lowest):
1. Modal Shortcuts (wenn Vim-Mode aktiv)
2. Tool-spezifische Shortcuts (z.B. Crop-Tool aktiv)
3. Modul-spezifische Shortcuts (Develop-Modul)
4. Globale Shortcuts
5. OS-Shortcuts (werden nicht überschrieben)
```

### 1.3 Keymap-Format

```json
{
  "name": "Lightroom Compatible",
  "version": "1.0.0",
  "platform": "all",
  "bindings": {
    "global": {
      "cmd+k": "command_palette.open",
      "cmd+,": "preferences.open",
      "cmd+z": "edit.undo",
      "cmd+shift+z": "edit.redo",
      "cmd+q": "app.quit",
      "tab": "ui.toggle_panels",
      "shift+tab": "ui.toggle_all_panels",
      "f": "ui.fullscreen_toggle",
      "l": "ui.lights_out_cycle",
      "t": "ui.toolbar_toggle"
    },
    "library": {
      "g": "view.grid",
      "e": "view.loupe",
      "c": "view.compare",
      "n": "view.survey",
      "d": "module.develop",
      "cmd+shift+i": "library.import",
      "cmd+shift+e": "export.open"
    },
    "develop": {
      "\\": "develop.before_after_toggle",
      "r": "tool.crop",
      "k": "tool.adjustment_brush",
      "m": "tool.graduated_filter",
      "shift+m": "tool.radial_filter",
      "j": "tool.healing",
      ",": "preset.previous_preview",
      ".": "preset.next_preview"
    }
  }
}
```

---

## 2. Vollständige Shortcut-Liste (Lightroom-Kompatibles Profil)

### 2.1 Navigation & Module

| Shortcut | Aktion | Kontext |
|----------|--------|---------|
| `G` | Grid View | Global |
| `E` | Loupe View | Global |
| `D` | Develop Module | Global |
| `C` | Compare View | Library |
| `N` | Survey View | Library |
| `Cmd+Alt+1` | Library Module | Global |
| `Cmd+Alt+2` | Develop Module | Global |
| `Cmd+Alt+3` | Map Module | Global |
| `Cmd+Alt+4` | Print Module | Global |
| `←` | Previous Photo | Library/Develop |
| `→` | Next Photo | Library/Develop |
| `Home` | First Photo | Library |
| `End` | Last Photo | Library |
| `Space` | Next Unflagged/Unrated | Library |
| `Page Up` | Scroll Grid Up | Library Grid |
| `Page Down` | Scroll Grid Down | Library Grid |
| `Cmd+↑` | Parent Folder | Library |
| `Enter` | Open in Loupe / Confirm | Library/Dialog |

### 2.2 Rating & Flagging

| Shortcut | Aktion | Kontext |
|----------|--------|---------|
| `0` | Rating: None | Global |
| `1` | Rating: ★ | Global |
| `2` | Rating: ★★ | Global |
| `3` | Rating: ★★★ | Global |
| `4` | Rating: ★★★★ | Global |
| `5` | Rating: ★★★★★ | Global |
| `]` | Rating +1 | Global |
| `[` | Rating -1 | Global |
| `P` | Flag: Pick | Global |
| `X` | Flag: Reject | Global |
| `U` | Flag: Unflagged | Global |
| `Cmd+↑` | Increase Flag | Global |
| `Cmd+↓` | Decrease Flag | Global |
| `6` | Color Label: Red | Global |
| `7` | Color Label: Yellow | Global |
| `8` | Color Label: Green | Global |
| `9` | Color Label: Blue | Global |

### 2.3 Selection

| Shortcut | Aktion | Kontext |
|----------|--------|---------|
| `Cmd+A` | Select All | Library |
| `Cmd+D` | Deselect All | Library |
| `Cmd+Shift+A` | Invert Selection | Library |
| `Shift+Click` | Range Select | Library Grid |
| `Cmd+Click` | Toggle Select | Library Grid |
| `/` | Filter Bar Toggle | Library |
| `Cmd+F` | Find / Search | Global |
| `B` | Add to Quick Collection | Library |
| `Cmd+B` | Show Quick Collection | Library |
| `Cmd+Shift+B` | Save Quick Collection | Library |

### 2.4 View Controls

| Shortcut | Aktion | Kontext |
|----------|--------|---------|
| `F` | Fullscreen Toggle | Global |
| `Shift+F` | Fullscreen with Panels | Global |
| `Tab` | Toggle Side Panels | Global |
| `Shift+Tab` | Toggle All Panels | Global |
| `T` | Toggle Toolbar | Global |
| `L` | Lights Out (Cycle: Normal → Dim → Dark) | Global |
| `I` | Info Overlay (Cycle) | Library |
| `J` | Highlight/Shadow Clipping Toggle | Develop |
| `Cmd+=` | Zoom In | Library/Develop |
| `Cmd+-` | Zoom Out | Library/Develop |
| `Cmd+0` | Zoom Fit | Library/Develop |
| `Cmd+Alt+0` | Zoom 1:1 | Library/Develop |
| `Z` | Toggle Fit ↔ 1:1 | Develop |
| `+` | Increase Thumbnail Size | Library Grid |
| `-` | Decrease Thumbnail Size | Library Grid |
| `Ctrl+Alt+1-5` | Grid Thumbnail Size Preset | Library Grid |

### 2.5 Develop — Tool Selection

| Shortcut | Aktion | Kontext |
|----------|--------|---------|
| `R` | Crop Tool | Develop |
| `K` | Adjustment Brush | Develop |
| `M` | Graduated Filter | Develop |
| `Shift+M` | Radial Filter | Develop |
| `Q` | Healing/Clone Tool | Develop |
| `Shift+Q` | Toggle Heal ↔ Clone Mode | Develop |
| `W` | White Balance Picker | Develop |
| `A` | Straighten Tool | Develop Crop |
| `O` | Crop Overlay Cycle | Develop Crop |
| `Shift+O` | Crop Overlay Orientation | Develop Crop |
| `X` | Crop: Swap Aspect Ratio | Develop Crop |
| `Escape` | Close Current Tool | Develop |

### 2.6 Develop — Adjustments

| Shortcut | Aktion | Kontext |
|----------|--------|---------|
| `Cmd+C` | Copy All Settings | Develop |
| `Cmd+V` | Paste Settings | Develop |
| `Cmd+Shift+C` | Copy Selected Settings | Develop |
| `Cmd+Shift+V` | Paste Selected Settings | Develop |
| `Cmd+Shift+S` | Sync Settings | Develop |
| `Cmd+Alt+Shift+S` | Toggle Auto-Sync | Develop |
| `\\` | Before/After Toggle | Develop |
| `Shift+\\` | Before/After Side-by-Side | Develop |
| `Alt+\\` | Before/After Split | Develop |
| `Cmd+'` | Create Virtual Copy | Develop |
| `Cmd+N` | New Snapshot | Develop |
| `Cmd+Shift+R` | Reset All Settings | Develop |
| `Cmd+Shift+E` | Match Total Exposure | Develop |
| `Cmd+Shift+H` | Edit Transfer History | Develop |
| `Cmd+Shift+N` | Save as New Preset | Develop |

### 2.7 Develop — Panel Navigation

| Shortcut | Aktion | Kontext |
|----------|--------|---------|
| `Cmd+1` | Basic Panel | Develop |
| `Cmd+2` | Tone Curve Panel | Develop |
| `Cmd+3` | HSL / Color Panel | Develop |
| `Cmd+4` | Color Grading Panel | Develop |
| `Cmd+5` | Detail Panel | Develop |
| `Cmd+6` | Lens Corrections Panel | Develop |
| `Cmd+7` | Transform Panel | Develop |
| `Cmd+8` | Effects Panel | Develop |
| `Cmd+9` | Calibration Panel | Develop |
| `Cmd+0` | Presets Panel (Toggle) | Develop |

### 2.8 Develop — Fine Control

| Shortcut | Aktion | Kontext |
|----------|--------|---------|
| `Alt+Drag` auf Slider | Show Masking Preview | Develop (Sharpening Masking, NR) |
| `Shift+Drag` auf Slider | Fine-Tuning (1/4 Speed) | Develop |
| `Double-Click` auf Slider | Reset to Default | Develop |
| `← →` auf fokussiertem Slider | ±1 | Develop |
| `Shift+← →` auf fokussiertem Slider | ±0.1 | Develop |
| `0-9` + Enter auf fokussiertem Slider | Direct Value Input | Develop |

### 2.9 Photo Management

| Shortcut | Aktion | Kontext |
|----------|--------|---------|
| `Cmd+R` | Rotate Clockwise | Global |
| `Cmd+[` | Rotate Counter-Clockwise | Global |
| `Cmd+Shift+H` | Flip Horizontal | Global |
| `Cmd+Shift+V` | Flip Vertical | Global (nur Library) |
| `Cmd+G` | Group into Stack | Library |
| `Cmd+Shift+G` | Unstack | Library |
| `S` | Collapse/Expand Stack | Library |
| `Shift+S` | Move to Top of Stack | Library |
| `Delete` / `Backspace` | Remove from Catalog (Confirm) | Library |
| `Cmd+Delete` | Delete from Disk (Confirm) | Library |
| `Cmd+E` | Edit in External Editor | Global |

### 2.10 Export

| Shortcut | Aktion | Kontext |
|----------|--------|---------|
| `Cmd+Shift+E` | Export Dialog | Global |
| `Cmd+Alt+Shift+E` | Export with Last Settings | Global |
| `Cmd+J` | Export as JPEG (Quick) | Global |
| `Cmd+Shift+J` | Export All Selected | Global |

### 2.11 Map Module

| Shortcut | Aktion | Kontext |
|----------|--------|---------|
| `+` / `-` | Zoom In / Out | Map |
| `Arrow Keys` | Pan Map | Map |
| `Cmd+Shift+G` | Toggle GPS Track Overlay | Map |
| `Cmd+Shift+L` | Lock GPS Position | Map |

### 2.12 System & UI

| Shortcut | Aktion | Kontext |
|----------|--------|---------|
| `Cmd+K` | Command Palette | Global |
| `Cmd+,` | Preferences | Global |
| `Cmd+Z` | Undo | Global |
| `Cmd+Shift+Z` | Redo | Global |
| `Cmd+S` | Save Metadata to File (XMP) | Global |
| `Cmd+W` | Close Window | Global |
| `Cmd+Q` | Quit | Global |
| `?` | Show Shortcut Overlay | Global |
| `F1` | Help | Global |
| `F5` | Toggle Second Display | Global |
| `Cmd+Alt+F` | Filter Panel | Library |
| `Cmd+P` | Print | Print Module |

### 2.13 Collections

| Shortcut | Aktion | Kontext |
|----------|--------|---------|
| `B` | Add to Quick Collection | Library |
| `Ctrl+N` | New Collection | Library |
| `Ctrl+Shift+N` | New Smart Collection | Library |
| `Ctrl+Alt+N` | New Collection Set | Library |

### 2.14 Metadata & Keywords

| Shortcut | Aktion | Kontext |
|----------|--------|---------|
| `Cmd+K` | Add Keywords (Feld fokussieren) | Library |
| `Cmd+Shift+K` | Keyword Set wechseln | Library |
| `Cmd+Alt+K` | Keyword Suggestions | Library |
| `Cmd+I` | Info / Metadata Panel Toggle | Library |

---

## 3. Vim-Mode (Expert Profile, Optional)

### 3.1 Aktivierung
- Preferences → Keyboard → Enable Vim Mode
- Oder: Command Palette → "Enable Vim Mode"

### 3.2 Modes

```
┌──────────────┐    i/a/R    ┌──────────────┐
│              │ ──────────> │              │
│  NORMAL      │             │  EDIT        │
│  (navigate,  │ <────────── │  (slider     │
│   rate,      │    Escape   │   adjust,    │
│   select)    │             │   text input)│
│              │             │              │
└──────┬───────┘             └──────────────┘
       │
       │ :
       ▼
┌──────────────┐
│              │
│  COMMAND     │
│  (:export,   │
│   :import,   │
│   :filter)   │
│              │
└──────────────┘
       │
       │ v
       ▼
┌──────────────┐
│              │
│  VISUAL      │
│  (range      │
│   select)    │
│              │
└──────────────┘
```

### 3.3 Normal Mode Shortcuts

| Shortcut | Aktion |
|----------|--------|
| `h` | Previous Photo |
| `j` | Next Row (Grid) / Next Photo (Loupe) |
| `k` | Previous Row (Grid) / Previous Photo (Loupe) |
| `l` | Next Photo |
| `gg` | Go to First Photo |
| `G` | Go to Last Photo |
| `{number}G` | Go to Photo #{number} |
| `/` | Search |
| `n` | Next Search Result |
| `N` | Previous Search Result |
| `v` | Start Visual Selection |
| `V` | Visual Line Selection (Row) |
| `dd` | Reject Current Photo |
| `yy` | Copy Settings |
| `pp` | Paste Settings |
| `YY` | Copy Selected Settings (opens dialog) |
| `PP` | Paste Selected Settings |
| `u` | Undo |
| `Ctrl+R` | Redo |
| `.` | Repeat Last Action |
| `zz` | Center Current Photo in View |
| `za` | Toggle Current Panel |
| `zo` | Open Current Panel |
| `zc` | Close Current Panel |
| `zM` | Close All Panels |
| `zR` | Open All Panels |
| `1`-`5` | Set Rating (Stars) |
| `0` | Clear Rating |
| `'p` | Flag: Pick |
| `'x` | Flag: Reject |
| `'u` | Flag: Unflagged |
| `'r` | Color: Red |
| `'y` | Color: Yellow |
| `'g` | Color: Green |
| `'b` | Color: Blue |
| `mm` | Create Snapshot |
| `mr` | Reset All Edits |
| `mc` | Crop Tool |
| `mk` | Adjustment Brush |

### 3.4 Command Mode

| Command | Aktion |
|---------|--------|
| `:e` / `:export` | Open Export Dialog |
| `:i` / `:import` | Open Import Dialog |
| `:sync` | Sync Settings |
| `:preset <name>` | Apply Preset by Name |
| `:rate <n>` | Set Rating |
| `:flag pick\|reject\|none` | Set Flag |
| `:color red\|yellow\|green\|blue\|none` | Set Color Label |
| `:sort <field>` | Sort by Field |
| `:filter <expr>` | Filter (e.g., `:filter rating>=4 camera:sony`) |
| `:collection <name>` | Jump to Collection |
| `:q` | Quit (with confirmation) |
| `:w` | Save Metadata to XMP |
| `:wq` | Save and Quit |
| `:set <option>=<value>` | Set Option |
| `:help` | Show Help |
| `:version` | Show Version |
| `:benchmark` | Show Performance Stats |

### 3.5 Visual Mode

| Shortcut | Aktion |
|----------|--------|
| `v` | Start/End Visual Selection |
| `V` | Visual Line (Row) |
| `Escape` | Cancel Selection |
| `d` | Reject Selected |
| `y` | Copy Settings (from first) |
| `p` | Paste Settings to Selected |
| `1`-`5` | Rate All Selected |
| `'p` / `'x` / `'u` | Flag All Selected |
| `:` | Command on Selection |

---

## 4. Minimal Pro Profile

Für Fotografen, die minimale Shortcuts wollen:

| Shortcut | Aktion |
|----------|--------|
| `←` `→` | Navigate |
| `1`-`5` | Rate |
| `P` `X` `U` | Flag |
| `Space` | Next Unrated |
| `Enter` | Develop |
| `Escape` | Back to Library |
| `Cmd+C/V` | Copy/Paste Edits |
| `Cmd+E` | Export |
| `Cmd+Z` | Undo |
| `Cmd+K` | Command Palette (alles andere hier) |

Alles andere über Command Palette erreichbar.

---

## 5. Shortcut-Konfiguration

### 5.1 Shortcut Editor UI
- Preferences → Keyboard
- Suchbar (Fuzzy-Search über Aktionsnamen)
- Konflikte rot markiert
- Import/Export als JSON

### 5.2 Shortcut Conflict Resolver
```
┌─────────────────────────────────────────────────┐
│  ⚠ Shortcut Conflict                      [X]  │
├─────────────────────────────────────────────────┤
│                                                   │
│  Cmd+Shift+V is already assigned to:            │
│                                                   │
│  1. develop.paste_selected (Develop Module)     │
│  2. photo.flip_vertical (Library Module)        │
│                                                   │
│  Options:                                        │
│  ○ Keep both (context-dependent)                │
│  ○ Replace existing assignment                   │
│  ○ Cancel                                        │
│                                                   │
│  [Apply]  [Cancel]                               │
│                                                   │
└─────────────────────────────────────────────────┘
```

### 5.3 Shortcut Discovery Overlay
**Shortcut:** `?`
**Zeigt:** Halbtransparentes Overlay mit allen aktiven Shortcuts für den aktuellen Kontext.

```
┌──────────────────────────────────────────────────────┐
│                  KEYBOARD SHORTCUTS                    │
│                  (Current: Develop)                     │
├──────────────────────────────────────────────────────┤
│                                                        │
│  NAVIGATION          RATING           TOOLS           │
│  ← → Navigate        0-5 Stars       R  Crop         │
│  Space  Next          P   Pick        K  Brush        │
│  G  Grid              X   Reject      M  Gradient     │
│  E  Loupe             U   Unflag     ⇧M  Radial      │
│  D  Develop           6-9 Colors      Q  Healing      │
│                                        W  WB Picker    │
│  VIEW                EDIT                              │
│  \  Before/After     ⌘C  Copy        ⌘Z  Undo        │
│  Z  Fit↔1:1          ⌘V  Paste      ⌘⇧Z  Redo       │
│  F  Fullscreen       ⌘⇧C Sel.Copy   ⌘⇧R  Reset      │
│  L  Lights Out       ⌘⇧V Sel.Paste  ⌘'   VirtCopy   │
│  Tab Panels          ⌘⇧S Sync       ⌘N   Snapshot    │
│                                                        │
│  Press ? again to dismiss     ⌘K Command Palette      │
└──────────────────────────────────────────────────────┘
```

---

## 6. Platform-Unterschiede

| Action | macOS | Windows | Linux |
|--------|-------|---------|-------|
| Modifier | `Cmd` | `Ctrl` | `Ctrl` |
| Alt Modifier | `Option` | `Alt` | `Alt` |
| Preferences | `Cmd+,` | `Ctrl+,` | `Ctrl+,` |
| Quit | `Cmd+Q` | `Alt+F4` | `Ctrl+Q` |
| Fullscreen | `Cmd+Ctrl+F` | `F11` | `F11` |
| Close Window | `Cmd+W` | `Ctrl+W` | `Ctrl+W` |

**OS-Konflikt-Vermeidung:**
- macOS: Keine Konflikte mit Spotlight (`Cmd+Space`), Mission Control (`Ctrl+↑↓`)
- Windows: Keine Konflikte mit Win-Key-Shortcuts
- Linux: Keine Konflikte mit DE-Shortcuts (Gnome/KDE)
- Alle potenziell kollidierenden Shortcuts sind umkonfigurierbar
