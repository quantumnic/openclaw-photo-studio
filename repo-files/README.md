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
- Import from folders, cards, cameras
- Grid, Loupe, Compare, Survey views
- Rating (★), Flags (Pick/Reject), Color Labels
- Collections, Smart Collections, Quick Collection
- Hierarchical Keywords, IPTC Metadata
- Full-text search, advanced filters
- Virtual Copies, Stacks

### Develop
- Non-destructive RAW editing
- White Balance, Exposure, Tone, HSL, Color Grading
- Tone Curves (Parametric + Point)
- Sharpening, Noise Reduction, Dehaze, Clarity
- Local Adjustments (Brush, Gradient, Radial)
- Lens Corrections (LensFun)
- Crop & Transform
- History, Snapshots, Before/After
- **Copy/Paste edits across photos in 2 keystrokes**

### Export
- JPEG, TIFF, PNG, WebP, AVIF, DNG
- Resize, Output Sharpening, Watermark
- Naming templates, Export presets
- Batch export (background processing)

### Compatibility
- XMP sidecar read/write (Adobe-compatible)
- Lightroom catalog import
- Lightroom preset import (.xmp, .lrtemplate)
- EXIF, IPTC, XMP metadata
- DNG support

## Screenshots

> *Coming in Phase 2*

## Quick Start

### Requirements
- Rust 1.78+ (stable)
- Node.js 20+
- pnpm 9+
- GPU with Vulkan, Metal, or DirectX 12 support

### Build from Source

```bash
# Clone
git clone https://github.com/openclaw/photo-studio.git
cd photo-studio

# Install dependencies
pnpm install

# Build and run
cargo tauri dev
```

### Download Binaries

> *Coming with v1.0 release*

## Documentation

- [Architecture](docs/architecture.md)
- [Build Guide](docs/build-guide.md)
- [Plugin Development](docs/plugin-dev-guide.md)
- [Preset Format](docs/preset-format.md)
- [XMP Compatibility](docs/xmp-compatibility.md)
- [Shortcut Reference](docs/shortcuts.md)
- [FAQ](docs/faq.md)

## Contributing

We welcome contributions! Please read:

1. [CONTRIBUTING.md](CONTRIBUTING.md) — How to contribute
2. [GOVERNANCE.md](GOVERNANCE.md) — How decisions are made
3. [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) — Community standards

**Note:** All contributors must sign our [CLA](CLA.md) (Contributor License Agreement). This is required because the project uses dual licensing. The CLA does not transfer exclusive rights — you retain copyright of your contributions.

## License

OpenClaw Photo Studio is licensed under the **PolyForm Noncommercial License 1.0.0**.

- ✅ Free for personal use, learning, research, and non-commercial work
- ✅ Free for professional photographers using the software to edit their own photos
- ✅ Free for community development and non-commercial forks
- ❌ Not free for embedding in commercial software, SaaS, OEM, or white-label products

For commercial licensing, see [COMMERCIAL.md](COMMERCIAL.md) or contact licensing@openclaw.photo.

**This is source-available software, not "open source" as defined by the OSI.**

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

## Status

🚧 **Pre-Alpha** — Under active development. Not ready for production use.

---

*Built with 🌊 by the OpenClaw Photo Studio community.*
