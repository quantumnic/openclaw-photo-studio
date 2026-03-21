# Plugin Development Guide — OpenClaw Photo Studio

Learn how to create plugins for OCPS to extend its functionality.

---

## Overview

OpenClaw Photo Studio supports WASM-based plugins for extending functionality without compromising security or stability.

**Plugin Capabilities:**
- Image filters and effects (LUTs, vintage looks, AI upscaling)
- Export targets (Flickr, SmugMug, custom services)
- Metadata processors (AI keywording, face detection)
- Custom presets and workflows

**Plugin Architecture:**
- Sandboxed WASM runtime (wasmtime)
- Permission-based system (read-only, read-write, network)
- Hot-reload during development
- Versioned API (currently v1)

---

## Quick Start

### Prerequisites

- Rust 1.78+ with `wasm32-unknown-unknown` target
- `wasm-pack` for building WASM modules

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

### Create Your First Plugin

1. **Create a new Rust project:**

```bash
cargo new --lib my-plugin
cd my-plugin
```

2. **Update `Cargo.toml`:**

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
# OCPS plugin SDK (hypothetical - adjust to actual package)
ocps-plugin-sdk = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[profile.release]
opt-level = "z"       # Optimize for size
lto = true            # Link-time optimization
strip = true          # Strip debug symbols
```

3. **Create `plugin.toml` manifest:**

```toml
[plugin]
name = "my-plugin"
version = "1.0.0"
api_version = "1"
type = "image_filter"
description = "Example image filter plugin"
author = "Your Name"
entry_point = "plugin.wasm"

[permissions]
read_image = true
write_image = true
# network = false    # Network access disabled by default
# file_system = false
```

4. **Write plugin code (`src/lib.rs`):**

```rust
use ocps_plugin_sdk::{Plugin, ImageFilter, RgbImage, PluginResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MyFilterParams {
    intensity: f32,
}

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }
}

impl ImageFilter for MyPlugin {
    fn process(&self, image: &RgbImage, params: &str) -> PluginResult<RgbImage> {
        let params: MyFilterParams = serde_json::from_str(params)?;

        // Your image processing logic here
        let mut output = image.clone();

        for pixel in output.data.iter_mut() {
            *pixel = (*pixel as f32 * params.intensity) as u8;
        }

        Ok(output)
    }
}

// Export the plugin entry point
#[no_mangle]
pub extern "C" fn ocps_plugin_init() -> *mut dyn Plugin {
    Box::into_raw(Box::new(MyPlugin))
}
```

5. **Build the plugin:**

```bash
wasm-pack build --target wasm32-unknown-unknown --release
cp pkg/my_plugin.wasm plugin.wasm
```

6. **Test in OCPS:**

```bash
# Copy plugin to OCPS plugins directory
cp plugin.wasm ~/.openclaw/plugins/my-plugin/
cp plugin.toml ~/.openclaw/plugins/my-plugin/

# Restart OCPS to load the plugin
```

---

## Plugin Types

### 1. Image Filter

Process images with custom algorithms.

**Use Cases:**
- Custom LUT (Look-Up Table) application
- Vintage film emulation
- AI-based enhancement (denoise, upscale)

**Interface:**
```rust
trait ImageFilter {
    fn process(&self, image: &RgbImage, params: &str) -> PluginResult<RgbImage>;
}
```

**Example:** See `plugins/example-lut-plugin/` in the OCPS repository.

---

### 2. Export Target

Export photos to custom destinations.

**Use Cases:**
- Upload to cloud services (Flickr, SmugMug, Zenfolio)
- Custom FTP/SFTP export
- Generate static website galleries

**Interface:**
```rust
trait ExportTarget {
    fn export(&self, images: &[ExportImage], config: &str) -> PluginResult<ExportStatus>;
}
```

**Example:**
```rust
pub struct FlickrExporter;

impl ExportTarget for FlickrExporter {
    fn export(&self, images: &[ExportImage], config: &str) -> PluginResult<ExportStatus> {
        let config: FlickrConfig = serde_json::from_str(config)?;

        for image in images {
            // Upload to Flickr API
            flickr_api::upload(
                &image.data,
                &image.metadata,
                &config.api_key,
            )?;
        }

        Ok(ExportStatus::Success)
    }
}
```

---

### 3. Metadata Processor

Analyze and extract metadata from images.

**Use Cases:**
- AI-based keywording
- Face detection and tagging
- GPS geocoding
- Object recognition

**Interface:**
```rust
trait MetadataProcessor {
    fn process(&self, image: &RgbImage, existing: &Metadata) -> PluginResult<Metadata>;
}
```

**Example:**
```rust
pub struct AIKeywordPlugin;

