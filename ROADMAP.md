# OpenClaw Photo Studio — Roadmap

> Version: 0.1.0-draft | 2026-03-19

---

# Phasen-Übersicht

```
Phase 0: Research           Monat 1-2       ████░░░░░░░░░░░░░░░░
Phase 1: Foundations        Monat 3-5       ░░░░████░░░░░░░░░░░░
Phase 2: Library MVP        Monat 6-8       ░░░░░░░░████░░░░░░░░
Phase 3: Develop MVP        Monat 9-12      ░░░░░░░░░░░░████░░░░
Phase 4: Compatibility      Monat 13-15     ░░░░░░░░░░░░░░░░███░
Phase 5: Power User         Monat 16-18     ░░░░░░░░░░░░░░░░░░██
Phase 6: Plugin SDK         Monat 19-21     ░░░░░░░░░░░░░░░░░░░█
Phase 7: Studio / Pro       Monat 22-24     ░░░░░░░░░░░░░░░░░░░░█
Phase 8: AI Layer           Monat 25-30     ░░░░░░░░░░░░░░░░░░░░░██
Phase 9: Enterprise / OEM   Monat 30+       ░░░░░░░░░░░░░░░░░░░░░░░█
```

---

## Phase 0 — Research (Monat 1-2)

### Ziele
- Technische Machbarkeit validieren
- RAW-Pipeline-Prototyp erstellen
- GPU-Pipeline Proof-of-Concept
- Lizenzstrategie finalisieren
- Wettbewerbsanalyse abschliessen

### Deliverables
- [ ] Prototyp: RAW-Datei (CR3/ARW/DNG) → GPU-Demosaic → Bildschirm-Output
- [ ] Benchmark: GPU vs. CPU Performance für Basis-Pipeline
- [ ] Evaluation: wgpu Kompatibilität auf macOS/Windows/Linux
- [ ] Evaluation: Tauri v2 für Desktop-App
- [ ] Evaluation: SolidJS vs. Alternativen
- [ ] Analyse: XMP-Spezifikation und Lightroom-Sidecar-Format
- [ ] Analyse: darktable + RawTherapee Quellcode (GPL — nur Algorithmen studieren, nichts kopieren)
- [ ] Analyse: LensFun-Integration
- [ ] Lizenztext: PolyForm Noncommercial finalisiert
- [ ] CLA-Vorlage erstellt
- [ ] Projekt-Website: Landing Page (Coming Soon)
- [ ] Governance-Dokument v1

### Risiken
| Risiko | Wahrscheinlichkeit | Mitigation |
|--------|--------------------| -----------|
| wgpu zu unreif für Produktion | Niedrig | Fallback: gfx-hal oder native Metal/Vulkan |
| RAW-Decoding-Qualität unzureichend | Mittel | rawloader evaluieren, LibRaw FFI als Fallback |
| Tauri v2 nicht stabil genug | Niedrig | Electron als Fallback (ungern) |
| Kein Team gefunden | Mittel | Solo MVP möglich, dann Community aufbauen |

### Teamrollen
- 1× Lead Engineer (Rust + GPU)
- 0.5× Researcher (Color Science, RAW-Formate)

### Erfolgskriterien
- [ ] Prototyp zeigt ein RAW-Foto in korrekten Farben auf dem Bildschirm
- [ ] GPU-Pipeline rendet Basis-Adjustments in <16ms
- [ ] XMP-Parser liest Lightroom-Sidecar korrekt
- [ ] Lizenz + Governance steht

### Exit-Kriterien
- Technische Machbarkeit bestätigt: Ja/Nein
- Go/No-Go-Entscheidung für Phase 1

---

## Phase 1 — Foundations (Monat 3-5)

### Ziele
- Monorepo aufsetzen
- CI/CD für alle Plattformen
- Core-Engine-Architektur implementieren
- SQLite-Katalog
- GPU-Pipeline-Framework

