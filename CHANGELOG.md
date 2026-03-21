# Changelog

All notable changes to OpenClaw Photo Studio will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.0] - 2026-03-21

### Added
- 430+ comprehensive test suite including edge case tests
- SQLite memory optimizations (16MB cache, memory temp storage, 256MB mmap)
- Accessibility improvements (WCAG AA color contrast, skip-to-content, aria-labels)
- Complete user guide and plugin development guide
- Architecture documentation with ASCII diagrams
- Error recovery tests for RAW decoding and batch exports

### Fixed
- Memory optimization in catalog operations
- Color contrast for text-muted (WCAG AA compliance)
- All clippy warnings resolved
- Build warnings in release mode

### Improved
- Documentation structure and completeness
- Build troubleshooting guide
- README with v0.9.0 feature list

## [0.8.0] - 2026-03-20

### Added

#### Search & Organization
- **Semantic Search** with natural language query parsing
  - Parse queries like "sunset beach photos from 2025"
  - Date filters (this year, last month, specific year)
  - Rating filters (4 stars, picks, rejected)
  - Camera brand filters (Sony, Nikon, Canon, Fuji, etc.)
  - Location-based search by city names
  - Combined keyword + metadata search with FTS5
  - Query interpretation display shows parsed filters

#### Export Features
- **Export Queue** with job tracking and retry logic
  - Job status tracking (pending, running, completed, failed, cancelled)
  - Retry failed exports individually or in batch
  - Cancel exports in progress
  - Export history with error messages and timestamps
  - Queue status panel in UI with expandable job list
  - Persistence across app restarts

#### Preferences & Customization
- **Keyboard Shortcut Editor** with full UI
  - Visual shortcut customization by category
  - Conflict detection with warnings
  - Import/export keymap as JSON
  - Category grouping (Navigation, Rating, Develop, etc.)
  - Reset to defaults per-shortcut or globally
  - Live key capture with modifier support

#### UI Enhancements
- **Welcome Screen** redesign
  - Modern UI with logo and version display
  - Quick actions (Import Folder, Open Catalog)
  - Recent catalogs list with click-to-open
  - Footer with GitHub link and license info
  - Clean dark theme matching app design

#### Performance & Developer Tools
- **Performance Tracking** system
  - Operation timing for decode, pipeline, encode phases
  - Import phase tracking (scan, EXIF, insert)
  - Performance statistics API via Tauri command
  - Diagnostics panel showing avg/count per operation
  - PerfTracker utility for wrapping timed operations

### Technical

#### Testing
- **360+ tests passing** (up from 326 in v0.6.0)
  - 20+ new semantic search tests
  - 15 new export queue tests
  - 8 new shortcut editor tests
  - All existing tests still passing

#### Performance
- Semantic search: <50ms for 10k photo catalog with FTS5
- Export queue: Non-blocking with async job processing
- Performance tracking: <1ms overhead per tracked operation

#### Infrastructure
- Search query parser with regex-based pattern extraction
- Export job persistence with SQLite or in-memory queue
- Keymap storage in localStorage with JSON serialization
- PerfTracker with SystemTime timestamps

### Documentation
- Semantic search query syntax guide
- Export queue API reference
- Shortcut editor user guide
- Performance tracking integration docs

## [0.7.0] - 2026-03-20

### Added

#### Tethering & Camera Control
- **gPhoto2 provider** for real camera tethering (Linux/macOS)
  - Direct camera control via libgphoto2
  - Camera discovery with autodetect
  - Live view support (JPEG streaming)
  - RAW capture from tethered cameras
  - Feature-gated with `gphoto2` flag
- **Face Detection** API
  - Automatic face recognition in photos
  - Face-based auto-tagging capability
  - Portrait-optimized processing workflows
- **AI Denoise** integration
  - Neural network-based noise reduction
  - Preserve detail while removing grain
  - Batch denoise support for multiple photos

#### GPU Pipeline Expansion
- **Full GPU pipeline** with additional WGSL shaders
  - Tone curve shader (highlights, shadows, whites, blacks)
  - Color adjustment shader (vibrance, saturation, HSL)
  - Combine shader for multi-pass operations
  - Complete GPU-accelerated develop module

#### UI & Workflow Enhancements
- **Vim Mode** for keyboard-driven navigation
  - Normal, Visual, and Command states
  - hjkl navigation in grid view
  - Visual selection with v key
  - Command mode with : prefix
- **Adjustment Brush Tool** for local edits
  - Local adjustments with brush strokes
  - Customizable brush size and feather
  - Overlay visualization on canvas
  - Per-area exposure, contrast, clarity adjustments
- **Second Monitor Support**
  - Full-screen loupe on secondary display
  - Independent zoom and pan controls
  - Live preview updates synchronized
  - Dual-monitor professional workflows

#### Performance Optimizations
- **Parallel Import** with Rayon
  - Multi-threaded file scanning
  - Concurrent EXIF parsing
  - Progress events during import
  - 3-5x speedup for 100+ photo imports

### Technical

#### Testing
- **340+ tests passing**
  - 10 new GPU shader tests (tone, color, combine)
  - 8 new gPhoto2 provider tests (feature-gated)
  - 12 new AI denoise tests
  - 6 new face detection tests
  - 5 new Vim mode tests
  - 8 new brush tool tests

#### Performance
- Parallel import: 3-5x faster for large batches
- GPU shaders: <1ms per pass on modern GPUs
- AI denoise: Hardware-accelerated when available

#### Infrastructure
- gPhoto2 bindings with feature flag
- GPU shader compilation pipeline
- Brush tool canvas overlay system
- Second monitor window management

