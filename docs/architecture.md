# Architecture — OpenClaw Photo Studio

High-level architecture overview of the OCPS codebase.

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Desktop Application                      │
│  ┌─────────────────────────────────────────────────────┐   │
│  │           SolidJS Frontend (TypeScript)              │   │
│  │  • Library Grid, Loupe, Compare, Survey              │   │
│  │  • Develop Module (Sliders, Histogram, Panels)       │   │
│  │  • Command Palette, Shortcuts, Diagnostics           │   │
│  └────────────────────┬────────────────────────────────┘   │
│                       │ Tauri IPC (JSON-RPC)                │
│  ┌────────────────────┴────────────────────────────────┐   │
│  │              Tauri Backend (Rust)                    │   │
│  │  • Command Handlers (import, rate, export)           │   │
│  │  • Event System (progress, updates)                  │   │
│  │  • File System Access, Dialog Integration            │   │
│  └────────────────────┬────────────────────────────────┘   │
│                       │ Function Calls                       │
│  ┌────────────────────┴────────────────────────────────┐   │
│  │            Core Crates (Rust Libraries)              │   │
│  │  • ocps-core:        RAW, Pipeline, Histogram        │   │
│  │  • ocps-catalog:     SQLite, Collections, Smart      │   │
│  │  • ocps-xmp:         XMP/EXIF Read/Write             │   │
│  │  • ocps-export:      JPEG/TIFF/PNG Export            │   │
│  │  • ocps-plugin-host: WASM Plugin Runtime             │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘

                              ↕
                    ┌─────────────────┐
                    │  File System     │
                    │  • RAW Files     │
                    │  • XMP Sidecars  │
                    │  • Catalog DB    │
                    │  • Previews      │
                    └─────────────────┘
```

---

## Crate Architecture

### 1. **ocps-core** — Core Engine
**Purpose:** RAW decoding, demosaic, image processing pipeline, histogram generation.

**Key Modules:**
- `raw/`: RAW file decoding (uses `rawloader` crate)
  - `decode()`: Extract sensor data from RAW files
  - Supports DNG, CR3, ARW, NEF, RAF, ORF, RW2

- `demosaic/`: Bayer/X-Trans to RGB conversion
  - `bilinear()`: Fast demosaic (Phase 1)
  - `amaze()`: High-quality demosaic (Phase 2)

- `pipeline/`: Non-destructive image processing
  - `types.rs`: RgbImage16, RgbImage8, EditRecipe
  - `color.rs`: Color space conversion, gamma encoding
  - `process.rs`: Exposure, WB, contrast, clarity, sharpening
  - `mod.rs`: ImageProcessor orchestrator
  - **Working space:** Linear RGB u16
  - **Output space:** sRGB u8

- `histogram/`: Histogram generation (RGB + Luminance)

**Design Principles:**
- CPU-first: GPU pipeline comes in Phase 3
- Deterministic: Same input → same output (critical for undo/redo)
- Immutable: Clone data for modifications, never mutate originals

**Test Coverage:** 74 tests (63 unit + 10 integration)

---

### 2. **ocps-catalog** — Catalog Database
**Purpose:** SQLite-based catalog with FTS5 search, collections, keywords, smart rules.

**Key Modules:**
- `db.rs`: Main Catalog interface
  - `import_folder()`: Recursive import with deduplication
  - `get_photos()`: Filtered, sorted queries
  - `batch_update_*()`: Batch operations for performance
  - `create_smart_collection()`: Rule-based auto-collections

- `models.rs`: Data types (PhotoRecord, PhotoFilter, ImportResult)

- `geo.rs`: GPS and geocoding support

- `lightroom_import.rs`: Lightroom catalog migration

**Schema (SQLite):**
- `photos`: Main photo table (id, path, EXIF, rating, flag, edits)
- `collections`: Manual + smart collections
- `photo_collections`: Many-to-many mapping
- `keywords`: Hierarchical keywords
- `photo_keywords`: Photo-keyword mapping
- `edits`: Non-destructive edit history (JSON)

**Indexes:**
- `idx_photos_date`: Date-based queries
- `idx_photos_rating`: Rating filters
- `idx_photos_flag`: Pick/reject filtering
- `idx_photos_hash`: Duplicate detection

**Performance:**
- Handles 100k+ photos efficiently
- FTS5 full-text search: <100ms for most queries
- Batch operations: 1000 photos/second

---

### 3. **ocps-xmp** — XMP/EXIF Engine
**Purpose:** Read/write XMP sidecars, EXIF extraction, preset management.

**Key Types:**
- `XmpDevelopSettings`: Adobe-compatible develop settings
  - Maps to Lightroom's XMP schema (PV2012)
  - Exposure, WB, HSL, tone curve, local adjustments

- `Preset`: Saved develop settings
  - User presets + builtin presets
  - Import/export .xmp preset files

- `IptcMetadata`: IPTC fields (title, keywords, copyright)

**Compatibility:**
- Read: Lightroom, Capture One, darktable XMP
- Write: Adobe-compatible XMP (Lightroom can read)
- Roundtrip: Preserve unknown fields

**Formats:**
- `.arw.xmp`, `.nef.xmp`: Sidecar files
- `.dng`: Embedded XMP in DNG files
- `.xmp`: Standalone preset files

---

### 4. **ocps-export** — Export Engine
**Purpose:** Render and export processed images to various formats.

**Formats:**
- **JPEG**: Quality 1-100, EXIF embedding, ICC profile support
- **TIFF**: 8/16-bit, uncompressed/LZW
- **PNG**: 8/16-bit, transparency support
- **WebP/AVIF**: Modern formats (Phase 5)
- **DNG**: Smart previews (Phase 2)

**Features:**
- Resize: Long edge, short edge, megapixels, percentage
- Output sharpening: Screen, matte paper, glossy paper
- Watermarking: Text + image overlays
- Batch export: Background job queue

**Pipeline Integration:**
```
RAW → Demosaic → Pipeline (16-bit linear RGB)
    → Gamma encode → Resize → Sharpen → JPEG/TIFF