impl MetadataProcessor for AIKeywordPlugin {
    fn process(&self, image: &RgbImage, _existing: &Metadata) -> PluginResult<Metadata> {
        // Run AI model to detect objects
        let keywords = ai_model::detect_objects(image)?;

        Ok(Metadata {
            keywords,
            ..Default::default()
        })
    }
}
```

---

## plugin.toml Format

Complete reference for the plugin manifest file.

```toml
[plugin]
# Required fields
name = "my-plugin"                 # Unique identifier (kebab-case)
version = "1.0.0"                  # Semantic version
api_version = "1"                  # OCPS plugin API version
type = "image_filter"              # Plugin type (see below)
description = "Example plugin"     # Brief description
author = "Your Name"               # Author name
entry_point = "plugin.wasm"        # WASM file name

# Optional fields
homepage = "https://example.com"
repository = "https://github.com/user/plugin"
license = "MIT"
keywords = ["filter", "vintage", "lut"]

[permissions]
# Permission flags (all default to false)
read_image = true       # Read image data
write_image = true      # Modify image data
read_metadata = false   # Read EXIF/XMP
write_metadata = false  # Write EXIF/XMP
file_system = false     # Access local file system
network = false         # Make network requests
clipboard = false       # Access clipboard

[config]
# Plugin-specific configuration (optional)
# These are exposed to users in the plugin settings UI
intensity_default = 1.0
color_mode = "rgb"
```

### Plugin Types

| Type | Description |
|------|-------------|
| `image_filter` | Process images (LUTs, effects, AI) |
| `export_target` | Export to custom destinations |
| `metadata_processor` | Extract/modify metadata |
| `preset` | Custom preset bundles |
| `workflow` | Automated workflows |

---

## Available Permissions

Plugins must declare permissions in `plugin.toml`. OCPS will deny access to undeclared capabilities.

### Image Permissions

```toml
[permissions]
read_image = true     # Required for filters
write_image = true    # Required to output modified image
```

**Use Case:** All image filter plugins need both read and write.

### Metadata Permissions

```toml
[permissions]
read_metadata = true   # Read EXIF, XMP, IPTC
write_metadata = true  # Modify metadata
```

**Use Case:** AI keywording plugin needs write_metadata.

### Network Permission

```toml
[permissions]
network = true         # Allow HTTP/HTTPS requests
```

**Use Case:** Export targets (Flickr, SmugMug) need network access.

**Security Note:** Network access is sandboxed. Plugins cannot access localhost or private IPs.

### File System Permission

```toml
[permissions]
file_system = true     # Access local file system
```

**Use Case:** Batch export to custom folder structure.

**Security Note:** File system access is limited to user-selected directories.

---

## Plugin Development Workflow

### 1. Development Setup

Create a `watch` script for hot-reload:

```bash
#!/bin/bash
# watch.sh - Auto-rebuild on file change
cargo watch -x "build --target wasm32-unknown-unknown --release" \
  -s "cp target/wasm32-unknown-unknown/release/my_plugin.wasm ~/.openclaw/plugins/my-plugin/plugin.wasm"
```

### 2. Testing

Test your plugin in OCPS:

1. Open OCPS
2. Go to Preferences → Plugins
3. Your plugin should appear in the list
4. Click "Reload Plugins" after rebuilding

**Debug Mode:**
```bash
OCPS_PLUGIN_DEBUG=1 ocps
```

This enables verbose plugin logs.

### 3. Debugging

Use `console.log` (mapped to OCPS debug output):

```rust
use ocps_plugin_sdk::debug;

debug!("Processing image with intensity: {}", params.intensity);
```

### 4. Distribution

Package your plugin for distribution:

```bash
# Create release package
mkdir -p release/my-plugin
cp plugin.wasm release/my-plugin/
cp plugin.toml release/my-plugin/
cp README.md release/my-plugin/
cp LICENSE release/my-plugin/

# Create archive
cd release
tar -czf my-plugin-1.0.0.tar.gz my-plugin/
```

Users can install by extracting to `~/.openclaw/plugins/`.

---

## Example: LUT Filter Plugin

Full example of a 3D LUT (Look-Up Table) filter plugin.

**Directory Structure:**
```
my-lut-plugin/
├── Cargo.toml
├── plugin.toml
├── src/
│   └── lib.rs
└── luts/
    ├── vintage.cube
    └── moody.cube
```

**plugin.toml:**
```toml
[plugin]
name = "lut-filter"
version = "1.0.0"
api_version = "1"
type = "image_filter"
description = "Apply 3D LUTs to photos"
author = "Community"
entry_point = "plugin.wasm"

[permissions]
read_image = true
write_image = true