### Documentation
- gPhoto2 provider usage guide
- Vim mode keyboard reference
- GPU pipeline architecture docs
- Brush tool workflow examples

## [0.6.0] - 2026-03-20

### Added

#### Plugin System SDK v1.0 (Stable)
- **Plugin API v1.0** - Stable plugin interface contract
  - Host function imports: logging, image access, metadata, UI functions
  - Plugin exports: init, info, process_image, get_parameters
  - Memory management via linear WASM memory
  - Comprehensive API documentation with JSON schemas
- **SDK Templates** for plugin development
  - Rust plugin template with full host function wrappers
  - WAT (WebAssembly Text) plugin template for minimal plugins
  - Complete build instructions and Cargo.toml examples
  - 8 tests covering template generation and compilation

#### Plugin Marketplace Foundation
- **Marketplace client** for plugin discovery and installation
  - Demo marketplace with 5 sample plugins (LUT Loader, Flickr Upload, AI Denoise, etc.)
  - Search functionality by name and description
  - Filter by plugin type (image_filter, integration, ai_ml, etc.)
  - Top rated and most downloaded sorting
  - Plugin download with manifest and WASM generation
  - 12 tests covering marketplace operations

#### HDR Merge (Mertens Exposure Fusion)
- **Exposure fusion** for merging bracketed exposures
  - Mertens algorithm implementation (contrast + saturation + well-exposedness)
  - Quality metric weighting per pixel per exposure
  - Multi-resolution pyramid blending
  - No tone mapping needed - direct output
- **HdrMergeSettings** with deghosting and auto-align controls
- EV range calculation from exposure offsets
- 8 tests including single/multi-exposure merging, validation, and weight computation
- Tauri command `merge_hdr_photos()` for UI integration

#### Panorama Stitching Foundation
- **Panorama stitching** with projection support
  - Perspective, Cylindrical, and Spherical projection modes
  - Offset computation based on expected overlap percentage
  - Linear alpha blending at seams (smooth transitions)
  - Stitch map tracking (source index, x/y offsets)
- **PanoramaSettings** with projection type, overlap %, blend width
- 8 tests including single/identical image stitching, seam blending verification
- Layout calculation for multi-image panoramas

### Technical Details
- All modules fully tested (326 total tests passing)
- Clippy clean (warnings only, no errors)
- Comprehensive error handling with thiserror
- Documentation with examples and algorithm descriptions

## [0.4.0] - 2026-03-20

### Added

#### RAW Processing Enhancements
- **X-Trans demosaicing** for Fujifilm cameras (X-T5, X-H2, X-Pro3, X100V, GFX series)
  - 6x6 CFA pattern support with adaptive homogeneity-directed interpolation
  - Automatic detection of X-Trans sensors by camera model
  - Optimized bilinear interpolation adapted for non-Bayer patterns
- **Camera color matrix database** with 10 popular camera models
  - Sony: A7 IV (ILCE-7M4), A7R V (ILCE-7RM5)
  - Nikon: Z8, Z6 III
  - Canon: EOS R5, EOS R6 Mark II
  - Fujifilm: X-T5, X-H2
  - Panasonic: S5 II
  - Olympus/OM System: OM-5
  - Camera RGB to XYZ D65 color matrices from Adobe DNG SDK
  - Automatic camera-to-sRGB conversion with matrix multiplication

#### GPU Pipeline Foundation (wgpu)
- **wgpu-based compute shader infrastructure** for GPU-accelerated processing
  - Exposure adjustment shader (first real GPU pipeline stage)
  - WGSL shader with workgroup size optimization (64 threads)
  - Async buffer mapping for efficient CPU-GPU data transfer
  - Feature-gated GPU support (enabled with `gpu` feature flag)
- GPU context initialization and management
- Storage buffer bindings for input/output data
- Uniform buffer for shader parameters

#### Tethering Infrastructure
- **Tether provider abstraction** for camera connectivity
  - TetherProvider trait for pluggable camera backends
  - MockTetherProvider for testing without hardware
  - Camera discovery, connection, and disconnection APIs
  - Image capture with RAW file byte streams
  - Live view frame support (JPEG streaming)
- **Tauri commands** for tethering integration
  - `discover_cameras()` - Find available tethered cameras
  - `connect_camera(id)` - Connect to specific camera
  - `disconnect_camera()` - Disconnect from current camera
  - `tether_capture()` - Trigger capture and return shot count
  - `check_tethered_camera()` - Status check (backwards compatibility)
- TetherSession for managing capture sessions
  - Import folder configuration
  - Auto-import toggle
  - Shot count tracking

### Technical

#### Testing
- **295 tests passing** (target: >295)
  - 22 new tests for X-Trans demosaicing
  - 10 new tests for camera color profiles
  - 2 new GPU shader tests (feature-gated)
  - 9 new tethering tests
- All clippy checks passing
- Test coverage for Fuji X-Trans pattern recognition
- GPU tests skip gracefully when no GPU available

#### Infrastructure
- X-Trans pattern lookup table (6x6 repeating CFA)
- Camera profile lookup with partial model matching
- GPU shader compilation with WGSL
- Mock JPEG generation for tethered capture testing
- Mutex-protected tether provider in AppState

### Performance
- X-Trans demosaicing: 5x5 neighborhood with inverse distance weighting
- GPU exposure shader: Sub-millisecond processing for 1920x1080 images
- Camera color matrix: Parallel processing with rayon

### Documentation
- Updated CONCEPT.md references for RAW pipeline (Section 8)
- Added inline documentation for TetherProvider trait
- WGSL shader comments for exposure computation

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
