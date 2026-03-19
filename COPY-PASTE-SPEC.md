# OpenClaw Photo Studio — Copy/Paste & Sync Spezifikation

> Version: 0.1.0-draft | 2026-03-19

---

## 1. Philosophie

Copy/Paste von Edits ist die einzelne wichtigste Produktivitätsfunktion in einer Foto-Workflow-Software. Ein Hochzeitsfotograf, der 3'000 Fotos bearbeitet, verwendet Copy/Paste hundertfach. Es muss:

- **Maximal 2-3 Tasten** brauchen für den Standardfall
- **Selektiv** sein (nur bestimmte Module kopieren)
- **Intelligent** mit Crop, Masken und kamera-spezifischen Settings umgehen
- **Reversibel** sein (Undo für Batch-Paste)
- **Vorschau** bieten vor Batch-Anwendung

---

## 2. Operationen

### 2.1 Copy All Settings
**Shortcut:** `Cmd+C` (macOS) / `Ctrl+C` (Win/Linux)
**Aktion:** Kopiert ALLE Develop-Settings des aktuellen Fotos in den Clipboard.
**Datenstruktur:** Vollständiger `EditRecipe` (JSON)

### 2.2 Copy Selected Settings
**Shortcut:** `Cmd+Shift+C`
**Aktion:** Öffnet Dialog zur Auswahl der zu kopierenden Module.
**UI:**
```
┌─────────────────────────────────────────┐
│  Copy Settings                     [X]  │
├─────────────────────────────────────────┤
│                                         │
│  ☑ Check All    ☐ Check None           │
│                                         │
│  ☑ White Balance                       │
│  ☑ Basic Tone                          │
│    ☑ Exposure                          │
│    ☑ Contrast                          │
│    ☑ Highlights                        │
│    ☑ Shadows                           │
│    ☑ Whites                            │
│    ☑ Blacks                            │
│  ☑ Clarity / Dehaze                    │
│  ☑ Vibrance / Saturation              │
│  ☑ Tone Curve                          │
│  ☑ HSL / Color Mixer                  │
│  ☑ Color Grading                       │
│  ☐ Detail (Sharpening / NR)           │
│  ☐ Lens Corrections                   │
│  ☐ Transform / Geometry               │
│  ☐ Effects (Vignette / Grain)         │
│  ☑ Calibration                         │
│  ☐ Crop                               │
│  ☐ Local Adjustments                   │
│  ☐ Healing / Clone Spots              │
│                                         │
│  [Copy]  [Cancel]                      │
│                                         │
│  Shortcuts: Space = Toggle             │
│             Enter = Copy               │
│             Esc = Cancel               │
└─────────────────────────────────────────┘
```

**Der Dialog merkt sich die letzte Auswahl** (Session-persistent).

### 2.3 Paste Settings
**Shortcut:** `Cmd+V`
**Aktion:** Wendet die kopierten Settings auf das aktuelle Foto (oder alle selektierten Fotos) an.
**Verhalten bei Mehrfachauswahl:**
- Wenn 1 Foto selektiert: Paste auf dieses Foto
- Wenn N Fotos selektiert: Paste auf alle N Fotos
- Fortschrittsanzeige bei >10 Fotos

### 2.4 Paste Selected Settings
**Shortcut:** `Cmd+Shift+V`
**Aktion:** Paste mit dem selben Auswahl-Dialog wie Copy. Erlaubt letztmalige Anpassung der zu pastenden Module.

### 2.5 Sync Settings
**Shortcut:** `Cmd+Shift+S` (oder Sync-Button)
**Aktion:** Wenn mehrere Fotos selektiert: Settings des "aktiven" Fotos auf alle anderen selektierten anwenden.
**Unterschied zu Paste:** Sync nimmt immer das aktive Foto als Quelle. Paste nimmt den Clipboard.

