# Architecture Decision Records (ADR)

> OpenClaw Photo Studio | 2026-03-19

---

## ADR-001: Lizenzmodell — PolyForm Noncommercial + Commercial Dual License

### Status: Akzeptiert

### Kontext
Wir brauchen eine Lizenz, die:
- Community-Beiträge und private Nutzung erlaubt
- Kommerzielle Einbettung (OEM, SaaS, White-Label) einschränkt
- Nicht als "Open Source" im OSI-Sinn gilt (um Missverständnisse zu vermeiden)
- Dual-Licensing ermöglicht

### Entscheidung
**PolyForm Noncommercial License 1.0.0** als Standard-Lizenz. Kommerzielle Nutzung erfordert separate Lizenz. Optional späterer Wechsel auf BSL (Business Source License) mit 36-Monats-Conversion.

### Alternativen betrachtet
| Lizenz | Pro | Contra |
|--------|-----|--------|
| MIT | Maximale Freiheit | Amazon-Effekt, keine Kontrolle |
| Apache 2.0 | Patent-Grant | Gleiche Probleme wie MIT |
| GPL-3.0 | Copyleft erzwingt Offenheit | Schreckt kommerzielle Partner ab, komplex |
| AGPL-3.0 | SaaS-Schutz | Noch restriktiver, CLA schwieriger |
| SSPL (MongoDB) | Starker SaaS-Schutz | Nicht OSI-anerkannt, kontrovers |
| BSL | Time-based Conversion | Komplexer, weniger bekannt |
| **PolyForm NC** | **Klar, kurz, rechtssicher** | **Nicht OSI, "source-available"** |

### Konsequenzen
- Projekt wird als "source-available" kommuniziert, nicht als "Open Source"
- CLA erforderlich für Contributors (IP-Übertragung für Dual-Licensing)
- Kommerzielle Kunden brauchen separaten Lizenzvertrag
- Community muss verstehen, warum diese Lizenz gewählt wurde (→ FAQ, Dokumentation)

---

## ADR-002: Monorepo-Struktur

### Status: Akzeptiert

### Kontext
Das Projekt hat mehrere Komponenten: Rust-Crates (Core, Catalog, XMP, Export, Plugin-Host), TypeScript-UI, Desktop-App (Tauri), CLI, Plugins, Presets, Docs, Tests.

### Entscheidung
**Monorepo** mit Cargo Workspace (Rust) + pnpm Workspace (TypeScript).

### Alternativen betrachtet
| Ansatz | Pro | Contra |
|--------|-----|--------|
| **Monorepo** | **Atomare Commits, einfache CI, konsistente Versionen** | **Grösseres Repo, Build-Komplexität** |
| Polyrepo | Unabhängige Releases | Versionierungs-Hölle, Cross-Repo-PRs |
| Hybrid | Flexibel | Komplexeste CI |

### Konsequenzen
- Cargo Workspace für alle Rust-Crates
- pnpm Workspace für TypeScript-Packages
- Ein CI-Workflow baut alles
- Plugins werden als separate Repos empfohlen (aber können im Monorepo starten)
- Git LFS für Test-Fixtures (RAW-Dateien)

---

## ADR-003: Core Engine in Rust

### Status: Akzeptiert

### Kontext
Die Core-Engine (RAW-Decode, Processing Pipeline, Catalog, XMP) muss performant, speichersicher und cross-platform sein.

### Entscheidung
**Rust** für die gesamte Core-Engine.

### Alternativen betrachtet
| Sprache | Pro | Contra |
|---------|-----|--------|
| **Rust** | **Memory-Safe, schnell, WASM-Target, moderne Toolchain** | **Steilere Lernkurve, kleinerer Talentpool** |
| C++ | Maximal performant, Industrie-Standard für Imaging | Memory-Bugs, kein WASM, Build-Systeme |
| C | Einfach, überall verfügbar | Kein Ownership-System, fehleranfällig |
| Go | Einfach, schnell zu entwickeln | GC-Pausen, kein WASM für Desktop |
| Zig | Modern, C-Interop | Zu jung, kleines Ökosystem |

### Konsequenzen
- Gesamte Core-Pipeline in Rust
- FFI nur für C-Bibliotheken (lcms2, optional LibRaw als Fallback)
- WASM-Target für Plugin-Sandbox
- Contributors brauchen Rust-Kenntnisse
- Hervorragende Testbarkeit durch Ownership-System

---

## ADR-004: GPU-Pipeline mit wgpu

### Status: Akzeptiert

