# OpenClaw Photo Studio — Teststrategie

> Version: 0.1.0-draft | 2026-03-19

---

## 1. Test-Philosophie

- **Kein Code ohne Tests.** Jeder PR braucht Tests für neue Funktionalität.
- **Golden Images sind heilig.** Die RAW-Pipeline muss deterministische Ergebnisse liefern.
- **Real-World-Daten.** Tests mit echten RAW-Dateien von echten Kameras.
- **Automatisiert.** Alles was automatisiert werden kann, wird automatisiert.
- **Cross-Platform.** Jeder Test läuft auf macOS, Windows, Linux.

---

## 2. Test-Kategorien

### 2.1 Unit Tests
**Was:** Einzelne Funktionen und Module.
**Wo:** Neben dem Code (`src/module.rs` → `src/module_test.rs` oder `#[cfg(test)]` Module)
**Framework:** Rust: built-in `#[test]`, TypeScript: `vitest`
**Abdeckung:** Minimum 80% für Core, 60% für UI

**Beispiele:**
- XMP-Parser: Parse bekannte XMP → prüfe Werte
- Rename-Template: `{date}_{seq}` → `2026-03-19_001`
- Smart Collection Rules: `rating >= 4 AND camera = "Sony"` → SQL
- Tone Curve Interpolation: Punkt-Input → Kurven-Output
- Color Space Conversion: sRGB → ProPhoto → sRGB Roundtrip

### 2.2 Integration Tests
**Was:** Zusammenspiel mehrerer Module.
**Wo:** `tests/integration/`
**Framework:** Rust Integration Tests

**Beispiele:**
- Import → Catalog Eintrag → Thumbnail generiert
- Import → XMP lesen → Edit-Daten in DB
- Edit ändern → XMP schreiben → XMP lesen → Werte identisch
- Export Pipeline: RAW → Develop → JPEG (End-to-End)
- Plugin laden → Plugin ausführen → Ergebnis prüfen

### 2.3 Golden Image Tests
**Was:** Deterministischer Output der RAW-Pipeline.
**Wo:** `tests/golden-images/`
**Methode:**
1. Referenz-RAW + definierte Edit-Settings
2. Pipeline durchlaufen lassen
3. Output-Pixel mit gespeichertem Golden Image vergleichen
4. Toleranz: ≤1 LSB Differenz pro Kanal (bei 8-bit), exakt bei 16-bit

**Golden Image Set:**
- 1 Bild pro unterstütztem RAW-Format (Minimum)
- 1 Bild pro Edit-Modul (Exposure, WB, HSL, Curves, NR, Sharpening etc.)
- 1 Bild mit allen Modulen aktiv
- 1 Bild mit Local Adjustments
- 1 Bild mit Crop + Transform

**Update-Prozess:**
- Golden Images dürfen NUR per RFC und TC-Approval aktualisiert werden
- Jedes Update = Breaking Change für die Pipeline
- Update-Tool: `cargo run --bin golden-update -- --approve`
- Approval-Commit muss die Begründung enthalten

### 2.4 RAW Regression Suite
**Was:** Sicherstellen, dass neue RAW-Formate oder Pipeline-Änderungen keine bestehenden Ergebnisse verändern.
**Wo:** `tests/raw-regression/`
**Aufbau:**
```
tests/raw-regression/
├── canon/
│   ├── cr2/
│   │   ├── 5d3_daylight.CR2
│   │   ├── 5d3_daylight_golden.png
│   │   └── 5d3_daylight_settings.json
│   └── cr3/
│       ├── r5_iso100.CR3
│       ├── r5_iso100_golden.png
│       └── r5_iso100_settings.json
├── nikon/
│   ├── z8_14bit.NEF
│   └── ...
├── sony/
│   ├── a7iv_arw.ARW
│   └── ...
├── fuji/
│   ├── xt5_xtrans.RAF
│   └── ...
└── dng/
    ├── leica_q3.DNG
    └── ...
```

**Testdaten-Beschaffung:**
- Eigene Testbilder (CC0 lizenziert)
- Community-Beiträge (mit CLA)
- raw.pixls.us (Creative Commons RAW-Sammlung)

### 2.5 XMP Compatibility Suite
**Was:** Lesen und Schreiben von XMP-Sidecars, kompatibel mit Lightroom.
**Wo:** `tests/xmp-compat/`

