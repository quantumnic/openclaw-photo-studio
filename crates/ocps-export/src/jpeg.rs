//! JPEG export functionality

use crate::ExportError;
use image::RgbImage;
use std::path::Path;

/// Export RGB image data as JPEG
///
/// # Arguments
/// * `rgb_data` - 8-bit sRGB data, RGB interleaved
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels
/// * `quality` - JPEG quality 1-100 (higher is better)
/// * `output_path` - Path to save the JPEG file
pub fn export_jpeg(
    rgb_data: &[u8],
    width: u32,
    height: u32,
    quality: u32,
    output_path: &Path,
) -> Result<(), ExportError> {
    // Validate input
    let expected_size = (width * height * 3) as usize;
    if rgb_data.len() != expected_size {
        return Err(ExportError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Invalid data size: expected {}, got {}",
                expected_size,
                rgb_data.len()
            ),
        )));
    }

    // Clamp quality to valid range
    let quality = quality.clamp(1, 100);

    // Create image from raw data
    let img = RgbImage::from_raw(width, height, rgb_data.to_vec()).ok_or_else(|| {
        ExportError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed to create image from RGB data",
        ))
    })?;

    // Create a JPEG encoder with quality setting
    let mut output = std::fs::File::create(output_path)?;
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, quality as u8);

    // Encode and write
    encoder
        .encode(img.as_raw(), width, height, image::ExtendedColorType::Rgb8)
        .map_err(|e| {
            ExportError::Io(std::io::Error::other(
                format!("JPEG encoding failed: {}", e),
            ))
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_jpeg_creates_valid_file() {
        // Create 4x4 RGB image (red gradient)
        let width = 4;
        let height = 4;
        let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);

        for y in 0..height {
            for x in 0..width {
                let value = ((y * width + x) * 16) as u8;
                rgb_data.push(value); // R
                rgb_data.push(0); // G
                rgb_data.push(0); // B
            }
        }

        // Export to temp file
        let temp_file = NamedTempFile::new().unwrap();
        let result = export_jpeg(&rgb_data, width, height, 90, temp_file.path());

        assert!(result.is_ok());
        assert!(temp_file.path().exists());

        // Verify file size is reasonable (JPEG of 4x4 should be small but > 0)
        let metadata = std::fs::metadata(temp_file.path()).unwrap();
        assert!(metadata.len() > 0);
        assert!(metadata.len() < 10000); // Should be very small for 4x4
    }

    #[test]
    fn test_export_jpeg_quality_clamping() {
        let width = 2;
        let height = 2;
        let rgb_data = vec![255u8; (width * height * 3) as usize];

        let temp_file = NamedTempFile::new().unwrap();

        // Test quality > 100 (should clamp to 100)
        let result = export_jpeg(&rgb_data, width, height, 150, temp_file.path());
        assert!(result.is_ok());

        // Test quality < 1 (should clamp to 1)
        let result = export_jpeg(&rgb_data, width, height, 0, temp_file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_jpeg_invalid_data_size() {
        let width = 4;
        let height = 4;
        let rgb_data = vec![0u8; 10]; // Wrong size

        let temp_file = NamedTempFile::new().unwrap();
        let result = export_jpeg(&rgb_data, width, height, 90, temp_file.path());

        assert!(result.is_err());
    }
}
