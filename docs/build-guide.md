# Build Guide — OpenClaw Photo Studio

Complete guide for building and running OCPS from source.

## Prerequisites

### Required Software

- **Rust 1.78+** (stable): Install via [rustup](https://rustup.rs/)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup default stable
  rustup update
  ```

- **Node.js 20+**: Install via [nvm](https://github.com/nvm-sh/nvm) or [official installer](https://nodejs.org/)
  ```bash
  node --version  # Should be v20.x or higher
  ```

- **pnpm 9+**: Install globally via npm
  ```bash
  npm install -g pnpm
  pnpm --version  # Should be 9.x or higher
  ```

### Platform-Specific Dependencies

#### macOS
```bash
xcode-select --install
```

Required for native compilation. Installs Clang, Git, and other build tools.

#### Linux (Debian/Ubuntu)
```bash
sudo apt-get update
sudo apt-get install -y \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev
```

#### Linux (Fedora/RHEL)
```bash
sudo dnf install -y \
  gtk3-devel \
  webkit2gtk4.1-devel \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  patchelf \
  openssl-devel
```

#### Windows
- **Visual Studio Build Tools 2022**: [Download](https://visualstudio.microsoft.com/downloads/)
  - Select "Desktop development with C++"
  - Includes MSVC, Windows SDK, and CMake

- **WebView2**: Usually pre-installed on Windows 11. For Windows 10, download [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

---

## Quick Start

### 1. Clone Repository
```bash
git clone https://github.com/quantumnic/openclaw-photo-studio.git
cd openclaw-photo-studio
```

### 2. Build and Run (Development Mode)
```bash
cd app
pnpm install
pnpm tauri dev
```

This will:
- Install frontend dependencies
- Build Rust backend in debug mode
- Launch the development app with hot-reload

**First build**: Expect 3-5 minutes (Rust compiles all dependencies).
**Subsequent builds**: 10-30 seconds (incremental).

### 3. Verify Installation
The app should open with the Library module. Try:
- **Import Folder**: Click "📁 Import Folder" and select a folder with photos
- **Rate Photos**: Click a photo, press `1`-`5` to rate
- **Keyboard Shortcuts**: Press `Cmd/Ctrl+K` for command palette

---

## Running Tests

### All Tests (Workspace-Wide)
```bash
cargo test --all
```

### Per-Crate Tests
```bash
# Core engine tests
cargo test --package ocps-core

# Catalog tests
cargo test --package ocps-catalog

# Pipeline tests only
cargo test --package ocps-core --lib pipeline

# Integration tests
cargo test --package ocps-core --test integration_pipeline
```

### CLI Tool Tests
```bash
cargo test --package ocps-cli
```

### Expected Test Count
Target: **>200 total tests** (as of Phase 2 completion)

---

## CLI Tool

### Build CLI
```bash
cargo build --release --bin ocps
```

Binary location:
- macOS/Linux: `target/release/ocps`
- Windows: `target\release\ocps.exe`

### Usage Examples
```bash
# Show help
./target/release/ocps --help

# Import photos
./target/release/ocps import /path/to/photos --catalog my-catalog.ocps

# Show stats
./target/release/ocps stats --catalog my-catalog.ocps

# List photos
./target/release/ocps list --rating 4 --catalog my-catalog.ocps

# Export photos
./target/release/ocps export all --output ./exports --quality 90 --catalog my-catalog.ocps
```

---

## Building for Release

### Desktop App
```bash
cd app
pnpm tauri build
```

**Output Locations:**

**macOS:**
- DMG: `app/src-tauri/target/release/bundle/dmg/openclaw-photo-studio_*.dmg`
- App: `app/src-tauri/target/release/bundle/macos/openclaw-photo-studio.app`

**Windows:**
- MSI: `app\src-tauri\target\release\bundle\msi\openclaw-photo-studio_*.msi`
- EXE: `app\src-tauri\target\release\openclaw-photo-studio.exe`

**Linux:**
- AppImage: `app/src-tauri/target/release/bundle/appimage/openclaw-photo-studio_*.AppImage`
- Deb: `app/src-tauri/target/release/bundle/deb/openclaw-photo-studio_*.deb`

### Release Build Time
- **First build**: 10-15 minutes
- **Incremental**: 1-3 minutes

### Optimizations
Release builds include:
- LTO (Link-Time Optimization)
- Code stripping (removes debug symbols)
- Frontend minification (JS/CSS)

---

## Development Workflow

### Hot Reload (Frontend Only)
```bash
cd app
pnpm dev
```

Runs Vite dev server without Tauri. Backend API calls will fail, but UI can be tested.

### Full Stack Dev Mode
```bash
cd app
pnpm tauri dev
```

Both frontend and backend with hot-reload (frontend only — Rust changes require restart).

### Rebuild Rust Only
```bash
cd app/src-tauri
cargo build
```

### Watch Mode (Rust)
```bash
cargo install cargo-watch
cargo watch -x "test --package ocps-core"
```

---

## Benchmarks

### Run Criterion Benchmarks
```bash
cargo bench --package ocps-core
```

Generates HTML report at `target/criterion/report/index.html`.

### Benchmark Targets
- `process_4mp_default`: Full pipeline on 4MP image
- `process_1mp_exposure_only`: Exposure adjustment only
- `histogram_4mp`: Histogram generation
- `white_balance_1mp`: White balance operation

**Note:** Benchmarks should be run in `--release` mode (criterion does this automatically).

---

## Troubleshooting

### Problem: "rustc version mismatch"
**Solution:**
```bash
rustup update stable
rustup default stable
```

### Problem: "failed to load plugin `typescript`"
**Solution:**
```bash
cd app
rm -rf node_modules pnpm-lock.yaml
pnpm install
```

### Problem: "dyld: Library not loaded" (macOS)
**Solution:** Reinstall Xcode Command Line Tools:
```bash
sudo rm -rf /Library/Developer/CommandLineTools
xcode-select --install
```

### Problem: "webkit2gtk not found" (Linux)
**Solution:**
```bash
sudo apt-get install libwebkit2gtk-4.1-dev
```

### Problem: Build takes forever (Windows)
**Solution:**
- Disable Windows Defender real-time scanning for project folder
- Add `target/` to exclusion list

### Problem: Out of memory during build
**Solution:** Reduce parallel jobs:
```bash
cargo build --release -j 2
```

---

## Environment Variables

### Build Configuration
- `RUST_LOG`: Set log level (`debug`, `info`, `warn`, `error`)
  ```bash
  RUST_LOG=debug pnpm tauri dev
  ```

- `OCPS_CATALOG_PATH`: Default catalog location (optional)
  ```bash
  export OCPS_CATALOG_PATH=~/Photos/catalog.ocps
  ```

### Performance Tuning
- `RAYON_NUM_THREADS`: Control parallelism (default: CPU cores)
  ```bash
  RAYON_NUM_THREADS=4 cargo test
  ```

---

## IDE Setup

### VS Code (Recommended)
**Extensions:**
- `rust-analyzer` (Rust LSP)
- `Tauri` (Tauri integration)
- `bradlc.vscode-tailwindcss` (TailwindCSS)
- `dbaeumer.vscode-eslint` (ESLint)

**Settings (`.vscode/settings.json`):**
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### IntelliJ IDEA / CLion
- Install Rust plugin
- Enable "Attach debugger" for Tauri
- Set working directory to `app/src-tauri`

---

## Clean Builds

### Full Clean
```bash
# Clean Rust artifacts
cargo clean

# Clean frontend artifacts
cd app && rm -rf node_modules dist pnpm-lock.yaml && pnpm install

# Clean everything (nuclear option)
git clean -fdx
```

### Incremental Clean
```bash
# Just Rust target directory
cargo clean

# Just frontend dist
cd app && rm -rf dist
```

---

## Next Steps

- See [architecture.md](./architecture.md) for codebase overview
- See [ROADMAP.md](../concept/ROADMAP.md) for project phases
- See [CONTRIBUTING.md](../CONTRIBUTING.md) for contribution guidelines

---

**Last Updated:** 2026-03-19
**Version:** 0.1.0-draft