**Tests:**
1. **Import-Tests:** Lightroom-generierte XMP-Dateien lesen
   - Basis-Settings (Exposure, WB, Contrast)
   - Tone Curve (parametric + point)
   - HSL Adjustments
   - Local Adjustments
   - Crop
   - Keywords
   - Ratings, Flags, Labels
   - GPS-Daten
   - IPTC-Felder

2. **Export-Tests:** OCPS-generierte XMP-Dateien in Lightroom lesen (manuell verifiziert, dann als Golden File)

3. **Roundtrip-Tests:**
   - OCPS schreibt XMP → OCPS liest XMP → Werte identisch
   - Lightroom-XMP → OCPS liest → OCPS schreibt → Diff minimal

4. **Preservation-Tests:**
   - Unbekannte XMP-Namespaces bleiben erhalten
   - Drittanbieter-Felder werden nicht gelöscht
   - Encoding (UTF-8, Sonderzeichen) bleibt intakt

5. **Edge Cases:**
   - Korrupte XMP-Dateien
   - Leere XMP-Dateien
   - Riesige XMP-Dateien (100+ Local Adjustments)
   - XMP mit Process Version PV2010 (Legacy)
   - XMP mit Process Version PV2012+

### 2.6 Catalog Migration Tests
**Was:** Sicherstellen, dass Schema-Migrationen funktionieren.
**Wo:** `tests/migrations/`

**Tests:**
- Jede Migration einzeln testen (v1→v2, v2→v3 etc.)
- Full-Chain-Migration (v1→latest)
- Datenerhaltung nach Migration
- Performance nach Migration (keine fehlenden Indizes)
- Rollback-Tests (wenn unterstützt)
- Tests mit Katalogen verschiedener Grössen (100, 10k, 100k)

### 2.7 Performance Benchmarks
**Was:** Automatisierte Performance-Tests.
**Wo:** `tests/benchmarks/` (Rust: `criterion`)
**CI:** Wöchentlich, Ergebnisse gespeichert, Regression-Alerts bei >10% Verschlechterung.

**Benchmarks:**
| Benchmark | Metrik | Target |
|-----------|--------|--------|
| RAW Decode (CR3, 45MP) | ms | <500ms |
| Demosaic AMaZE (45MP) | ms | <200ms (GPU) |
| Full Pipeline (default settings) | ms | <300ms (GPU) |
| Thumbnail Generation (1000 Fotos) | s | <30s |
| Catalog Open (100k Fotos) | ms | <3000ms |
| FTS5 Search (100k Katalog) | ms | <100ms |
| Grid Scroll (100k Thumbnails) | fps | >60fps |
| Slider Change → Preview | ms | <16ms |
| Export JPEG (45MP → 2048px) | ms | <500ms |
| Batch Export (100 JPEG) | s | <60s |
| Import 1000 RAW | s | <30s |
| Copy/Paste auf 100 Fotos | ms | <500ms |

### 2.8 Shortcut Conflict Tests
**Was:** Keine doppelten Shortcuts im gleichen Kontext.
**Methode:** Automatische Analyse aller Keymap-Dateien.

**Tests:**
- Keine Konflikte innerhalb eines Kontexts (Library, Develop, Global)
- Globale Shortcuts konfligieren nicht mit OS-Shortcuts (Liste gepflegt)
- Custom Keymaps werden auf Konflikte geprüft
- Import einer Keymap zeigt Konflikte an

### 2.9 UI Automation Tests
**Was:** End-to-End Tests der Benutzeroberfläche.
**Framework:** Playwright oder Tauri-Test-Framework
**Wo:** `tests/e2e/`

**Tests:**
- Import-Workflow (Ordner wählen → Fotos erscheinen in Grid)
- Develop-Workflow (Foto öffnen → Slider ändern → Before/After → Export)
- Copy/Paste-Workflow (Foto bearbeiten → Copy → Andere selektieren → Paste)
- Rating/Flagging (0-5, P/X/U per Tastatur)
- Collection erstellen und Fotos hinzufügen
- Search und Filter
- Export Dialog und Batch-Export
- Preferences öffnen und Shortcut ändern

### 2.10 Cross-Platform Matrix