### Kontext
RAW-Entwicklung ist rechenintensiv. Slider-Änderungen müssen in <16ms (60fps) sichtbar sein. CPU allein reicht nicht für grosse RAW-Dateien (50-100MP).

### Entscheidung
**wgpu** als GPU-Abstraktionslayer. Compute Shaders in **WGSL** für alle Bildverarbeitungs-Operationen.

### Alternativen betrachtet
| Technologie | Pro | Contra |
|------------|-----|--------|
| **wgpu (Vulkan/Metal/DX12/WebGPU)** | **Cross-Platform, Rust-native, zukunftssicher (WebGPU)** | **Jünger als native APIs** |
| Native Metal + Vulkan + DX12 | Maximum Performance | 3× Code schreiben und warten |
| OpenCL | Cross-Platform compute | Veraltend, kein Metal-Support |
| CUDA | NVIDIA-optimiert | NVIDIA-only |
| OpenGL | Breit unterstützt | Veraltet, kein Compute-Focus |
| CPU only | Einfach, überall | Zu langsam für interaktive RAW-Entwicklung |

### Konsequenzen
- Ein Shader-Code (WGSL) für alle Plattformen
- CPU-Fallback-Pipeline für Systeme ohne GPU (Server, alte Hardware)
- GPU-Kompatibilitätstests auf verschiedenen Generationen nötig
- Floating-Point-Varianz zwischen GPUs akzeptieren (±2 LSB bei 8-bit)

---

## ADR-005: XMP-First Kompatibilität

### Status: Akzeptiert

### Kontext
Lightroom-Nutzer haben jahrelange Workflows aufgebaut, die auf XMP-Sidecars basieren. Kompatibilität ist der wichtigste Wechselgrund.

### Entscheidung
**XMP-Sidecars als primäres Austauschformat.** Alle Develop-Settings werden intern in einem eigenen JSON-Format gespeichert, aber können jederzeit als Adobe-kompatible XMP-Sidecars geschrieben und gelesen werden.

### Details
- Internes Format: JSON (in SQLite DB)
- Austauschformat: XMP mit `crs:` Namespace (Adobe Camera Raw Settings)
- Sync-Optionen: Auto-Write, Manual, Read-Only
- Unbekannte XMP-Namespaces werden erhalten (nicht gelöscht)

### Alternativen betrachtet
| Ansatz | Pro | Contra |
|--------|-----|--------|
| **Eigenes JSON + XMP-Export** | **Flexibel, kein Adobe-Lock-in, trotzdem kompatibel** | **Zwei Formate pflegen** |
| XMP als primäres Format | Maximale Kompatibilität | Eingeschränkt durch Adobe-Spec, Parsing-Overhead |
| Eigenes Format, kein XMP | Volle Freiheit | Keine Lightroom-Kompatibilität |

### Konsequenzen
- Bidirektionales Mapping OCPS-JSON ↔ Adobe-XMP
- Kompatibilitäts-Levels (1-4) für transparente Kommunikation
- XMP Compatibility Test Suite mit Lightroom-generierten Testdateien
- Verlustfreier Roundtrip ist Ziel, aber nicht garantiert (Algorithmus-Unterschiede)

---

## ADR-006: Shortcut-First UX

### Status: Akzeptiert

### Kontext
Fotografen verbringen Stunden mit Culling und Entwicklung. Jede gesparte Sekunde multipliziert sich über tausende Fotos. Tastatur ist schneller als Maus.

### Entscheidung
**Keyboard-First Design:** Jede Aktion ist per Shortcut erreichbar. Lightroom-kompatibles Default-Profil. Optionaler Vim-Mode.

### Details
- Shortcut-Engine mit Kontext-Auflösung (Global > Module > Tool)
- Frei konfigurierbare Keymaps (JSON)
- Lightroom-Profil als Default
- Vim-Profil als Alternative
- Command Palette (Cmd+K) für alles
- Shortcut Discovery Overlay (?)

### Konsequenzen
- Jede neue Aktion braucht einen Default-Shortcut
- Shortcut-Conflict-Tests in CI
- Höherer Entwicklungsaufwand (Shortcut-Engine + Keymaps)
- Bessere Accessibility (vollständige Tastaturnavigation)

---

## ADR-007: Plugin Sandbox (WASM)

### Status: Akzeptiert

### Kontext
Plugins sind essenziell für Erweiterbarkeit, aber dürfen die Hauptanwendung nicht destabilisieren. Sicherheit und Stabilität sind kritisch.

### Entscheidung
**WASM-basierte Plugin-Sandbox mit wasmtime.** Plugins laufen in isoliertem WASM-Speicher mit definierten Permissions.

