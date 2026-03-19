# Test Fixtures

This directory contains test files used for integration testing.

## Files

### sample.xmp
A minimal Adobe Lightroom-style XMP sidecar file with known values for testing the XMP parser.

Contains:
- Camera Raw Settings (crs: namespace)
- Rating and Label (xmp: namespace)
- IPTC metadata (dc: namespace)

## Usage

These fixtures are used by tests in:
- `crates/ocps-xmp/src/reader.rs`
- `crates/ocps-catalog/src/db.rs`

## Adding New Fixtures

When adding new test files:
1. Add the file to this directory
2. Document it in this README
3. Reference it in the relevant test module