### Deliverables
- [ ] GitHub Repository mit Monorepo-Struktur
- [ ] CI/CD: Build + Test auf macOS, Windows, Linux
- [ ] `ocps-core`: RAW-Decode für DNG, CR3, ARW, NEF, RAF
- [ ] `ocps-core`: Demosaic (Bilinear + AMaZE)
- [ ] `ocps-core`: GPU-Pipeline mit wgpu (Exposure, WB, Contrast Shaders)
- [ ] `ocps-catalog`: SQLite Schema v1, Migrations-System
- [ ] `ocps-xmp`: XMP-Reader (Adobe-kompatibel)
- [ ] `ocps-xmp`: EXIF-Reader
- [ ] Cache-System (L2 RAM, L3 Disk)
- [ ] Tauri v2 App-Shell (leeres Fenster, IPC funktioniert)
- [ ] SolidJS Setup mit TailwindCSS
- [ ] Dark Theme Grundstruktur
- [ ] Shortcut-Engine (Basis)
- [ ] LICENSE, CONTRIBUTING.md, GOVERNANCE.md, CLA
- [ ] README.md mit Vision und Build-Anleitung

### Risiken
| Risiko | Wahrscheinlichkeit | Mitigation |
|--------|--------------------| -----------|
| CR3-Parser komplexer als erwartet | Mittel | ISOBMFF-Bibliothek nutzen |
| X-Trans Demosaic (Fuji) schwierig | Hoch | Fuji-Support auf Phase 2 verschieben |
| GPU-Shader-Debugging aufwändig | Mittel | Ausgiebiges Logging, CPU-Vergleich |

### Teamrollen
- 1× Core Engine (Rust)
- 1× UI/Frontend (TypeScript)
- 0.5× DevOps/CI

### Erfolgskriterien
- [ ] 5 RAW-Formate werden korrekt dekodiert
- [ ] GPU-Pipeline rendert Exposure + WB korrekt
- [ ] Katalog speichert und liest 10'000 Einträge in <1s
- [ ] XMP-Sidecars von Lightroom werden korrekt gelesen
- [ ] App startet auf allen 3 Plattformen

### Exit-Kriterien
- Core-Pipeline funktioniert End-to-End
- Mindestens 3 RAW-Formate unterstützt
- App-Shell zeigt ein RAW-Foto an

---

## Phase 2 — Library MVP (Monat 6-8)

### Ziele
- Funktionale Bibliotheksverwaltung
- Import-Engine
- Grid + Loupe View
- Rating/Flagging/Labels
- Filmstrip

### Deliverables
- [ ] Import Engine: Copy, Move, Add-in-Place
- [ ] Import: Rename-Templates
- [ ] Import: Duplikat-Erkennung
- [ ] Grid View mit virtualisierter Thumbnail-Liste (10k+ flüssig)
- [ ] Loupe View mit GPU-Rendering
- [ ] Filmstrip
- [ ] Rating (0-5), Flagging (P/X/U), Color Labels (6-9)
- [ ] Auto-Advance nach Rating
- [ ] Ordner-Panel
- [ ] Collections (manuell)
- [ ] Quick Collection (B-Taste)
- [ ] Keyword-Panel (Basis)
- [ ] EXIF-Info-Panel
- [ ] Sort & Filter (Rating, Flag, Color, Date)
- [ ] Freitext-Suche (FTS5)
- [ ] Thumbnail-Generierung (Background)
- [ ] Preview-Cache (L3 Disk)
- [ ] Histogram (Basis)
- [ ] Before/After (Toggle \)
- [ ] Alle Lightroom-kompatiblen Library-Shortcuts

### Risiken
| Risiko | Wahrscheinlichkeit | Mitigation |
|--------|--------------------| -----------|
| Grid-Performance bei 100k+ Fotos | Mittel | Virtualisierung von Anfang an, Benchmark früh |
| Thumbnail-Qualität (Farben) | Niedrig | Embedded JPEG aus RAW als Fallback |
| Import-Geschwindigkeit | Niedrig | Parallel EXIF-Extraction |

### Teamrollen
- 1× Core Engine
- 1× UI/Frontend
- 0.5× QA/Testing