### Alternativen betrachtet
| Ansatz | Pro | Contra |
|--------|-----|--------|
| **WASM (wasmtime)** | **Sandbox, Multi-Language, performant** | **Grössere Images, WASM-Einschränkungen** |
| Native Plugins (dylib) | Maximale Performance | Crash-Gefahr, Sicherheitsrisiko |
| Lua Scripting | Einfach, bewährt | Langsam für Bildverarbeitung |
| JavaScript (V8) | Grosses Ökosystem | Overhead, kein GPU-Zugang |
| Docker/Subprocess | Maximale Isolation | Zu langsam, komplex |

### Konsequenzen
- Plugin-API muss WASM-kompatibel sein (keine rohen Pointer, kein unsafe)
- Bilddaten werden per Shared Memory übergeben (keine Kopie)
- GPU-Zugang für Plugins über definierte Pipeline-Hooks
- Plugin-Autoren können Rust, C, C++, AssemblyScript, Go (WASM-fähig) verwenden
- Performance-Overhead: ~5-10% vs. native (akzeptabel)

---

## ADR-008: Catalog + Sidecar Dual Strategy

### Status: Akzeptiert

### Kontext
Fotos und ihre Bearbeitungen müssen persistent gespeichert werden. Es gibt zwei Traditionen: Catalog-DB (Lightroom) und Sidecar-Files (darktable, Photo Mechanic).

### Entscheidung
**Beide.** SQLite-Katalog als primärer Speicher + optionale XMP-Sidecars für Interoperabilität und Backup.

### Konfliktstrategie
```
Sync-Modus: Auto | Manual | ReadOnly | Disabled

Auto:
  - Jede Änderung → sofort in DB UND XMP
  - Externe XMP-Änderung erkannt → Merge-Dialog

Manual:
  - Änderungen nur in DB
  - XMP-Export per Cmd+S oder Batch

ReadOnly:
  - XMP wird gelesen, nie geschrieben
  - Für shared Storage

Disabled:
  - Keine XMP-Interaktion
```

### Konsequenzen
- Doppelte Write-Logik (DB + XMP)
- Konflikt-Detection nötig (File-Watcher + Hash-Vergleich)
- Nutzer muss Sync-Modus wählen können
- Bei Crash: DB ist authoritative, XMP ist Backup

---

## ADR-009: Golden Image Test Suite

### Status: Akzeptiert

### Kontext
Die RAW-Processing-Pipeline muss deterministisch sein. Änderungen an Shadern oder Algorithmen dürfen bestehende Ergebnisse nicht unbeabsichtigt verändern.

### Entscheidung
**Golden Image Test Suite:** Referenz-RAW-Dateien + definierte Settings → Expected Output. Jeder CI-Run vergleicht Pipeline-Output mit Golden Images.

### Details
- 1 Golden Image pro RAW-Format (Minimum)
- 1 Golden Image pro Edit-Modul
- Toleranz: ≤1 LSB (CPU), ≤2 LSB (GPU)
- Golden Images leben in Git LFS
- Updates nur per RFC und TC-Approval

### Konsequenzen
- Pipeline-Änderungen sind explizit und nachvollziehbar
- Golden Image Updates sind "Breaking Changes"
- Höherer CI-Speicherbedarf (LFS)
- Deterministischer Seed für Grain und andere Random-Operationen

---

## ADR-010: Dual Licensing / OEM-Modell

### Status: Akzeptiert

### Kontext
Das Projekt braucht langfristig Einnahmen. Community-Version soll kostenlos bleiben. Kommerzielle Einbettung soll Geld bringen.

### Entscheidung
**Dual-License-Modell:** PolyForm Noncommercial (frei) + Commercial License (kostenpflichtig). CLA von Contributors sichert die Rechte für Dual-Licensing.

### Einnahme-Quellen (nach Priorität)
1. Kommerzielle Lizenzen (OEM, SaaS, Enterprise)
2. Support & Consulting
3. Marketplace-Provision (Plugins, Presets)
4. Sponsoring (GitHub Sponsors, Open Collective)
5. Optional: Pro-Features (Cloud-Sync, Team-Features)

### Fairness-Massnahmen
- Community-Version hat vollen Funktionsumfang
- Kein Feature-Gating
- CLA enthält Fairness-Klausel (Apache 2.0 bei Verkauf)
- Transparente Preisstruktur
- Indie-Preise deutlich unter Enterprise

### Konsequenzen
- CLA ist Pflicht → potenzielle Hürde für Contributors
- Klare Kommunikation nötig (FAQ: "Warum CLA?")
- Lizenz-Management-System nötig (später)
- Regelmässige Review der Preisstrategie
