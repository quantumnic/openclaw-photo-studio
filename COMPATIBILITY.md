# Lightroom Compatibility — OpenClaw Photo Studio

## Compatibility Philosophy

OCPS aims for maximum workflow compatibility with Adobe Lightroom. We want Lightroom users to feel at home. However, we are NOT a Lightroom clone — we are an independent application with our own processing engine.

**Pixel-identical results are not a goal.** Different processing engines produce different results. We aim for "visually similar" and "workflow-compatible."

---

## Compatibility Levels

| Level | Name | Description |
|-------|------|-------------|
| **L1** | Metadata Import | EXIF, IPTC, XMP metadata, ratings, flags, labels, keywords |
| **L2** | Basic Develop | Exposure, WB, contrast, highlights, shadows, whites, blacks, vibrance, saturation |
| **L3** | Extended Develop | Tone curve, HSL, color grading, detail, lens corrections, crop |
| **L4** | Full Roundtrip | XMP written by OCPS is readable by Lightroom and vice versa |

---

## What Works

### L1 — Metadata (Fully Supported)
- ✅ EXIF data (camera, lens, exposure, GPS, dates)
- ✅ IPTC metadata (title, description, copyright, contact, location)
- ✅ XMP metadata (custom namespaces preserved)
- ✅ Star ratings (0-5)
- ✅ Pick/Reject flags
- ✅ Color labels (Red, Yellow, Green, Blue, Purple)
- ✅ Keywords (including hierarchies)
- ✅ GPS coordinates
- ✅ Face regions (read-only, no face recognition)

### L2 — Basic Develop Settings (Supported)
- ✅ White Balance (Temperature, Tint)
- ✅ Exposure
- ✅ Contrast
- ✅ Highlights / Shadows / Whites / Blacks
- ✅ Vibrance / Saturation
- ✅ Clarity
- ✅ Dehaze
- ⚠️ Results will differ slightly from Lightroom (different processing engine)

### L3 — Extended Develop Settings (Partially Supported)
- ✅ Tone Curve (Parametric + Point)
- ✅ HSL (Hue, Saturation, Luminance per channel)
- ✅ Color Grading (replaces Split Toning)
- ✅ Sharpening (Amount, Radius, Detail, Masking)
- ✅ Noise Reduction (Luminance + Color)
- ✅ Crop (position, aspect ratio, angle)
- ✅ Post-Crop Vignette
- ✅ Grain
- ⚠️ Lens Corrections (LensFun profiles, not Adobe profiles)
- ⚠️ Transform / Upright (algorithm differs)
- ⚠️ Local Adjustments (position imported, effect may differ)
- ⚠️ Camera Profiles (DCP import is best-effort)

### L4 — Roundtrip (Experimental)
- ⚠️ XMP written by OCPS → readable by Lightroom (basic settings)
- ⚠️ Local adjustments roundtrip may lose precision
- ❌ Lightroom AI Masks cannot be imported (proprietary)
- ❌ Lightroom Adaptive Presets not supported
- ❌ Process Version PV2010 (legacy) not supported

---

## What Does NOT Work

| Feature | Reason |
|---------|--------|
| Lightroom AI Masking (Subject, Sky, etc.) | Proprietary ML model, not in XMP |
| Adobe Camera Profiles (.dcp) | Semi-proprietary format, best-effort import |
| Lightroom Adaptive Presets | Proprietary format |
| Lightroom Publish Services state | Lightroom-internal, not in XMP |
| Lightroom face recognition data | Proprietary, stored in catalog only |
| Lightroom history | Not stored in XMP |
| Process Version PV2010 | Legacy, not supported |
| Pixel-identical results | Different processing engine |

---

## Import Paths

### XMP Sidecar Import
OCPS reads Adobe-compatible XMP sidecars (`.xmp` files next to RAW files).

**Automatic:** If OCPS finds a `.xmp` file next to a RAW file during import, it reads the develop settings and metadata.

**What's preserved:** All L1-L3 data as listed above.

**What's lost:** Lightroom-specific internal data, AI masks, publish status.

### Lightroom Catalog Import (.lrcat)
OCPS can import a Lightroom Classic catalog (`.lrcat` SQLite database).

**Imported:**
- Photo paths and folder structure
- Ratings, flags, color labels
- Keywords and keyword hierarchy
- Collections and Smart Collections (rule mapping)
- Virtual Copies
- Stacks
- Develop settings (via XMP sidecars)

**Not imported:**
- Plugin data
- Publish Services status
- Face recognition data
- Lightroom-specific history

**Process:**
1. Select `.lrcat` file
2. OCPS reads the SQLite database (read-only)
3. Maps Lightroom tables to OCPS schema
4. Reads XMP sidecars for develop settings
5. Generates migration report

### Lightroom Preset Import
OCPS can import Lightroom presets in both formats:
- `.xmp` presets (modern format) — direct import
- `.lrtemplate` presets (legacy format) — parsed and converted

**Compatibility:** ~80% of basic presets work. Presets using Adobe Camera Profiles or AI features may not translate fully.

---

## Sidecar Sync

### Sync Modes

| Mode | Behavior |
|------|----------|
| **Auto** | Every edit writes to XMP immediately. External changes detected via file watcher. |
| **Manual** | XMP written only on Cmd+S or batch save. |
| **Read-Only** | XMP read on import, never written. |
| **Disabled** | No XMP interaction. |

### Conflict Handling

When both the catalog DB and XMP sidecar have been modified:

1. **Detection:** File watcher detects XMP change, compares hash with stored hash
2. **Notification:** User sees conflict indicator on the photo
3. **Resolution options:**
   - Keep catalog version (ignore sidecar changes)
   - Keep sidecar version (overwrite catalog)
   - Merge (field-by-field comparison)
   - Create virtual copy with sidecar version

### Atomic Operations

When moving/renaming RAW files:
- RAW + XMP are always moved/renamed together
- If XMP write fails, the operation is rolled back
- Orphaned XMP files are detected and reported

---

## Compatibility Testing

We maintain a test suite with:
- XMP files generated by Lightroom Classic (various versions)
- XMP files with various Process Versions
- Presets in both .xmp and .lrtemplate format
- Edge cases (corrupted XMP, unknown namespaces, extreme values)

Tests run on every CI build. See `tests/xmp-compat/` for details.

---

## Known Differences

### Color Science
OCPS uses its own demosaicing algorithms and color pipeline. Colors will look slightly different from Lightroom, especially:
- Skin tones (Adobe has proprietary color science)
- Camera-specific color rendering
- Highlight recovery behavior
- Noise reduction artifacts

### Clarity / Dehaze
These are algorithm-dependent. Same slider value may produce visually different results.

### Lens Corrections
OCPS uses LensFun profiles instead of Adobe's lens profiles. Coverage and correction quality may differ.

---

*For questions about compatibility, open a Discussion or contact support@openclaw.photo.*