### 2.6 Auto-Sync Mode
**Shortcut:** `Cmd+Alt+Shift+S` (Toggle)
**Aktion:** Wenn aktiviert: JEDE Änderung am aktuellen Foto wird sofort auf alle selektierten Fotos angewendet.
**Indikator:** Gelbes Sync-Symbol in der Toolbar.
**Warnung:** Bei Aktivierung: "Auto-Sync aktiv: Änderungen werden auf [N] Fotos angewendet."
**Undo:** Jede Auto-Sync-Operation ist einzeln rückgängig machbar.

### 2.7 Preset aus aktuellem Edit erzeugen
**Shortcut:** `Cmd+Shift+N`
**Aktion:** Aktueller Edit → neues Preset. Dialog für Name, Gruppe, und welche Module enthalten sein sollen.

### 2.8 Reset einzelner Module
**Shortcut:** Doppelklick auf Modul-Header oder Rechtsklick → "Reset"
**Aktion:** Setzt ein einzelnes Modul auf Default zurück.
**Batch:** Wenn Auto-Sync aktiv, gilt Reset für alle selektierten Fotos.

---

## 3. Datenmodell für Copy/Paste

### 3.1 EditClipboard

```typescript
interface EditClipboard {
  // Was wurde kopiert
  source: {
    photoId: string;
    cameraModel: string | null;
    lensModel: string | null;
    aspectRatio: number;        // width / height
    orientation: number;        // 1-8 (EXIF)
  };

  // Kopierte Settings
  settings: Partial<EditRecipe>;

  // Welche Module wurden kopiert (Bitmaske oder Set)
  appliedModules: Set<EditModule>;

  // Zeitstempel
  copiedAt: string;             // ISO 8601

  // Konfiguration
  includesCrop: boolean;
  includesLocalAdjustments: boolean;
  includesHealingSpots: boolean;
  includesLensProfile: boolean;
}

enum EditModule {
  WhiteBalance = "white_balance",
  BasicTone = "basic_tone",
  Exposure = "exposure",
  Contrast = "contrast",
  Highlights = "highlights",
  Shadows = "shadows",
  Whites = "whites",
  Blacks = "blacks",
  Clarity = "clarity",
  Dehaze = "dehaze",
  Vibrance = "vibrance",
  Saturation = "saturation",
  ToneCurve = "tone_curve",
  HSL = "hsl",
  ColorGrading = "color_grading",
  Detail = "detail",
  Sharpening = "sharpening",
  NoiseReduction = "noise_reduction",
  LensCorrections = "lens_corrections",
  Transform = "transform",
  Effects = "effects",
  Grain = "grain",
  Vignette = "vignette",
  Calibration = "calibration",
  Crop = "crop",
  LocalAdjustments = "local_adjustments",
  HealingSpots = "healing_spots",
  Monochrome = "monochrome",
}
```

### 3.2 Copy/Paste Exclusion Rules

```typescript
interface PasteRules {
  // Automatische Anpassungen
  cropBehavior: "skip" | "apply_exact" | "apply_ratio_only" | "adapt_to_orientation";
  localAdjustmentBehavior: "skip" | "apply" | "warn";
  healingSpotBehavior: "skip" | "apply" | "warn";

  // Kamera-Kompatibilität
  lensProfileBehavior: "skip_if_different_lens" | "apply_always" | "warn";
  cameraProfileBehavior: "skip_if_different_camera" | "apply_always" | "warn";

  // Warnungen
  warnOnDifferentCamera: boolean;
  warnOnDifferentAspectRatio: boolean;
  warnOnDifferentOrientation: boolean;

  // Optionale Anpassungen
  matchTotalExposure: boolean;  // Belichtung angleichen
  adaptWhiteBalance: boolean;   // WB anpassen falls anders belichtet
}
```

### 3.3 Intelligentes Crop-Handling

Wenn das Quell-Foto ein anderes Seitenverhältnis hat als das Ziel:

| Situation | Verhalten |
|-----------|-----------|
| Gleiche Kamera, gleiches Ratio | Crop 1:1 übernehmen |
| Gleiche Kamera, anderes Ratio (Portrait/Landscape) | Crop auf neues Ratio anpassen, Position interpolieren |
| Andere Kamera, gleiches Ratio | Crop relativ übernehmen |
| Andere Kamera, anderes Ratio | Crop-Ratio übernehmen, Position zentrieren |

Konfigurierbar: Der Nutzer kann wählen, ob Crop übersprungen oder intelligent angepasst werden soll.

---

## 4. Batch-Preview

Vor dem Anwenden auf viele Fotos (>5) bietet OCPS eine Vorschau:

```
┌──────────────────────────────────────────────────┐
│  Paste Preview — 147 Fotos betroffen        [X]  │
├──────────────────────────────────────────────────┤
│                                                    │
│  Quelle: DSC_4523.ARW (Nikon Z8, 24-70 f/2.8)  │
│  Kopierte Module: WB, Tone, HSL, Color Grading  │
│                                                    │
│  ⚠ Warnungen:                                    │
│  • 12 Fotos: andere Kamera (Sony A7IV)           │
│  • 3 Fotos: anderes Seitenverhältnis (16:9)      │
│  • 0 Fotos: anderes Objektiv                     │
│                                                    │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐            │
│  │ Before  │ │ After   │ │ Before  │ ...        │
│  │ [Foto1] │ │ [Foto1] │ │ [Foto2] │            │
│  └─────────┘ └─────────┘ └─────────┘            │
│                                                    │
│  ☑ Auch auf abweichende Kameras anwenden         │
│  ☑ Crop überspringen bei anderem Ratio           │
│                                                    │
│  [Apply All]  [Apply Compatible Only]  [Cancel]  │
│                                                    │
└──────────────────────────────────────────────────┘
```

**Performance:** Preview wird per GPU im Hintergrund berechnet. Die ersten 6 Previews sofort, Rest on-demand beim Scrollen.

---

## 5. Match Total Exposure

**Shortcut:** `Cmd+Shift+E` (mit Auswahl)
**Funktion:** Gleicht die Gesamtbelichtung aller selektierten Fotos an das aktive Foto an.

**Algorithmus:**
1. Berechne für jedes Foto die mittlere Luminanz nach Entwicklung
2. Berechne die Differenz zum aktiven Foto
3. Kompensiere über den Exposure-Slider
4. Berücksichtige bestehende Tone-Adjustments

**Use Case:** Zeitraffer, Panorama-Reihen, ungleichmässig belichtete Serien.

---

## 6. Edit Transfer History

Die letzten 10 Copy/Paste-Operationen werden gespeichert:

**Shortcut:** `Cmd+Shift+H` → Edit Transfer History

```
┌─────────────────────────────────────────────────┐
│  Edit Transfer History                     [X]  │
├─────────────────────────────────────────────────┤
│                                                   │
│  1. 14:23 — DSC_4523 → 47 Fotos                │
│     Modules: WB, Tone, HSL                      │
│     [Re-Apply]  [Copy Settings]                 │
│                                                   │
│  2. 14:15 — DSC_4480 → 23 Fotos                │
│     Modules: ALL                                │
│     [Re-Apply]  [Copy Settings]                 │
│                                                   │
│  3. 13:58 — DSC_4399 → 1 Foto                  │
│     Modules: Crop only                          │
│     [Re-Apply]  [Copy Settings]                 │
│                                                   │
│  ...                                             │
└─────────────────────────────────────────────────┘
```

---

## 7. Undo für Batch-Operationen

Jede Batch-Operation (Paste auf N Fotos) wird als EINE Undo-Aktion behandelt:

- `Cmd+Z` → alle N Fotos werden zurückgesetzt
- Im History-Panel: "Pasted WB, Tone, HSL on 47 photos" als ein Eintrag
- Einzelne Fotos können trotzdem einzeln zurückgesetzt werden
