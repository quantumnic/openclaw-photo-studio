# OpenClaw Photo Studio — Executive Summary

> 2026-03-19 | Final Document

---

## 1. Executive Summary

OpenClaw Photo Studio (OCPS) ist ein source-available Foto-Workflow-Tool, das Adobe Lightroom funktional nachbildet — mit eigener Codebasis, eigenem Branding und eigener technischer Architektur. Zielgruppe: professionelle und ambitionierte Fotografen, die Lightroom-Qualität ohne Abo-Zwang, Cloud-Abhängigkeit und Vendor-Lock-in wollen.

**Kernversprechen:**
- Lightroom-vertrauter Workflow (Import → Sichten → Entwickeln → Exportieren)
- GPU-beschleunigte RAW-Entwicklung (<16ms Slider-Response)
- Keyboard-First (jede Aktion per Shortcut, Vim-Mode optional)
- Copy/Paste von Edits in 2 Tasten auf 100+ Fotos
- XMP-Sidecar-kompatibel (Adobe-Format lesen/schreiben)
- Nicht-destruktiv, lokal, erweiterbar (Plugin-System)

**Lizenz:** PolyForm Noncommercial 1.0.0 (frei für Privatnutzung, kostenpflichtig für kommerzielle Einbettung). Dual-License-Ready.

**Tech-Stack:** Rust Core + wgpu GPU-Pipeline + Tauri v2 Desktop + SolidJS UI.

**Timeline:** ~24 Monate bis v1.0 Release, MVP (Library + Develop) in 12 Monaten.

---

## 2. MVP-Definition (Minimum Viable Product)

Das MVP ermöglicht einem Fotografen, ein komplettes Shooting zu importieren, zu sichten, zu bewerten, zu entwickeln und zu exportieren.

### MVP-Scope (Phase 0-3, Monat 1-12):

| Feature | Status |
|---------|--------|
| RAW-Decode (DNG, CR3, ARW, NEF, RAF) | Core |
| GPU-Processing-Pipeline (Basis) | Core |
| Import (Copy/Move/Add) mit Rename-Templates | Core |
| SQLite-Katalog | Core |
| Grid View (virtualisiert, 50k+ flüssig) | Core |
| Loupe View (GPU-Rendering) | Core |
| Rating (0-5), Flags (P/X/U), Color Labels | Core |
| Filmstrip | Core |
| Collections (manuell) | Core |
| Keyword-Panel (Basis) | Core |
| EXIF-Info-Panel | Core |
| Sort & Filter | Core |
| Freitext-Suche (FTS5) | Core |
| Histogram | Core |
| Develop: WB, Exposure, Contrast, H/S/W/B | Core |
| Develop: Clarity, Dehaze, Vibrance, Saturation | Core |
| Develop: Tone Curve (Parametric + Point) | Core |
| Develop: HSL / Color Mixer | Core |
| Develop: Color Grading | Core |
| Develop: Sharpening + Noise Reduction | Core |
| Develop: Crop & Rotate | Core |
| Develop: Vignette + Grain | Core |
| Before/After (4 Modi) | Core |
| History + Snapshots | Core |
| Copy/Paste All (Cmd+C/V) | Core |
| Copy/Paste Selected (Cmd+Shift+C/V) | Core |
| Sync Settings (Multi-Select) | Core |
| Auto-Sync Mode | Core |
| Virtual Copies | Core |
| Export: JPEG, TIFF, PNG | Core |
| Export Presets | Core |
| Batch Export (Background) | Core |
| XMP-Sidecar lesen (Adobe-kompatibel) | Core |
| XMP-Sidecar schreiben | Core |
| Shortcut-Engine (Lightroom-kompatibel) | Core |
| Command Palette (Cmd+K) | Core |
| Dark Theme | Core |

### Explizit NICHT im MVP:
- Local Adjustments (Masken)
- Lens Corrections
- Transform/Upright
- Smart Collections
- Plugin-System
- Map/Print Module
- AI-Features
- Tethering
- Lightroom-Katalog-Import

---

## 3. Top 25 Features (priorisiert)