### Erfolgskriterien
- [ ] 1'000 RAW-Dateien importieren in <30s
- [ ] Grid scrollt flüssig (60fps) bei 50'000 Thumbnails
- [ ] Rating/Flagging per Tastatur funktioniert fehlerfrei
- [ ] Freitext-Suche liefert Ergebnisse in <100ms

### Exit-Kriterien
- Ein Fotograf kann Fotos importieren, sichten, bewerten und filtern
- Vergleichbar mit Lightroom Library Modul (Basis-Funktionen)

---

## Phase 3 — Develop MVP (Monat 9-12)

### Ziele
- Vollständige nicht-destruktive RAW-Entwicklung
- Alle Basis-Adjustments
- Tone Curve, HSL, Color Grading
- Detail (Sharpening, NR)
- Crop & Rotate
- Copy/Paste Edits
- Before/After
- History + Snapshots
- Export (JPEG, TIFF)

### Deliverables
- [ ] Develop View (Panel-Layout wie Lightroom)
- [ ] GPU-Shaders für alle Basis-Adjustments
- [ ] White Balance (Temperature, Tint, Picker, Presets)
- [ ] Exposure, Contrast, Highlights, Shadows, Whites, Blacks
- [ ] Clarity, Dehaze, Vibrance, Saturation
- [ ] Tone Curve (Parametric + Point Curve)
- [ ] HSL / Color Mixer (8 Kanäle)
- [ ] Color Grading (3-Way + Global)
- [ ] Sharpening (Amount, Radius, Detail, Masking)
- [ ] Noise Reduction (Luminance + Color)
- [ ] Crop & Rotate (Aspect Ratios, Straighten)
- [ ] Post-Crop Vignette
- [ ] Grain
- [ ] Calibration
- [ ] History Panel (Voller Undo-Stack)
- [ ] Snapshots (benannt)
- [ ] Before/After (4 Modi: Toggle, Side-by-Side, Split, Top/Bottom)
- [ ] Copy All Settings (Cmd+C)
- [ ] Paste Settings (Cmd+V)
- [ ] Copy Selected (Cmd+Shift+C)
- [ ] Paste Selected (Cmd+Shift+V)
- [ ] Sync Settings (Multi-Select)
- [ ] Auto-Sync Mode
- [ ] Virtual Copies
- [ ] Export: JPEG (Qualitätsstufe, Resize, Metadaten)
- [ ] Export: TIFF (8/16-bit)
- [ ] Export: PNG
- [ ] Export Presets
- [ ] Batch Export (Background)
- [ ] XMP-Sidecar schreiben (Adobe-kompatibel)
- [ ] Slider-Interaktion (Drag, Double-Click Reset, Scroll, Shift+Drag, Direct Input)
- [ ] Slider Masking Preview (Alt+Drag)
- [ ] Alle Develop-Shortcuts (Lightroom-kompatibel)
- [ ] Command Palette (Cmd+K)

### Risiken
| Risiko | Wahrscheinlichkeit | Mitigation |
|--------|--------------------| -----------|
| GPU-Shader-Bugs (Farbartefakte) | Hoch | CPU-Vergleichs-Pipeline, Golden Image Tests |
| Noise Reduction Qualität | Mittel | Wavelet+NLM Hybrid, iterativ verbessern |
| Copy/Paste Complexity | Niedrig | Klares Datenmodell, Unit Tests |
| Export-Determinismus | Mittel | Deterministischer Seed, Golden Image Tests |

### Teamrollen
- 2× Core Engine (Rust, Shaders)
- 1× UI/Frontend
- 1× Color Science (Beratung)
- 0.5× QA

### Erfolgskriterien
- [ ] Slider-Änderung → Preview in <16ms (60fps)
- [ ] Volle RAW-Entwicklung funktioniert für Top-5-Kameramarken
- [ ] Copy/Paste auf 100 Fotos in <500ms
- [ ] Export 100 JPEG in <60s
- [ ] XMP-Roundtrip: OCPS → XMP → Lightroom liest korrekt
- [ ] Golden Image Tests bestehen für alle Module
- [ ] Fotografen-Feedback: "Fühlt sich wie Lightroom an"

