//! PNG export functionality

use crate::ExportError;
use image::{ImageFormat, RgbImage};
use std::path::Path;

/// Export RGB image data as PNG
///
/// # Arguments
/// * `rgb_data` - 8-bit sRGB data, RGB interleaved
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels
/// * `output_path` - Path to save the PNG file
pub fn export_png(
    rgb_data: &[u8],
    width: u32,
    height: u32,
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

    // Create image from raw data
    let img = RgbImage::from_raw(width, height, rgb_data.to_vec()).ok_or_else(|| {
        ExportError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed to create image from RGB data",
        ))
    })?;

    // Save as PNG
    img.save_with_format(output_path, ImageFormat::Png)
        .map_err(|e| {
            ExportError::Io(std::io::Error::other(
                format!("PNG encoding failed: {}", e),
            ))
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_png_creates_valid_file() {
        // Create 4x4 RGB image (blue gradient)
        let width = 4;
        let height = 4;
        let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);

        for y in 0..height {
            for x in 0..width {
                let value = ((y * width + x) * 16) as u8;
                rgb_data.push(0); // R
                rgb_data.push(0); // G
                rgb_data.push(value); // B
            }
        }

        // Export to temp file
        let temp_file = NamedTempFile::new().unwrap();
        let result = export_png(&rgb_data, width, height, temp_file.path());

        assert!(result.is_ok());
        assert!(temp_file.path().exists());

        // Verify file size is reasonable
        let metadata = std::fs::metadata(temp_file.path()).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_export_png_invalid_data_size() {
        let width = 4;
        let height = 4;
        let rgb_data = vec![0u8; 10]; // Wrong size

        let temp_file = NamedTempFile::new().unwrap();
        let result = export_png(&rgb_data, width, height, temp_file.path());

        assert!(result.is_err());
    }
}