| # | Feature | Phase | Impact |
|---|---------|-------|--------|
| 1 | GPU-accelerated RAW Pipeline | MVP | Kernfunktion |
| 2 | Import Engine | MVP | Kernfunktion |
| 3 | Library (Grid + Loupe) | MVP | Kernfunktion |
| 4 | Non-destructive Develop (alle Basis-Module) | MVP | Kernfunktion |
| 5 | Copy/Paste Edits (2-Tasten-Workflow) | MVP | Killer-Feature |
| 6 | XMP Sidecar Read/Write | MVP | Lightroom-Kompatibilität |
| 7 | Keyboard-First Shortcuts | MVP | Differenzierung |
| 8 | Batch Export | MVP | Workflow-Abschluss |
| 9 | Rating/Flagging/Labels per Keyboard | MVP | Culling-Speed |
| 10 | Before/After Comparison | MVP | Qualitätskontrolle |
| 11 | Local Adjustments (Brush/Gradient/Radial) | V1.0 | Pro-Feature |
| 12 | Lens Corrections (LensFun) | V1.0 | Qualität |
| 13 | Smart Collections | V1.0 | Organisation |
| 14 | Lightroom Catalog Import | V1.0 | Migration |
| 15 | Lightroom Preset Import | V1.0 | Migration |
| 16 | Plugin System (WASM) | V1.0 | Erweiterbarkeit |
| 17 | Command Palette | MVP | Power-User |
| 18 | Vim-Mode | V1.5 | Power-User |
| 19 | Compare + Survey Mode | V1.0 | Culling |
| 20 | CLI Export (Headless) | V1.0 | Automation |
| 21 | Soft Proofing | V1.5 | Print-Workflow |
| 22 | Tethered Shooting (Plugin) | V2.0 | Studio |
| 23 | HDR Merge (Plugin) | V2.0 | Spezial-Workflow |
| 24 | AI Denoise (Plugin) | V2.0 | Qualität |
| 25 | AI Subject Masking (Plugin) | V2.0 | Produktivität |

---

## 4. Empfohlene Lizenzstrategie