### Exit-Kriterien
- Ein Fotograf kann ein komplettes Shooting bearbeiten und exportieren
- RAW-Qualität ist "professionell nutzbar" (nicht perfekt, aber gut)
- Copy/Paste funktioniert zuverlässig

---

## Phase 4 — Compatibility Layer (Monat 13-15)

### Ziele
- Lightroom-Katalog-Import
- Erweiterte XMP-Kompatibilität
- Lightroom-Preset-Import
- Lightroom-Keymap als Default
- Kompatibilitäts-Dokumentation und -Tests

### Deliverables
- [ ] Lightroom Catalog Import (.lrcat → OCPS)
- [ ] Import: Fotos, Ratings, Flags, Labels, Keywords, Collections
- [ ] Import: Smart Collections (Regel-Mapping)
- [ ] Import: Virtual Copies
- [ ] Import: Stacks
- [ ] Migration Report (was importiert wurde, Warnungen)
- [ ] Lightroom Preset Import (.lrtemplate + .xmp Presets)
- [ ] Preset-Kompatibilitäts-Matrix (was funktioniert, was nicht)
- [ ] XMP: Process Version PV2012 Support
- [ ] XMP: Local Adjustments Import (Brush, Gradient, Radial)
- [ ] XMP: Unbekannte Felder erhalten
- [ ] Kompatibilitäts-Level-System (Level 1-4)
- [ ] COMPATIBILITY.md Dokumentation
- [ ] XMP Compatibility Test Suite (50+ Testdateien)
- [ ] Roundtrip-Tests (OCPS → XMP → Lightroom → XMP → OCPS)
- [ ] DNG-Support (Lesen + Schreiben)

### Risiken
| Risiko | Wahrscheinlichkeit | Mitigation |
|--------|--------------------| -----------|
| Lightroom-Katalog-Schema undokumentiert | Hoch | Reverse Engineering, Community-Wissen |
| Preset-Kompatibilität lückenhaft | Hoch | Best-Effort, klare Dokumentation |
| DCP-Profile nicht replizierbar | Mittel | ICC als Alternative, DCP Best-Effort |

### Teamrollen
- 1× Compatibility Engineer
- 1× Core Engine
- 1× QA (Kompatibilitäts-Tests)

### Erfolgskriterien
- [ ] Lightroom-Katalog mit 10'000 Fotos wird korrekt importiert
- [ ] 80% der Lightroom-Presets funktionieren (Basis-Settings)
- [ ] XMP-Roundtrip verliert keine Daten
- [ ] Migration Report ist verständlich und vollständig

### Exit-Kriterien
- Ein Lightroom-Nutzer kann seinen Katalog migrieren und weiterarbeiten

---

## Phase 5 — Power User Workflow (Monat 16-18)

### Ziele
- Local Adjustments (Masken)
- Lens Corrections
- Transform/Upright
- Smart Collections
- Stacking
- Soft Proofing
- Vim-Mode
- Erweiterte Export-Optionen

### Deliverables
- [ ] Local Adjustments: Adjustment Brush
- [ ] Local Adjustments: Graduated Filter
- [ ] Local Adjustments: Radial Filter
- [ ] Range Mask: Luminance + Color
- [ ] Lens Corrections (LensFun-Integration)
- [ ] Transform / Upright (manuell + Auto)
- [ ] Chromatic Aberration Removal
- [ ] Smart Collections (Query Builder)
- [ ] Stacking (Auto-Stack by Time, Manual)
- [ ] Soft Proofing (ICC-Profile)
- [ ] Vim-Mode (Normal, Command, Visual)
- [ ] Compare Mode (Sync-Zoom, Sync-Pan)
- [ ] Survey Mode
- [ ] Export: WebP, AVIF
- [ ] Export: Output Sharpening
- [ ] Export: Watermarking
- [ ] Export: Naming Templates
- [ ] Second Monitor Support
- [ ] Keyword Hierarchies (vollständig)
- [ ] Keyword Sets
- [ ] Batch Metadata Editing
- [ ] Monochrome / B&W Mix