| Plattform | OS | GPU | CI |
|-----------|-----|-----|-----|
| macOS ARM | macOS 14+ | Apple Silicon (Metal) | GitHub Actions (macOS runner) |
| macOS Intel | macOS 13+ | AMD/Intel (Metal) | GitHub Actions |
| Windows | Windows 10+ | NVIDIA (Vulkan/DX12) | GitHub Actions (Windows runner) |
| Windows | Windows 10+ | AMD (Vulkan/DX12) | Self-hosted |
| Windows | Windows 10+ | Intel (Vulkan/DX12) | Self-hosted |
| Linux | Ubuntu 22.04+ | NVIDIA (Vulkan) | GitHub Actions (Linux runner) |
| Linux | Ubuntu 22.04+ | AMD (Vulkan) | Self-hosted |
| Linux | Fedora 38+ | Intel (Vulkan) | Self-hosted |

### 2.11 GPU Matrix Tests
**Was:** Pipeline-Ergebnisse müssen auf allen GPUs identisch sein (innerhalb Toleranz).
**Methode:** Golden Image Comparison auf verschiedenen GPUs.
**Toleranz:** ≤2 LSB bei 8-bit (GPU-Floating-Point-Varianz)

### 2.12 Camera/Lens Profile Tests
**Was:** LensFun-Profile korrekt angewendet.
**Methode:** Before/After mit bekannten Korrektur-Erwartungen.

### 2.13 Fuzzing
**Was:** Robustheit der Parser gegen korrupte/malicious Eingaben.
**Framework:** `cargo-fuzz` (libFuzzer)
**Targets:**
- XMP Parser
- EXIF Parser
- RAW Format Parser (pro Format)
- Preset Parser (JSON)
- Keymap Parser
- Import Rename Template Parser
- Smart Collection Rule Parser

### 2.14 Corrupted File Recovery Tests
**Was:** App crasht nicht bei korrupten Dateien. Graceful Error Handling.
**Tests:**
- Truncated RAW-Dateien
- Korrupte EXIF-Header
- Ungültige XMP
- Beschädigte SQLite-Datenbank
- Fehlende Thumbnails im Cache
- Volles Dateisystem während Export

### 2.15 Large Library Stress Tests
**Was:** Performance und Stabilität bei grossen Katalogen.
**Setup:** Synthetischer Katalog mit 100k, 500k, 1M Einträgen.

**Tests:**
- Katalog öffnen
- Grid scrollen
- Suche (FTS5)
- Smart Collection Update
- Bulk Rating/Flagging
- Memory-Verbrauch über 8h Session
- Concurrent Import während Bearbeitung

---

## 3. Test-Infrastruktur

### 3.1 CI/CD Pipeline

```
On Push (any branch):
├── Lint (clippy, rustfmt, eslint, prettier)
├── Unit Tests (Rust + TypeScript)
├── Integration Tests
└── Build Check (all platforms)

On PR (to develop/main):
├── All of above +
├── Golden Image Tests
├── XMP Compatibility Tests
├── Shortcut Conflict Tests
├── UI Automation Tests (headless)
└── Security Audit (cargo-audit)

Nightly:
├── Full Regression Suite
├── Performance Benchmarks
├── Fuzzing (continuous, 1h per target)
├── Cross-Platform Matrix
└── Large Library Stress Tests

Weekly:
├── GPU Matrix Tests (self-hosted runners)
├── Memory Leak Detection (Valgrind/Instruments)
└── Benchmark Trend Report
```

### 3.2 Test Data Management
- Test-RAW-Dateien in Git LFS (separates Repository oder Git LFS im Monorepo)
- Golden Images versioniert (Hash-basiert)
- Synthetische Testdaten per Script generierbar
- Kein proprietäres Material (nur CC0 oder eigen)

### 3.3 Determinismus-Garantie

Die Export-Pipeline MUSS deterministisch sein:
- Gleiche RAW + Gleiche Settings = Gleicher Output (Bit-genau bei CPU-Pipeline, ≤2 LSB bei GPU)
- Kein Zufalls-Seed in der Pipeline (Grain wird mit deterministischem Seed generiert, basierend auf Photo-ID + Settings-Hash)
- Tests: Gleiche Pipeline 2× laufen lassen → Output vergleichen
