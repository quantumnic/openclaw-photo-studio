# OCPS Image Processing Pipeline

## Overview

This module implements a **CPU-based reference image processing pipeline** for OpenClaw Photo Studio. It serves as:

1. **Primary processing engine** for Phase 1-2 (before GPU pipeline)
2. **Reference implementation** for validating GPU pipeline correctness
3. **Fallback** for systems without GPU acceleration
4. **Test harness** for algorithm development

## Architecture

```
RAW Image (u16) → Demosaic → RGB16 (linear) → Edit Pipeline → RGB8 (sRGB)
                                                    ↓
                                              EditRecipe
```

### Processing Order

The pipeline applies adjustments in a specific order for optimal quality:

1. **White Balance** - Color temperature and tint adjustment
2. **Exposure** - Global brightness (EV adjustment)
3. **Contrast** - Midtone contrast (S-curve)
4. **Tone Mapping** - Highlights, Shadows, Whites, Blacks
5. **Clarity** - Local contrast enhancement (unsharp mask on luminance)
6. **Color Adjustments** - Vibrance and Saturation (HSV manipulation)
7. **Sharpening** - Detail enhancement (unsharp mask)
8. **Crop** - Geometric transformation
9. **Output Conversion** - Linear RGB → sRGB gamma encoding → u8

### Key Components

#### `types.rs`
- **`RgbImage16`** - 16-bit linear RGB image (working format)
- **`RgbImage8`** - 8-bit sRGB image (output format)
- **`EditRecipe`** - Complete set of adjustments
- **`WhiteBalance`** - Temperature (K) + Tint
- **`SharpeningSettings`** - Amount, Radius, Detail, Masking
- **`NoiseReductionSettings`** - Luminance and Color NR (placeholder)
- **`CropSettings`** - Normalized crop coordinates
- **`ColorGradingSettings`** - 3-way color wheels (future)

#### `color.rs`
- **Gamma encoding/decoding** - sRGB transfer function
- **RGB ↔ HSV conversion** - For saturation/vibrance
- **White balance calculation** - Planckian locus approximation
- **Color space utilities**

#### `process.rs`
- **`apply_exposure()`** - Multiply by 2^EV
- **`apply_white_balance()`** - Apply RGB multipliers
- **`apply_contrast()`** - S-curve around midpoint
- **`apply_highlights_shadows()`** - Tone range adjustments
- **`apply_saturation()`** - HSV-based color manipulation
- **`apply_clarity()`** - Local contrast (simplified unsharp mask)
- **`apply_sharpening()`** - Edge enhancement
- **`apply_crop()`** - Geometric crop

#### `mod.rs` (ImageProcessor)
- **`ImageProcessor::process()`** - Main pipeline
- **`ImageProcessor::process_batch()`** - Batch processing
- **`convert_to_u8()`** - Output conversion

## Usage

### Basic Processing

```rust
use ocps_core::{RgbImage16, EditRecipe, ImageProcessor};

// Create or load an RGB16 image (from RAW decode + demosaic)
let image = RgbImage16::new(1920, 1080);

// Create edit recipe
let mut recipe = EditRecipe::default();
recipe.exposure = 0.5;  // +0.5 EV
recipe.contrast = 20;   // +20%
recipe.saturation = 10; // +10%

// Process
let output = ImageProcessor::process(&image, &recipe);
// output is RgbImage8 (8-bit sRGB) ready for display or export
```

### Batch Processing

```rust
let images = vec![image1, image2, image3];
let recipe = EditRecipe {
    exposure: 1.0,
    ..Default::default()
};

let outputs = ImageProcessor::process_batch(&images, &recipe);
```

### Copy/Paste Workflow

```rust
// Copy settings from one image
let recipe_from_photo_a = catalog.get_recipe(photo_a_id);

// Apply to multiple images
for photo_id in selected_photos {
    let image = load_image(photo_id);
    let output = ImageProcessor::process(&image, &recipe_from_photo_a);
    save_or_display(output);
}
```

## Edit Recipe Structure

### Basic Tone

```rust
EditRecipe {
    exposure: f32,        // -5.0 to +5.0 EV
    contrast: i32,        // -100 to +100
    highlights: i32,      // -100 to +100 (recover/enhance bright tones)
    shadows: i32,         // -100 to +100 (lift/darken shadows)
    whites: i32,          // -100 to +100 (very bright tones)
    blacks: i32,          // -100 to +100 (very dark tones)
    ..Default::default()
}
```

### Color

```rust
EditRecipe {
    white_balance: WhiteBalance {
        temperature: 6500, // 2000-50000 Kelvin
        tint: 10,          // -150 to +150
    },
    vibrance: 30,          // -100 to +100 (smart saturation)
    saturation: 10,        // -100 to +100 (all colors)
    ..Default::default()
}
```

### Detail

```rust
EditRecipe {
    clarity: 25,           // -100 to +100 (local contrast)
    sharpening: SharpeningSettings {
        amount: 60,        // 0-150
        radius: 1.0,       // 0.5-3.0
        detail: 50,        // 0-100
        masking: 20,       // 0-100
    },
    ..Default::default()
}
```