### Risiken
| Risiko | Wahrscheinlichkeit | Mitigation |
|--------|--------------------| -----------|
| Masken-Performance (viele Masken) | Mittel | GPU-Compositing, Mask-Caching |
| Auto-Upright Qualität | Mittel | Iterativ verbessern, manuell als Fallback |
| Vim-Mode Komplexität | Niedrig | Sauber modular, gute Tests |

### Teamrollen
- 2× Core Engine
- 1× UI/Frontend
- 1× QA

### Erfolgskriterien
- [ ] 20 lokale Masken pro Foto bei 60fps
- [ ] LensFun deckt 80% der populären Objektive ab
- [ ] Vim-Mode funktioniert für kompletten Workflow
- [ ] Power-User-Feedback: "Schneller als Lightroom"

### Exit-Kriterien
- Feature-Parity mit Lightroom (Basis + Erweitert) erreicht

---

## Phase 6 — Plugin SDK (Monat 19-21)

### Ziele
- Stabiles Plugin-System (v1.0)
- Plugin-API dokumentiert
- Beispiel-Plugins
- Plugin-Distribution

### Deliverables
- [ ] Plugin Host (wasmtime)
- [ ] Plugin API v1 (stabil, versioniert)
- [ ] Plugin Manifest Format
- [ ] Plugin Sandbox (Permissions)
- [ ] Plugin Discovery (lokaler Scan)
- [ ] Plugin Dev Guide (Dokumentation)
- [ ] Plugin Template (Starter-Projekt)
- [ ] Beispiel-Plugin: Custom LUT Loader
- [ ] Beispiel-Plugin: FTP Upload
- [ ] Beispiel-Plugin: Simple AI Tagger (ONNX)
- [ ] Plugin CLI: `ocps-plugin create`, `ocps-plugin test`, `ocps-plugin build`
- [ ] Plugin-Signing (optional, für Marketplace)
- [ ] Plugin UI Integration (Custom Panels)

### Risiken
| Risiko | Wahrscheinlichkeit | Mitigation |
|--------|--------------------| -----------|
| WASM Performance zu langsam | Niedrig | Shared Memory, keine Kopien |
| API-Design zu eng/weit | Mittel | Beta-Feedback von Plugin-Autoren |
| Sandbox-Escape möglich | Niedrig | Security-Audit, wasmtime ist robust |

### Teamrollen
- 1× Plugin System Engineer
- 1× Developer Relations / Docs
- 0.5× Security

### Erfolgskriterien
- [ ] 3+ funktionierende Plugins von externen Entwicklern
- [ ] Plugin-API ist stabil und dokumentiert
- [ ] Sandbox schützt vor bösartigen Plugins

### Exit-Kriterien
- Plugin SDK veröffentlicht und von Community angenommen

---

## Phase 7 — Studio / Pro Workflows (Monat 22-24)

### Ziele
- Tethered Shooting (Plugin)
- Session Mode
- HDR Merge (Plugin)
- Panorama Merge (Plugin)
- Print Module
- Map Module
- Client Review

### Deliverables
- [ ] Tethered Shooting Plugin (gPhoto2-basiert)
- [ ] Session Mode (Shooting-Session mit Auto-Import)
- [ ] Live Ingest (neue Fotos sofort anzeigen)
- [ ] Backup on Import (zweites Ziel)
- [ ] HDR Merge Plugin
- [ ] Panorama Merge Plugin
- [ ] Map Module (OpenStreetMap, GPS-Fotos)
- [ ] GPX-Track-Import
- [ ] Print Module (Layouts, Contact Sheets)
- [ ] Print: Soft Proofing
- [ ] Contact Sheet Export (PDF)
- [ ] Client Review Mode (vereinfacht)
- [ ] Healing / Clone Tool (Spot Removal)

### Risiken
| Risiko | Wahrscheinlichkeit | Mitigation |
|--------|--------------------| -----------|
| Tethering-Kompatibilität (Kameras) | Hoch | gPhoto2 deckt vieles ab, Canon/Nikon SDK optional |
| HDR-Qualität (Deghosting) | Mittel | Iterativ, Open-Source-Referenzen |
| Panorama-Stitching-Qualität | Hoch | Externe Engine möglich (OpenCV via Plugin) |

