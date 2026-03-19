//! JPEG export functionality

use crate::ExportError;
use crate::color::{OutputColorSpace, embed_icc_profile};
use image::RgbImage;
use std::path::Path;

/// Export RGB image data as JPEG with ICC profile
///
/// # Arguments
/// * `rgb_data` - 8-bit sRGB data, RGB interleaved
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels
/// * `quality` - JPEG quality 1-100 (higher is better)
/// * `output_path` - Path to save the JPEG file
/// * `color_space` - Optional color space (defaults to sRGB)
pub fn export_jpeg(
    rgb_data: &[u8],
    width: u32,
    height: u32,
    quality: u32,
    output_path: &Path,
) -> Result<(), ExportError> {
    export_jpeg_with_profile(rgb_data, width, height, quality, output_path, &OutputColorSpace::SRGB)
}

/// Export RGB image data as JPEG with specific color space
///
/// # Arguments
/// * `rgb_data` - 8-bit RGB data
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels
/// * `quality` - JPEG quality 1-100 (higher is better)
/// * `output_path` - Path to save the JPEG file
/// * `color_space` - Color space and ICC profile to embed
pub fn export_jpeg_with_profile(
    rgb_data: &[u8],
    width: u32,
    height: u32,
    quality: u32,
    output_path: &Path,
    color_space: &OutputColorSpace,
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

    // Encode to memory first so we can embed ICC profile
    let mut jpeg_buffer = Vec::new();
    {
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_buffer, quality as u8);
        encoder
            .encode(img.as_raw(), width, height, image::ExtendedColorType::Rgb8)
            .map_err(|e| {
                ExportError::Io(std::io::Error::other(
                    format!("JPEG encoding failed: {}", e),
                ))
            })?;
    }

    // Embed ICC profile
    let jpeg_with_profile = embed_icc_profile(&jpeg_buffer, color_space);

    // Write to file
    std::fs::write(output_path, jpeg_with_profile)?;

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

    #[test]
    fn test_export_jpeg_embeds_icc_profile() {
        use crate::color::OutputColorSpace;

        let width = 4;
        let height = 4;
        let rgb_data = vec![128u8; (width * height * 3) as usize];

        let temp_file = NamedTempFile::new().unwrap();
        let result = export_jpeg_with_profile(
            &rgb_data,
            width,
            height,
            90,
            temp_file.path(),
            &OutputColorSpace::SRGB,
        );

        assert!(result.is_ok());

        // Read back and verify ICC profile is present
        let jpeg_data = std::fs::read(temp_file.path()).unwrap();

        // Should start with JPEG SOI marker
        assert_eq!(jpeg_data[0], 0xFF);
        assert_eq!(jpeg_data[1], 0xD8);

        // Should contain APP2 marker (0xFF 0xE2)
        let has_app2 = jpeg_data.windows(2)
            .any(|w| w[0] == 0xFF && w[1] == 0xE2);
        assert!(has_app2, "JPEG should contain APP2 marker for ICC profile");

        // Should contain ICC_PROFILE identifier
        let icc_marker = b"ICC_PROFILE\0";
        let has_icc = jpeg_data.windows(icc_marker.len())
            .any(|w| w == icc_marker);
        assert!(has_icc, "JPEG should contain ICC_PROFILE marker");
    }

    #[test]
    fn test_export_jpeg_adobe_rgb() {
        use crate::color::OutputColorSpace;

        let width = 2;
        let height = 2;
        let rgb_data = vec![200u8; (width * height * 3) as usize];

        let temp_file = NamedTempFile::new().unwrap();
        let result = export_jpeg_with_profile(
            &rgb_data,
            width,
            height,
            95,
            temp_file.path(),
            &OutputColorSpace::AdobeRGB,
        );

        assert!(result.is_ok());

        // Verify file was created
        let metadata = std::fs::metadata(temp_file.path()).unwrap();
        assert!(metadata.len() > 0);
    }
}