### Crop

```rust
EditRecipe {
    crop: CropSettings {
        left: 0.1,         // Normalized 0.0-1.0
        top: 0.1,
        right: 0.9,
        bottom: 0.9,
        angle: 2.5,        // Rotation in degrees
    },
    ..Default::default()
}
```

## Performance Characteristics

### CPU Performance (estimated)

| Image Size | Process Time | Throughput |
|------------|--------------|------------|
| 1920x1080  | ~8-15ms      | ~60-120 fps |
| 3840x2160  | ~30-50ms     | ~20-30 fps |
| 6000x4000  | ~80-120ms    | ~8-12 fps |

*On modern CPU (e.g., Apple M1/M2, AMD Ryzen 5000+, Intel 12th gen+)*

### Optimization Notes

- All processing is **single-threaded** currently
- **Future optimization**: Rayon for parallel processing
- **GPU pipeline** (Phase 3) will be 10-50x faster
- Current implementation prioritizes **correctness over speed**

## Algorithm Details

### White Balance

Uses simplified Planckian locus approximation:
- Temperature affects R/B balance
- Tint affects G/M balance
- Results normalized to prevent darkening

### Contrast

S-curve centered at midpoint (32768 in 16-bit):
```
output = midpoint + (input - midpoint) * factor
```

### Highlights/Shadows

Weighted adjustment based on luminance ranges:
- **Shadows**: 0-30% luminance
- **Highlights**: 70-100% luminance
- **Blacks**: 0-15% luminance
- **Whites**: 85-100% luminance

### Clarity

Local contrast enhancement using unsharp mask:
1. Calculate local average (box blur, radius=5)
2. Enhance difference: `output = input + strength * (input - blurred)`
3. Applied to all channels equally

### Saturation/Vibrance

HSV-based adjustment:
- **Saturation**: Uniform scaling of S channel
- **Vibrance**: More effect on muted colors, protects skin tones

### Sharpening

Unsharp mask:
1. Blur image (box blur with configurable radius)
2. Calculate difference
3. Add back scaled difference: `output = input + amount * (input - blurred)`

## Testing

### Test Coverage

- ✅ 63 unit tests (all passing)
- ✅ Identity tests (zero adjustment = no change)
- ✅ Boundary tests (extreme values don't crash)
- ✅ Roundtrip tests (encode/decode, RGB/HSV)
- ✅ Determinism tests (same input = same output)
- ✅ Batch processing tests

### Running Tests

```bash
# All pipeline tests
cargo test --package ocps-core --lib pipeline

# Specific test
cargo test --package ocps-core --lib test_exposure_zero_is_identity

# With output
cargo test --package ocps-core --lib -- --nocapture
```

## Future Work

### Phase 1 (Current)
- ✅ CPU reference pipeline
- ✅ All basic adjustments
- ✅ Comprehensive tests

### Phase 2
- [ ] Noise reduction implementation (currently placeholder)
- [ ] Better demosaic (AMaZE algorithm)
- [ ] Lens corrections integration

### Phase 3 (GPU Pipeline)
- [ ] wgpu compute shaders for all operations
- [ ] GPU/CPU parity tests (golden images)
- [ ] Performance benchmarks

### Phase 4+
- [ ] Local adjustments (masks)
- [ ] Tone curve (parametric + points)
- [ ] HSL mixer (8 channels)
- [ ] Color grading (3-way wheels)

## Known Limitations

1. **Sharpening/Clarity**: Uses box blur instead of Gaussian (faster but less accurate)
2. **Noise Reduction**: Not implemented (settings exist but don't process)
3. **Color Science**: Simplified WB calculation (not exact Planckian)
4. **Crop Rotation**: Not implemented (only rectangular crop)
5. **Single-threaded**: No parallelization yet

## Compatibility

### Lightroom Equivalents

| OCPS Parameter | Lightroom Equivalent |
|----------------|---------------------|
| `exposure`     | Exposure            |
| `contrast`     | Contrast            |
| `highlights`   | Highlights          |
| `shadows`      | Shadows             |
| `whites`       | Whites              |
| `blacks`       | Blacks              |
| `clarity`      | Clarity             |
| `dehaze`       | Dehaze (NYI)        |
| `vibrance`     | Vibrance            |
| `saturation`   | Saturation          |

Parameter ranges and behavior are designed to match Lightroom Classic as closely as possible.

## Contributing

When adding new processing functions:

1. Add the algorithm to `process.rs`
2. Add parameters to `EditRecipe` in `types.rs`
3. Update `ImageProcessor::process()` to call it in the right order
4. Write tests (identity, boundaries, correctness)
5. Update this README

---

**See also:**
- [ROADMAP.md](../../../../../docs/concept/ROADMAP.md) - Phase 1 Day 71-90
- [COPY-PASTE-SPEC.md](../../../../../docs/concept/COPY-PASTE-SPEC.md) - Copy/Paste workflow
- [CONCEPT.md](../../../../../docs/concept/CONCEPT.md) - Overall architecture