```

---

### 5. **ocps-plugin-host** — Plugin System
**Purpose:** WASM-based plugin runtime for extensibility.

**Current State:** Foundation only (Phase 1)

**Planned Features (Phase 6):**
- WASM sandbox (wasmtime)
- Plugin manifest (permissions, capabilities)
- Plugin API v1 (stable, versioniert)
- AI plugins (denoise, masking, keywording)
- Custom export targets (Flickr, SmugMug)

**Security:**
- Sandboxed execution (no direct file system access)
- Permission system (read-only, read-write, network)
- No native code execution

---

### 6. **ocps-cli** — Command-Line Interface
**Purpose:** Headless operations for automation, CI/CD, batch processing.

**Commands:**
- `import <path>`: Import folder into catalog
- `export <id|all>`: Export photos
- `stats`: Show catalog statistics
- `list`: List photos with filters

**Use Cases:**
- CI/CD: Automated export for web galleries
- Batch processing: Nighttime exports
- Server deployment: Headless processing engine

**Example:**
```bash
ocps import /photos --catalog ~/catalog.ocps
ocps list --rating 5 --flag pick --catalog ~/catalog.ocps
ocps export all --output ./exports --quality 90 --resize 2048
```

---

## Data Flow

### Import Workflow
```
User selects folder
    ↓
Tauri dialog → Backend import_folder command
    ↓
ocps-catalog: Walk directory, filter extensions
    ↓
For each file:
  - Calculate SHA-256 hash (deduplication)
  - Extract EXIF (ocps-xmp)
  - Insert into SQLite (photos table)
  - Generate thumbnail (ocps-core pipeline)
    ↓
Return ImportResult (total, inserted, skipped, errors)
    ↓
Frontend refreshes grid
```

### Rating Workflow
```
User presses 0-5 (or P/X/U)
    ↓
Frontend: KeyDown event → invoke("update_rating", { id, rating })
    ↓
Backend: Tauri command handler
    ↓
ocps-catalog: UPDATE photos SET rating = ?
    ↓
(Optional) Write XMP sidecar if auto-sync enabled
    ↓
Frontend: Update local state (optimistic UI)
```

### Edit Workflow
```
User adjusts slider (e.g., Exposure +0.5)
    ↓
Frontend: Update EditRecipe state
    ↓
Debounce 100ms → invoke("process_image", { id, recipe })
    ↓
Backend: Load RAW → Demosaic → Apply recipe
    ↓
ocps-core pipeline: Exposure → WB → Contrast → ... → Gamma encode
    ↓
Return RgbImage8 (JPEG-compressed for transport)
    ↓
Frontend: Display preview
    ↓
On save: invoke("save_edit", { id, recipe })
    ↓
ocps-catalog: Save JSON to edits table + XMP sidecar
```

### Export Workflow
```
User clicks Export button
    ↓
Frontend: Open export dialog, configure settings
    ↓
invoke("batch_export", { photo_ids, settings })
    ↓
Backend: Create background job
    ↓
For each photo:
  - Load RAW + Edit recipe
  - Process via pipeline
  - ocps-export: Resize → Sharpen → Save JPEG/TIFF
  - Emit progress event
    ↓
Frontend: Update progress bar
    ↓
Complete: Show notification
```

---

## Frontend Architecture (SolidJS)

### Component Structure
```
<AppShell>
  ├── <TopBar>                    # Module switcher, menu
  ├── <LeftSidebar>               # Folders, collections, keywords
  ├── <MainView>
  │   ├── <LibraryView>           # Grid, loupe, compare, survey
  │   ├── <DevelopView>           # Photo viewer + histogram
  │   ├── <MapView>               # GPS photo map
  │   └── <PrintView>             # Print layouts
  ├── <RightSidebar>
  │   ├── <HistogramPanel>        # RGB histogram
  │   ├── <DevelopPanels>         # Slider groups
  │   └── <MetadataPanel>         # EXIF/IPTC
  ├── <Filmstrip>                 # Bottom thumbnail strip
  └── <CommandPalette>            # Cmd+K quick actions
