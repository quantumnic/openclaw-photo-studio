# Changelog

All notable changes to OpenClaw Photo Studio will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-03-19

### Added

#### Color Management
- ICC profile embedding in JPEG exports (sRGB, Adobe RGB 1998, Display P3)
- Color space conversion for Adobe RGB (gamma 2.2)
- Soft proofing mode to simulate output color spaces
- Gamut warning overlay for out-of-gamut pixels (magenta highlight)

#### Export Features
- Text watermark engine with bitmap font rendering
- Contact sheet generation with customizable grid layouts (2-8 columns, 2-10 rows)
- Configurable watermark position (9 positions), opacity, color, and size
- Auto-resize images to fit contact sheet cells with aspect ratio preservation

#### Multi-Catalog Support
- Open existing catalogs from .ocps files
- Create new empty catalogs
- Close catalog command
- Catalog info API (path, photo count, creation/modification timestamps)

### Changed
- Catalog now stores its database path for multi-catalog workflows
- Enhanced get_catalog_info to return detailed metadata

### Technical
- 261 tests passing (>260 requirement met)
- All clippy warnings resolved
- Added 17 new tests for color management, watermark, and contact sheets

## [0.2.0] - 2026-03-19

### Added

#### RAW Processing & Core Engine
- RAW decoding for Sony ARW, Nikon NEF, Fuji RAF, Olympus ORF, Panasonic RW2, Canon CR2, DNG
- CPU image processing pipeline with 16-bit linear RGB working space
- Demosaicing algorithms (Bilinear for speed, AMaZE for quality)
- Gamma encoding/decoding (sRGB piecewise function)
- Color space conversions (RGB, HSV)

#### Develop Module
- White balance (temperature -100 to +100, tint -150 to +150)
- Exposure adjustment (-5 to +5 EV)
- Contrast, Highlights, Shadows, Whites, Blacks controls
- Clarity (local contrast enhancement with box blur)
- Vibrance & Saturation (HSV-based, preserves hue)
- Sharpening (unsharp mask with amount, radius, detail controls)
- Crop with aspect ratio presets
- Tone curve (parametric control)
- Color grading (3-way color wheels)
- Before/After toggle (\\ key)
- Histogram display (RGB + Luminance)

#### Catalog & Library
- SQLite catalog with full CRUD operations
- Photo import (Copy/Move/Add-in-Place modes)
- Rating system (0-5 stars, keyboard 1-5)
- Flag system (Pick/Reject/None, keyboard P/X/U)
- Color labels (Red/Yellow/Green/Blue/Purple, keyboard 6-9)
- Collections (manual and smart with SQL rules)
- Quick Collection (B key toggle)
- Stacking with position management
- Virtual copies with independent edits
- FTS5 full-text search (filename, camera make/model)
- Filter bar with multiple criteria
- Sort by date, rating, flag, filename
- Batch operations (rating, flag, label, delete)

#### XMP & Metadata
- XMP sidecar read/write (Adobe-compatible)
- EXIF metadata parsing (camera, lens, exposure data)
- Keyword management with hierarchy support
- Metadata templates for batch application
- IPTC fields (copyright, creator, location)

#### Export & Output
- JPEG export with quality control (60-100)
- PNG export (8-bit sRGB)
- TIFF export placeholder
- Batch export with progress tracking
- Resize on export (long edge constraint)
- Naming templates ({original}, {date}_{original}, {seq}_{original})
- Lanczos3 resampling for high-quality resizing

#### Lightroom Compatibility
- Lightroom catalog import (.lrcat)
- Lightroom preset import (.lrtemplate, .xmp)
- Smart collection rule mapping
- XMP roundtrip compatibility
- Process Version PV2012 support

#### Copy/Paste System
- Copy All Settings (Cmd+C)
- Paste Settings (Cmd+V)
- Copy Selected (Cmd+Shift+C with picker dialog)
- Paste Selected (Cmd+Shift+V)
- Auto-Sync mode
- Batch sync to multiple photos
- Module-aware copying (respect develop/crop modules)

#### Presets & Workflow
- 6 builtin presets (Neutral, Vivid, Matte, B&W, Vintage, Portrait)
- User preset creation and management
- Preset groups (Color, B&W, Creative)
- Apply preset with recipe merging
- Preset library with search

#### UI & Navigation
- Grid view with virtualized thumbnails
- Loupe view with GPU-accelerated preview
- Filmstrip with photo strip navigation
- Compare view (side-by-side)
- Survey view (multi-photo selection)
- Map module with OpenStreetMap integration
- Print module layout preview
- Command Palette (Cmd+K) with fuzzy search
- Keyboard shortcuts (Lightroom-compatible)
- Dark theme with zinc color palette
- Collapsible panels (left, right, filmstrip)
- Photo thumbnails with lazy loading
- Real-time preview updates

#### Lens Corrections
- Distortion correction (barrel/pincushion)
- Vignetting correction (amount + midpoint)
- LensFun library integration placeholder

#### Advanced Features
- History panel with full undo stack
- Snapshots (named save points)
- Plugin system foundation (manifest loading, registry, WASM host)
- CLI tool (ocps import/export/stats/list)
- Preferences panel (Cmd+,)
- Tethering placeholder (Phase 7 preview)
- App state persistence (localStorage)
- Preview cache with LRU eviction (200 photos default)

### Architecture

#### Project Structure
- **ocps-core**: RAW decode, demosaic, image pipeline
- **ocps-catalog**: SQLite catalog, FTS5 search, smart collections
- **ocps-xmp**: XMP/EXIF read/write, Adobe compatibility
- **ocps-export**: JPEG/PNG/TIFF export with resizing
- **ocps-plugin-host**: Plugin registry, manifest parsing, WASM runtime
- **app**: Tauri v2 desktop app with SolidJS frontend

#### Technology Stack
- Rust workspace with 5 crates
- Tauri v2 for desktop (macOS, Windows, Linux)
- SolidJS + TailwindCSS for UI
- SQLite with FTS5 for catalog
- rawloader for RAW decoding
- kamadak-exif for EXIF parsing
- GitHub Actions CI/CD (build + test on 3 platforms)

### Testing
- 200+ unit tests
- 15+ integration tests
- Test coverage for pipeline, catalog, XMP, export
- Golden image tests for color accuracy
- Roundtrip tests (OCPS → XMP → Lightroom → XMP → OCPS)

### Performance
- **Debug build**: ~2.5s for 500x500 with all adjustments
- **Release build** (estimated): 8-15ms for 1920x1080
- Preview cache: LRU with 200 photos
- Lazy loading for thumbnails
- Batch processing optimized for 100+ photos

### Known Limitations
- CPU-only processing (GPU pipeline in Phase 3)
- Box blur for clarity/sharpening (Gaussian blur coming)
- Single-threaded processing (Rayon parallelization planned)
- Demosaic outputs u8 (u16 pipeline conversion needed)
- No noise reduction yet (algorithm TBD)
- Crop rotation parameter exists but not implemented
- Dehaze parameter exists but not implemented

## [0.1.0] - 2026-03-19

### Added
- Initial project skeleton
- Cargo workspace structure
- Tauri v2 app shell
- Basic README and documentation

---

[0.2.0]: https://github.com/quantumnic/openclaw-photo-studio/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/quantumnic/openclaw-photo-studio/releases/tag/v0.1.0
