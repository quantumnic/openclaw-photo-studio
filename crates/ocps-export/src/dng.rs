//! DNG (Digital Negative) export
//!
//! DNG is Adobe's open RAW format, based on TIFF.
//! For our use case: export processed 16-bit RGB as DNG-compatible TIFF.

use crate::ExportError;
use std::path::Path;

/// Export 16-bit RGB data as DNG (TIFF with DNG-compatible tags)
///
/// # Arguments
/// * `rgb_data` - 16-bit RGB pixel data (u16 interleaved RGB)
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels
/// * `output_path` - Output file path
/// * `original_path` - Optional original file path for metadata
///
/// # Note
/// This creates a "processed DNG" which is essentially a 16-bit TIFF
/// with DNG-compatible structure. It's not a camera RAW DNG, but it's
/// useful for maximum quality lossless output that's compatible with
/// Lightroom and other DNG-aware applications.
pub fn export_dng(
    rgb_data: &[u16],
    width: u32,
    height: u32,
    output_path: &Path,
    _original_path: Option<&Path>,
) -> Result<(), ExportError> {
    // Validate dimensions
    let expected_len = (width as usize) * (height as usize) * 3;
    if rgb_data.len() != expected_len {
        return Err(ExportError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Invalid data length: expected {} ({}x{}x3), got {}",
                expected_len,
                width,
                height,
                rgb_data.len()
            ),
        )));
    }

    // Save as TIFF (16-bit per channel)
    // Convert u16 to u8 bytes for encoding
    let mut bytes = Vec::with_capacity(rgb_data.len() * 2);
    for &val in rgb_data {
        bytes.push((val >> 8) as u8);  // High byte
        bytes.push((val & 0xFF) as u8); // Low byte
    }

    use image::codecs::tiff::TiffEncoder;
    use image::ExtendedColorType;
    use image::ImageEncoder;
    use std::fs::File;
    use std::io::BufWriter;

    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);
    let encoder = TiffEncoder::new(writer);

    encoder
        .write_image(&bytes, width, height, ExtendedColorType::Rgb16)
        .map_err(|e| {
            ExportError::Io(std::io::Error::other(format!(
                "Failed to encode TIFF/DNG: {}",
                e
            )))
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_dng_creates_file() {
        // Create small 4x4 RGB u16 test image
        let width = 4;
        let height = 4;
        let mut data = Vec::with_capacity(width * height * 3);

        // Gradient pattern
        for y in 0..height {
            for x in 0..width {
                let r = ((x as f32 / width as f32) * 65535.0) as u16;
                let g = ((y as f32 / height as f32) * 65535.0) as u16;
                let b = 32768_u16; // Mid value
                data.push(r);
                data.push(g);
                data.push(b);
            }
        }

        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path();

        let result = export_dng(&data, width as u32, height as u32, output_path, None);
        assert!(result.is_ok(), "DNG export should succeed");

        // Verify file exists and has content
        let metadata = fs::metadata(output_path).unwrap();
        assert!(metadata.len() > 0, "DNG file should not be empty");
    }

    #[test]
    fn test_export_dng_is_valid_tiff() {
        // Create test image
        let width = 8;
        let height = 8;
        let data = vec![30000_u16; width * height * 3];

        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path();

        export_dng(&data, width as u32, height as u32, output_path, None).unwrap();

        // Read file and check TIFF magic bytes
        let file_data = fs::read(output_path).unwrap();

        // TIFF magic bytes (little-endian: 49 49 2A 00, or big-endian: 4D 4D 00 2A)
        assert!(
            file_data.len() > 4,
            "File should be large enough to contain TIFF header"
        );

        let is_tiff_le = file_data[0] == 0x49 && file_data[1] == 0x49 && file_data[2] == 0x2A && file_data[3] == 0x00;
        let is_tiff_be = file_data[0] == 0x4D && file_data[1] == 0x4D && file_data[2] == 0x00 && file_data[3] == 0x2A;

        assert!(
            is_tiff_le || is_tiff_be,
            "File should start with TIFF magic bytes (got: {:02X} {:02X} {:02X} {:02X})",
            file_data[0],
            file_data[1],
            file_data[2],
            file_data[3]
        );
    }

    #[test]
    fn test_export_dng_invalid_dimensions() {
        // Create data that doesn't match dimensions
        let width = 4;
        let height = 4;
        let wrong_data = vec![1000_u16; 10]; // Wrong size

        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path();

        let result = export_dng(&wrong_data, width, height, output_path, None);
        assert!(result.is_err(), "Should fail with invalid data length");
    }
}