```
┌─────────────────────────────────────────────────────────────┐
│                    LIZENZSTRATEGIE                            │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  JETZT (Launch):                                             │
│  ├── PolyForm Noncommercial 1.0.0 (Community)               │
│  └── Commercial License Agreement (OEM/SaaS/Enterprise)     │
│                                                               │
│  SPÄTER (bei Traktion, optional):                            │
│  ├── Business Source License (BSL 1.1)                       │
│  │   └── 36-Monats-Conversion → Apache 2.0                  │
│  └── Marketplace Revenue Share (Plugins/Presets)             │
│                                                               │
│  CLA: Ja (non-exclusive, mit Fairness-Klausel)              │
│  Trademark: Registrierung empfohlen                          │
│  Kommunikation: "source-available", NICHT "open source"     │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

**Begründung:**
- PolyForm NC ist kurz, rechtssicher, klar → idealer Start
- CLA sichert Dual-Licensing-Rechte
- BSL als Upgrade-Pfad wenn nötig (gibt Contributors Vertrauen durch zeitverzögerte Freigabe)
- Kein Feature-Gating — Community-Version hat vollen Funktionsumfang

---

## 5. Empfohlene Architektur

```
┌─────────────────────────────────────────────────────────┐
│              ARCHITEKTUR-ENTSCHEIDUNG                     │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  Gewählt: Rust Core + wgpu GPU + Tauri v2 + SolidJS    │
│                                                           │
│  Warum nicht:                                            │
│  ├── C++/Qt: Memory-Unsicherheit, Build-Komplexität     │
│  ├── Electron: 150-300MB App, hoher RAM-Verbrauch        │
│  ├── Native pro Plattform: 3× Code, 3× Maintenance     │
│  ├── Rust+Qt: Qt-Licensing-Probleme, C++-FFI-Overhead   │
│  └── Pure WASM/Web: Kein GPU-Zugang, File-API limitiert │
│                                                           │
│  Stack:                                                  │
│  ├── Core:     Rust (rawloader, wgpu, rusqlite, tokio)  │
│  ├── GPU:      wgpu + WGSL Shaders                      │
│  ├── Desktop:  Tauri v2 (5-15MB, native WebView)        │
│  ├── UI:       SolidJS + TailwindCSS                    │
│  ├── Plugins:  WASM (wasmtime)                          │
│  ├── DB:       SQLite (rusqlite)                        │
│  ├── IPC:      Tauri Commands (type-safe)               │
│  └── Formats:  XMP, IPTC, EXIF, ICC, DNG               │
│                                                           │
│  Performance-Ziele:                                      │
│  ├── Slider → Preview: <16ms (60fps)                    │
│  ├── Foto wechseln: <100ms                              │
│  ├── App-Start: <2s                                      │
│  ├── Import 1'000 RAW: <30s                             │
│  ├── Export 100 JPEG: <60s                              │
│  └── Grid Scroll (100k): 60fps                          │
│                                                           │
└─────────────────────────────────────────────────────────┘
```

---

## 6. Die ersten 90 Tage (Detailplan)

### Tag 1-14: Projekt-Bootstrap

| Tag | Aufgabe |
|-----|---------|
| 1-2 | GitHub-Repo erstellen, Monorepo-Struktur (Cargo + pnpm Workspace) |
| 3 | CI/CD: GitHub Actions für macOS, Windows, Linux (Build + Test) |
| 4 | Lizenz-Dateien: LICENSE, LICENSE-POLYFORM.md, COMMERCIAL.md |
| 5 | Governance: CONTRIBUTING.md, GOVERNANCE.md, CODE_OF_CONDUCT.md, CLA.md |
| 6-7 | CLA-Bot einrichten (CLA Assistant), README.md |
| 8-10 | Rust Workspace Setup: ocps-core, ocps-catalog, ocps-xmp, ocps-export |
| 11-12 | Tauri v2 App-Shell: Leeres Fenster, IPC-Grundstruktur |
| 13-14 | SolidJS + TailwindCSS Setup, Dark Theme Grundstruktur |

**Deliverable:** Repo steht, CI baut auf 3 Plattformen, App-Fenster öffnet sich.

### Tag 15-35: RAW-Pipeline Prototyp

| Tag | Aufgabe |
|-----|---------|
| 15-18 | RAW-Decode: DNG-Parser (rawloader evaluieren + integrieren) |
| 19-22 | RAW-Decode: CR3 (Canon) und ARW (Sony) |
| 23-25 | Demosaicing: Bilinear (schnell, für Thumbnails) + AMaZE (Qualität) |
| 26-28 | wgpu Setup: GPU-Kontext, erster Compute Shader (Exposure) |
| 29-31 | GPU Shaders: White Balance, Contrast |
| 32-33 | GPU Output: Render-to-Screen via Tauri WebView |
| 34-35 | Cache: L3 Disk-Cache für Thumbnails |

**Deliverable:** App zeigt ein RAW-Foto an, Exposure + WB-Slider funktionieren.

### Tag 36-50: Katalog + XMP

| Tag | Aufgabe |
|-----|---------|
| 36-38 | SQLite Schema v1 (Photos, Collections, Keywords, Edits) |
| 39-40 | Migration-System (nummerierte SQL-Dateien) |
| 41-43 | XMP-Reader: Adobe-kompatibles Parsing (crs: Namespace) |
| 44-45 | XMP-Reader: IPTC + Dublin Core |
| 46-47 | EXIF-Parser: Integration (kamadak-exif) |
| 48-50 | Import-Engine: Ordner scannen → EXIF lesen → Katalog-Einträge erstellen → Thumbnails generieren |

**Deliverable:** 1'000 RAW-Dateien importieren, EXIF + XMP korrekt gelesen.

### Tag 51-70: Library UI

| Tag | Aufgabe |
|-----|---------|
| 51-54 | Grid View: Virtualisierte Thumbnail-Liste (VirtualList-Komponente) |
| 55-57 | Loupe View: Einzelbild mit GPU-Rendering |
| 58-59 | Filmstrip: Unterer Balken mit Thumbnail-Strip |
| 60-62 | Rating/Flagging/Labels: 0-5, P/X/U, 6-9 per Tastatur |
| 63-64 | Auto-Advance nach Rating |
| 65-66 | Sort & Filter: Rating, Flag, Date |
| 67-68 | Shortcut-Engine: Grundframework + Lightroom-Default-Keybindings |
| 69-70 | Panel-System: Collapsible Sidebars, Solo-Mode |

**Deliverable:** Fotografen können Fotos importieren, in Grid/Loupe sichten und bewerten.

### Tag 71-90: Develop MVP

| Tag | Aufgabe |
|-----|---------|
| 71-73 | Develop View Layout: Panels rechts, Foto in der Mitte |
| 74-75 | Slider-Komponente: Drag, Double-Click Reset, Shift+Drag, Scroll |
| 76-78 | GPU Shaders: Highlights, Shadows, Whites, Blacks, Clarity, Dehaze |
| 79-80 | GPU Shaders: Vibrance, Saturation |
| 81-82 | Tone Curve UI + Shader (Parametric) |
| 83-84 | HSL Panel + Shader |
| 85 | Edit-Daten speichern (JSON in SQLite) |
| 86-87 | Copy/Paste Edits: Cmd+C/V (alle Settings) |
| 88 | Copy/Paste Selected: Cmd+Shift+C/V (mit Dialog) |
| 89 | Before/After Toggle (\\-Taste) |
| 90 | Histogram |

**Deliverable:** Basis-RAW-Entwicklung funktioniert. Copy/Paste funktioniert. Fotografen-Feedback einholbar.

### Tag 90: Milestone-Review

- [ ] App startet auf macOS, Windows, Linux
- [ ] 3+ RAW-Formate dekodiert
- [ ] GPU-Pipeline rendert alle Basis-Adjustments
- [ ] Import funktioniert (1'000+ Fotos)
- [ ] Grid + Loupe funktioniert
- [ ] Rating/Flagging per Tastatur funktioniert
- [ ] Basis-Develop funktioniert (WB, Exposure, Tone, HSL)
- [ ] Copy/Paste funktioniert
- [ ] XMP-Sidecar wird gelesen
- [ ] Performance: Slider-Response <50ms
- [ ] Golden Image Tests: 3+ bestehen

**Entscheidung:** Go/No-Go für Phase 2 (Library MVP Completion).

---

## Dokument-Übersicht

Das komplette Konzept besteht aus folgenden Dokumenten:

| Datei | Inhalt | Grösse |
|-------|--------|--------|
| `CONCEPT.md` | Produktvision, Lizenz, Governance, Architektur, Features, UX, Kompatibilität, RAW-Pipeline, Plugin-System, Roadmap | ~87 KB |
| `FEATURES.md` | Vollständige Feature-Liste mit Bewertung (100+ Features) | ~38 KB |
| `COPY-PASTE-SPEC.md` | Copy/Paste & Sync Spezifikation | ~10 KB |
| `SHORTCUTS.md` | Vollständige Shortcut-Spezifikation (200+ Shortcuts, Vim-Mode) | ~16 KB |
| `TESTING.md` | Teststrategie (15 Kategorien) | ~10 KB |
| `ROADMAP.md` | 10-Phasen-Roadmap (30+ Monate) | ~17 KB |
| `DATA-MODELS.md` | 16 Datenmodelle (Rust-Structs) | ~32 KB |
| `ADR.md` | 10 Architecture Decision Records | ~11 KB |
| `EXECUTIVE-SUMMARY.md` | Zusammenfassung, MVP, Top 25, 90-Tage-Plan | dieses Dokument |
| `MASTER-PROMPT.md` | Original-Prompt (Auftragsarchiv) | ~3 KB |
| `repo-files/README.md` | Repository README | ~6 KB |
| `repo-files/CONTRIBUTING.md` | Beitragsrichtlinien | ~8 KB |
| `repo-files/GOVERNANCE.md` | Governance-Struktur | ~2 KB |
| `repo-files/LICENSE-CHOICE.md` | Lizenz-Begründung | ~5 KB |
| `repo-files/COMMERCIAL.md` | Kommerzielle Lizenz-Info | ~2 KB |
| `repo-files/CODE_OF_CONDUCT.md` | Verhaltenskodex | ~2 KB |
| `repo-files/SECURITY.md` | Security Policy | ~2 KB |
| `repo-files/COMPATIBILITY.md` | Lightroom-Kompatibilität | ~6 KB |

**Gesamtumfang: ~257 KB / ~18 Dokumente**

---

*Erstellt von Ocean 🌊 — Lead Product Architect, OpenClaw Photo Studio*