```

### State Management
- **Signals:** Reactive primitives (SolidJS)
- **Stores:** Complex state (e.g., filter, selection)
- **IPC Cache:** Minimize backend calls (debounce, batching)

### Keyboard Shortcuts
- Managed by `ShortcutEngine` class
- Context-aware (library vs. develop)
- Fully remappable (Phase 5)

**Examples:**
- `G`: Grid view
- `E`: Loupe view
- `D`: Develop module
- `0-5`: Rate photo
- `P/X/U`: Flag pick/reject/unflag
- `Cmd+C/V`: Copy/paste edits
- `\`: Before/after toggle

---

## Performance Considerations

### Bottlenecks (Current)
1. **Demosaic:** Bilinear is fast, AMaZE is slow (CPU-bound)
2. **Pipeline (debug):** 2.5s for 500x500 → Use `--release`
3. **Thumbnail generation:** Sequential → Need parallelization (Rayon)
4. **Grid scrolling:** Virtualization required for 100k+ photos

### Optimizations (Planned)
- **Phase 3:** GPU pipeline (wgpu) → 10-50x speedup
- **Phase 2:** Rayon for parallel processing
- **Phase 2:** Smart previews (compressed proxies)
- **Phase 2:** L1-L4 cache hierarchy (GPU → RAM → Disk → Original)

### Memory Budget
- **Target:** <2GB for 50k photo catalog + 10 previews
- **Current:** ~500MB baseline (SQLite + thumbnails)

---

## Testing Strategy

### Unit Tests
- Each crate has `#[cfg(test)] mod tests`
- Cover individual functions, edge cases
- Example: `test_exposure_identity()`, `test_white_balance_roundtrip()`

### Integration Tests
- `tests/integration/`: Cross-crate workflows
- Example: Import → Rate → Copy/Paste → Export
- Use in-memory catalog for speed

### Benchmark Tests
- `criterion` benchmarks in `benches/`
- Measure pipeline performance, histogram generation
- Track regressions

### Golden Image Tests (Future)
- Compare rendered output to reference images
- Detect visual regressions
- Use perceptual diff (SSIM)

---

## Security & Privacy

### Design Principles
1. **Local-first:** All data stored locally, no cloud dependency
2. **No telemetry:** Zero analytics, no phone-home
3. **Plugin sandboxing:** WASM isolation, permission model
4. **XMP safety:** Validate XML, prevent XXE attacks

### File Permissions
- Catalog: Read/write only to catalog file and XMP sidecars
- Import: Read-only access to source files
- Export: Write-only to export directory

---

## Future Architecture (Phase 3+)

### GPU Pipeline (wgpu)
```
CPU: RAW decode → Upload to GPU
GPU: Demosaic → Pipeline (shaders) → Download
CPU: Gamma encode → Display/Export
```

**Benefits:**
- 10-50x faster processing
- Real-time adjustments (60fps)
- 1:1 preview without lag

**Challenges:**
- GPU compatibility (Vulkan, Metal, DX12)
- Shader debugging
- CPU fallback for unsupported GPUs

### Plugin Ecosystem (Phase 6)
- Community marketplace (presets, plugins)
- Revenue sharing (85/15 split)
- Quality review process
- Automated testing

### Headless Server Mode (Phase 9)
- Docker deployment
- REST API for remote processing
- Web review companion app
- CI/CD integration

---

## Key Design Decisions

### Why Rust?
- **Performance:** CPU-intensive image processing
- **Safety:** Memory safety without GC overhead
- **Concurrency:** Rayon for parallel processing
- **Ecosystem:** rawloader, image, rusqlite, wgpu

### Why SolidJS?
- **Performance:** Fine-grained reactivity, minimal re-renders
- **Bundle size:** <10KB framework (vs. React 40KB)
- **Developer experience:** Similar to React, easier than Svelte

### Why Tauri?
- **Bundle size:** 15-20MB (vs. Electron 100-150MB)
- **Performance:** Native WebView, lower memory
- **Security:** Process isolation, capability-based permissions

### Why SQLite?
- **Simplicity:** Single-file database, no server
- **Performance:** Fast enough for 100k+ photos
- **Portability:** Cross-platform, embedded
- **FTS5:** Built-in full-text search

---

## References

- [ROADMAP.md](../concept/ROADMAP.md): Project phases and timeline
- [FEATURES.md](../concept/FEATURES.md): Complete feature list
- [build-guide.md](./build-guide.md): Build and test instructions
- [CONTRIBUTING.md](../CONTRIBUTING.md): Contribution guidelines

---

**Last Updated:** 2026-03-19
**Version:** 0.1.0-draft
