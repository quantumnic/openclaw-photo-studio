# OpenClaw Photo Studio

> Professional RAW photo workflow — source-available, Lightroom-compatible, keyboard-first.

[![Build](https://img.shields.io/badge/build-passing-brightgreen)]()
[![License](https://img.shields.io/badge/license-PolyForm%20NC%201.0-blue)]()
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)]()

---

## What is this?

OpenClaw Photo Studio (OCPS) is a professional photo workflow application for RAW development, photo management, and batch processing. It's built from scratch as a modern, fast, keyboard-first alternative to Adobe Lightroom — with maximum workflow compatibility.

**This is source-available software**, not "open source" in the OSI sense. Free for personal use, research, and community development. Commercial embedding requires a [separate license](COMMERCIAL.md).

## Why?

- **No subscription.** Download, build, use forever.
- **Familiar workflow.** If you know Lightroom, you know OCPS.
- **Fast.** GPU-accelerated pipeline. Sub-100ms preview updates.
- **Keyboard-first.** Every action has a shortcut. Vim mode optional.
- **Local-first.** Your photos stay on your disk. No cloud required.
- **Extensible.** Plugin system, community presets, scriptable.
- **Compatible.** Reads XMP sidecars, imports Lightroom catalogs, supports Lightroom presets.
- **Transparent.** Full source code available. No telemetry. No dark patterns.

## Features

### Library
- ✅ Import from folders (Copy/Move/Add)
- ✅ Grid, Loupe, Compare, Survey views
- ✅ Rating (0-5★), Flags (Pick/Reject), Color Labels
- ✅ Collections, Smart Collections, Quick Collection
- ✅ Hierarchical Keywords, IPTC Metadata
- ✅ Full-text search (FTS5), advanced filters
- ✅ Virtual Copies, Stacks
- ⏳ Tethered capture (Phase 7)

### Develop
- ✅ Non-destructive RAW editing
- ✅ White Balance, Exposure, Contrast, Tone controls
- ✅ Highlights, Shadows, Whites, Blacks, Clarity
- ✅ Vibrance, Saturation (HSV-based)
- ✅ Sharpening (unsharp mask)
- ✅ Crop with aspect ratios
- ✅ Tone Curves (Parametric control)
- ✅ Color Grading (3-way wheels)
- ✅ History, Snapshots, Before/After
- ✅ **Copy/Paste edits across photos in 2 keystrokes**
- ⏳ Noise Reduction (coming)
- ⏳ Dehaze (coming)
- ⏳ Local Adjustments (Phase 5)
- ⏳ Lens Corrections (Phase 5)

### Export
- ✅ JPEG (quality 60-100)
- ✅ PNG (8-bit sRGB)
- ✅ Resize with Lanczos3 resampling
- ✅ Naming templates ({original}, {date}, {seq})
- ✅ Batch export with progress tracking
- ⏳ TIFF, WebP, AVIF (coming)
- ⏳ Output Sharpening, Watermark (coming)

### Compatibility
- ✅ XMP sidecar read/write (Adobe-compatible)
- ✅ Lightroom catalog import (.lrcat)
- ✅ Lightroom preset import (.xmp, .lrtemplate)
- ✅ EXIF, IPTC, XMP metadata
- ✅ RAW: ARW, NEF, RAF, ORF, RW2, CR2, DNG
- ⏳ CR3 support (Canon R-series) coming

## Screenshots

> *Coming in Phase 2*

## Quick Start

### Requirements
- **Rust 1.78+** (stable) — Install via [rustup](https://rustup.rs/)
- **Node.js 20+** — Install via [nvm](https://github.com/nvm-sh/nvm) or [nodejs.org](https://nodejs.org/)
- **pnpm 9+** — Install via `npm install -g pnpm`
- **Platform dependencies:**
  - **macOS:** Xcode Command Line Tools (`xcode-select --install`)
  - **Linux:** `libgtk-3-dev libwebkit2gtk-4.1-dev` (see [build guide](docs/build-guide.md))
  - **Windows:** Visual Studio Build Tools 2022 + WebView2

### Build from Source

```bash
# Clone the repository
git clone https://github.com/quantumnic/openclaw-photo-studio.git
cd openclaw-photo-studio

# Install frontend dependencies
cd app
pnpm install

# Build and run in development mode
pnpm tauri dev
```

**First build:** Expect 3-5 minutes (Rust compiles dependencies).
**Subsequent builds:** 10-30 seconds (incremental compilation).

**Troubleshooting:** See [Build Guide](docs/build-guide.md) for common issues.

### Download Binaries

> Pre-built binaries coming with v1.0 stable release. For now, build from source.

## Documentation

### For Users
- [User Guide](docs/user-guide.md) — Complete guide to using OCPS
- [Keyboard Shortcuts](docs/concept/SHORTCUTS.md) — Full shortcut reference

### For Developers
- [Architecture](docs/architecture.md) — System design and data flow
- [Build Guide](docs/build-guide.md) — Building from source
- [Plugin Development](docs/plugin-guide.md) — Create custom plugins
- [Contributing](CONTRIBUTING.md) — How to contribute

### Advanced Topics
- [Roadmap](docs/concept/ROADMAP.md) — Development timeline and phases
- [Copy/Paste Spec](docs/concept/COPY-PASTE-SPEC.md) — Edit workflow design
- [Phase 1 Summary](docs/implementation/PHASE1-DAY71-90-SUMMARY.md) — Implementation details

## Contributing

We welcome contributions! Please read:

1. [CONTRIBUTING.md](CONTRIBUTING.md) — How to contribute
2. [GOVERNANCE.md](GOVERNANCE.md) — How decisions are made
3. [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) — Community standards

**Note:** All contributors must sign our [CLA](CLA.md) (Contributor License Agreement). This is required because the project uses dual licensing. The CLA does not transfer exclusive rights — you retain copyright of your contributions.

## License

OpenClaw Photo Studio is **source-available** under the **PolyForm Noncommercial License 1.0.0**.

### What's Allowed
- ✅ **Personal use** — Edit your own photos, manage your library
- ✅ **Professional use** — Photographers can use OCPS for client work
- ✅ **Learning & research** — Study the code, learn from implementation
- ✅ **Community development** — Contribute improvements, fix bugs
- ✅ **Non-commercial forks** — Create custom versions for personal/community use
- ✅ **Plugin development** — Create and sell commercial plugins independently

### What's Not Allowed
- ❌ **Commercial embedding** — Cannot embed OCPS in commercial software products
- ❌ **SaaS offerings** — Cannot offer OCPS as a hosted service
- ❌ **OEM/white-label** — Cannot rebrand and resell OCPS
- ❌ **Proprietary forks** — Cannot create closed-source commercial derivatives

### Commercial Licensing

For commercial use cases (embedding, SaaS, OEM), contact licensing@openclaw.photo or see [COMMERCIAL.md](COMMERCIAL.md).

**Important:** This is **NOT** "open source" as defined by the OSI. The source code is publicly available for transparency and community development, but commercial use requires a separate license.

## FAQ

### Is this Open Source?
No. The source code is publicly available, but the license restricts commercial use. We use the term "source-available" or "community-source." See [LICENSE-CHOICE.md](LICENSE-CHOICE.md) for the reasoning.

### Can I use it for my photography business?
Yes! Using OCPS to edit and export your photos is personal/professional use — no commercial license needed. The restriction is about *embedding OCPS into other software products*.

### Can I build a commercial plugin?
Yes. Plugins are separate works and don't require a commercial license. You can sell your plugins independently.

### Can I fork this?
Yes, for non-commercial purposes. If your fork is commercially distributed, you need a commercial license. Community forks that contribute upstream are encouraged.

### How compatible is it with Lightroom?
We support XMP sidecar import/export, Lightroom catalog import, and preset import. Develop settings are mapped as closely as possible, but pixel-identical results are not guaranteed (different processing engine). See [COMPATIBILITY.md](COMPATIBILITY.md).

## Acknowledgements

OCPS builds on the shoulders of excellent open-source projects:

- [rawloader](https://github.com/nicola-spieser/rawloader) — RAW format parsing
- [wgpu](https://wgpu.rs/) — GPU abstraction
- [Tauri](https://tauri.app/) — Desktop application framework
- [SolidJS](https://www.solidjs.com/) — UI framework
- [LensFun](https://lensfun.github.io/) — Lens correction profiles
- [dcraw](https://www.dechifro.org/dcraw/) — RAW processing reference
- [darktable](https://www.darktable.org/) / [RawTherapee](https://rawtherapee.com/) — Algorithmic inspiration (studied, not copied)

## Current Status

**v0.9.0 Release Candidate** — Core features complete, extensive testing in progress.

### Completed (Phase 1-2)
✅ **RAW Processing:** Decode and demosaic (ARW, NEF, RAF, DNG, CR2, ORF, RW2)
✅ **Image Pipeline:** Full CPU-based processing (WB, exposure, tone, clarity, saturation)
✅ **Catalog System:** SQLite catalog with FTS5 search, 100k+ photo support
✅ **Library Module:** Grid, Loupe, Compare, Survey views with keyboard navigation
✅ **Rating & Flagging:** 0-5 stars, pick/reject flags, color labels
✅ **Collections:** Manual and smart collections with rule-based filtering
✅ **Develop Module:** Non-destructive editing with history and snapshots
✅ **Copy/Paste Workflow:** 2-keystroke edit transfer across photos
✅ **Export:** JPEG, PNG with quality/resize options and batch processing
✅ **XMP Compatibility:** Read/write Adobe-compatible XMP sidecars
✅ **Keyboard Shortcuts:** 100+ shortcuts, command palette, customizable
✅ **Test Coverage:** 200+ tests passing (unit + integration)

### In Progress (Phase 3)
🚧 **GPU Pipeline:** wgpu-based acceleration for real-time performance
🚧 **Smart Previews:** Compressed proxies for faster editing
🚧 **Performance:** Rayon parallelization, optimized algorithms

### Planned (Phase 4-9)
📋 **Local Adjustments:** Brushes, gradients, radial filters (Phase 5)
📋 **Lens Corrections:** Distortion, vignetting, chromatic aberration (Phase 5)
📋 **Plugin System:** WASM-based extensibility (Phase 6)
📋 **Tethering:** Camera capture support (Phase 7)
📋 **Print Module:** Layout and printing (Phase 8)
📋 **AI Features:** Denoise, face detection, auto-keywording (Phase 9)

---

*Built with 🌊 by the OpenClaw Photo Studio community.*