[config]
lut_file = "luts/vintage.cube"
intensity = 1.0
```

**src/lib.rs:**
```rust
use ocps_plugin_sdk::{Plugin, ImageFilter, RgbImage, PluginResult};

pub struct LutPlugin {
    lut: Lut3D,
}

impl Plugin for LutPlugin {
    fn name(&self) -> &str { "lut-filter" }
    fn version(&self) -> &str { "1.0.0" }
}

impl ImageFilter for LutPlugin {
    fn process(&self, image: &RgbImage, params: &str) -> PluginResult<RgbImage> {
        let config: LutConfig = serde_json::from_str(params)?;
        let mut output = image.clone();

        for pixel in output.data.chunks_mut(3) {
            let (r, g, b) = (pixel[0], pixel[1], pixel[2]);
            let (r_new, g_new, b_new) = self.lut.lookup(r, g, b);

            // Apply intensity
            pixel[0] = lerp(r, r_new, config.intensity);
            pixel[1] = lerp(g, g_new, config.intensity);
            pixel[2] = lerp(b, b_new, config.intensity);
        }

        Ok(output)
    }
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 * (1.0 - t) + b as f32 * t) as u8
}
```

---

## Publishing Your Plugin

### Community Plugin Registry

Submit your plugin to the OCPS community registry:

1. **Open a PR** at `github.com/openclaw/plugin-registry`
2. **Include:**
   - Plugin source code (if open source)
   - Or compiled `.wasm` + `plugin.toml`
   - README with usage instructions
   - LICENSE file
3. **Review process:** ~1-2 weeks
4. **Once approved:** Plugin appears in OCPS Plugin Manager

### Selling Commercial Plugins

You can sell plugins independently:

- OCPS plugin system is open — no approval needed
- Distribute via your own website
- Use Gumroad, Patreon, or custom payment gateway
- No revenue share with OCPS (you keep 100%)

**Note:** The plugin runtime is source-available under PolyForm Noncommercial 1.0. Creating commercial plugins is allowed under the license.

---

## API Reference

### Core Types

```rust
pub struct RgbImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,  // RGB u8 data (length = width * height * 3)
}

pub struct Metadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub rating: u8,
    pub gps: Option<GpsCoords>,
}

pub struct ExportImage {
    pub data: Vec<u8>,
    pub format: ImageFormat,
    pub metadata: Metadata,
}
```

### Plugin Traits

```rust
pub trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
}

pub trait ImageFilter: Plugin {
    fn process(&self, image: &RgbImage, params: &str) -> PluginResult<RgbImage>;
}

pub trait ExportTarget: Plugin {
    fn export(&self, images: &[ExportImage], config: &str) -> PluginResult<ExportStatus>;
}

pub trait MetadataProcessor: Plugin {
    fn process(&self, image: &RgbImage, existing: &Metadata) -> PluginResult<Metadata>;
}
```

---

## Best Practices

### Performance

- **Minimize allocations**: Reuse buffers when possible
- **Parallelize**: Use rayon for multi-threaded processing
- **Optimize for size**: WASM size affects load time

### Security

- **Validate input**: Always validate user parameters
- **Limit resources**: Set timeouts for long-running operations
- **Never trust user data**: Sanitize all inputs

### UX

- **Provide defaults**: Good default parameters
- **Clear error messages**: Users should understand what went wrong
- **Documentation**: Include usage examples in README

---

## Troubleshooting

### Plugin Not Loading

**Check:**
1. `plugin.toml` is valid TOML
2. `entry_point` matches actual `.wasm` filename
3. Plugin is in `~/.openclaw/plugins/<plugin-name>/`

**Debug:**
```bash
OCPS_PLUGIN_DEBUG=1 ocps
```

### Permission Denied Errors

**Cause:** Plugin trying to access resource without permission.

**Fix:** Add permission to `plugin.toml`:
```toml
[permissions]
network = true  # Example: if network access denied
```

### WASM Build Fails

**Common issues:**
- Missing `wasm32-unknown-unknown` target
- Incompatible dependency versions

**Fix:**
```bash
rustup target add wasm32-unknown-unknown
cargo update
cargo clean && cargo build --target wasm32-unknown-unknown
```

---

## Further Resources

- [OCPS Plugin SDK Docs](https://docs.openclaw.photo/plugin-sdk) (coming soon)
- [Example Plugins](https://github.com/openclaw/plugins)
- [Plugin API Changelog](https://github.com/openclaw/photo-studio/blob/main/PLUGIN_API.md)
- [Community Discord](https://discord.gg/openclaw) — #plugin-dev channel

---

**Last Updated:** 2026-03-21
**Version:** 0.9.0