### Teamrollen
- 2× Core Engine
- 1× UI/Frontend
- 1× Plugin Developer
- 1× QA

### Erfolgskriterien
- [ ] Tethering funktioniert mit Canon, Nikon, Sony
- [ ] HDR Merge liefert brauchbare Ergebnisse
- [ ] Map Module zeigt GPS-Fotos korrekt

### Exit-Kriterien
- Studio-Fotografen können OCPS als Haupttool verwenden

---

## Phase 8 — AI Layer (Monat 25-30)

### Ziele
- AI-basierte Features als Plugins
- Lokale Inferenz (kein Cloud-Zwang)
- Optionale Cloud-AI-Integration

### Deliverables
- [ ] AI Denoise Plugin (ONNX-basiert)
- [ ] AI Subject Masking (SAM oder vergleichbar)
- [ ] AI Sky Masking
- [ ] Face Detection + Recognition Plugin
- [ ] Auto Keywording Plugin
- [ ] Scene Detection Plugin
- [ ] Semantic Search
- [ ] Dust Spot Detection
- [ ] Blink/Closed-Eye Detection
- [ ] Smart Preset Recommendation
- [ ] Natural Language Command Palette (optional, LLM)
- [ ] AI Sharpness Estimation (Culling-Hilfe)

### Risiken
| Risiko | Wahrscheinlichkeit | Mitigation |
|--------|--------------------| -----------|
| Modell-Grösse (Download) | Hoch | Optionale Downloads, mehrere Modellgrössen |
| Inferenz-Geschwindigkeit | Mittel | GPU-Inferenz, quantisierte Modelle |
| Qualität vs. proprietäre Lösungen | Hoch | Open-Source-Modelle werden besser, Community-Training |

### Teamrollen
- 1× ML Engineer
- 1× Core Engine
- 1× QA

### Erfolgskriterien
- [ ] AI Denoise vergleichbar mit Lightroom AI NR
- [ ] Subject Masking IoU >85%
- [ ] Face Detection Precision >95%
- [ ] Alle AI-Features lokal lauffähig (kein Cloud-Zwang)

### Exit-Kriterien
- AI-Features sind "nice to have", kein Blocker für Nutzung

---

## Phase 9 — Enterprise / OEM Licensing (Monat 30+)

### Ziele
- Kommerzielle Lizenz operationalisieren
- Enterprise-Features
- OEM-API
- Support-Infrastruktur

### Deliverables
- [ ] Kommerzielle Lizenz-Verträge (Templates)
- [ ] Licensing Service (License Key Validation, optional)
- [ ] Enterprise Features:
  - Multi-User-Katalog
  - Netzwerk-Storage-Support
  - LDAP/SSO Integration
  - Audit Trail
  - Zentrale Preset-Verwaltung
- [ ] OEM API: Headless Processing Engine
- [ ] OEM: Custom Branding API
- [ ] OEM: Embedded SDK
- [ ] Support-Portal
- [ ] SLA-Verträge
- [ ] Training / Zertifizierung
- [ ] Marketplace Launch (Plugins + Presets)
- [ ] Marketplace: Revenue Sharing (15-30%)

### Risiken
| Risiko | Wahrscheinlichkeit | Mitigation |
|--------|--------------------| -----------|
| Kein Enterprise-Interesse | Mittel | Erst bauen wenn Nachfrage besteht |
| OEM-Integration komplex | Mittel | Saubere API von Anfang an |
| Support-Last | Hoch | Community-First, dann Paid Support |

### Teamrollen
- 1× Business Development
- 1× Enterprise Engineer
- 1× Support

### Erfolgskriterien
- [ ] Erster kommerzieller Lizenzvertrag unterzeichnet
- [ ] OEM-SDK in einem Produkt integriert
- [ ] Marketplace hat 50+ Plugins/Presets

### Exit-Kriterien
- Projekt ist finanziell nachhaltig
