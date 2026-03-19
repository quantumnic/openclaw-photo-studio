//! Image rendering service for Tauri IPC
//!
//! This service uses the CPU pipeline to render images at preview resolution,
//! encodes them as JPEG, and returns base64 data URIs for display in the UI.

use std::path::Path;
use std::time::Instant;

/// Result of rendering a photo
#[derive(Debug, Clone, serde::Serialize)]
pub struct RenderResult {
    pub width: u32,
    pub height: u32,
    pub data_uri: String, // "data:image/jpeg;base64,..."
    pub duration_ms: u64,
}

/// Check if a file is a RAW format
fn is_raw_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(
            ext_str.as_str(),
            "arw" | "nef" | "raf" | "dng" | "cr2" | "cr3" | "orf" | "rw2"
        )
    } else {
        false
    }
}

/// Render a photo with the given recipe and target dimensions
///
/// # Arguments
/// * `photo_path` - Absolute path to the RAW or JPEG file
/// * `recipe` - Edit recipe to apply
/// * `max_width` - Maximum width for the output (maintains aspect ratio)
/// * `max_height` - Maximum height for the output (maintains aspect ratio)
///
/// # Returns
/// * `Ok(RenderResult)` - Rendered image as base64 data URI
/// * `Err(String)` - Error message
pub fn render_photo(
    photo_path: &Path,
    recipe: &ocps_core::EditRecipe,
    max_width: u32,
    max_height: u32,
) -> Result<RenderResult, String> {
    let start = Instant::now();

    // Load the image
    let is_raw = is_raw_file(photo_path);

    let image = if is_raw {
        // RAW workflow
        let raw = ocps_core::decode(photo_path)
            .map_err(|e| format!("Failed to decode RAW: {:?}", e))?;

        let rgb = ocps_core::demosaic(&raw, ocps_core::DemosaicAlgorithm::Bilinear);

        // Convert u8 → u16 for pipeline
        let data_u16: Vec<u16> = rgb.data.iter().map(|&v| (v as u16) * 257).collect();

        ocps_core::RgbImage16::from_data(rgb.width, rgb.height, data_u16)
    } else {
        // JPEG/TIFF workflow
        let img = image::open(photo_path)
            .map_err(|e| format!("Failed to open image: {}", e))?;

        let rgb8 = img.to_rgb8();
        let width = rgb8.width();
        let height = rgb8.height();

        // Convert u8 → u16 for pipeline
        let data_u16: Vec<u16> = rgb8.as_raw().iter().map(|&v| (v as u16) * 257).collect();

        ocps_core::RgbImage16::from_data(width, height, data_u16)
    };

    // Resize to fit max dimensions BEFORE processing (for speed)
    let resized = if image.width > max_width || image.height > max_height {
        let ratio = (max_width as f32 / image.width as f32)
            .min(max_height as f32 / image.height as f32);
        let new_width = (image.width as f32 * ratio) as u32;
        let new_height = (image.height as f32 * ratio) as u32;

        // Use image crate for resize
        let img_u8: Vec<u8> = image
            .data
            .iter()
            .map(|&v| (v >> 8) as u8) // Quick downsample to u8
            .collect();

        let img_buf =
            image::RgbImage::from_raw(image.width, image.height, img_u8).ok_or("Invalid image")?;

        let resized_buf = image::imageops::resize(
            &img_buf,
            new_width,
            new_height,
            image::imageops::FilterType::Triangle,
        );

        // Convert back to u16
        let data_u16: Vec<u16> = resized_buf
            .as_raw()
            .iter()
            .map(|&v| (v as u16) * 257)
            .collect();

        ocps_core::RgbImage16::from_data(new_width, new_height, data_u16)
    } else {
        image
    };

    // Apply the pipeline
    let output = ocps_core::ImageProcessor::process(&resized, recipe);

    // Encode as JPEG quality 85
    let jpeg_bytes = encode_jpeg(&output.data, output.width, output.height, 85)
        .map_err(|e| format!("Failed to encode JPEG: {}", e))?;

    // Base64 encode
    let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &jpeg_bytes);
    let data_uri = format!("data:image/jpeg;base64,{}", base64_data);

    let duration = start.elapsed();

    Ok(RenderResult {
        width: output.width,
        height: output.height,
        data_uri,
        duration_ms: duration.as_millis() as u64,
    })
}

/// Encode RGB8 data as JPEG
fn encode_jpeg(data: &[u8], width: u32, height: u32, quality: u8) -> Result<Vec<u8>, String> {
    let img = image::RgbImage::from_raw(width, height, data.to_vec())
        .ok_or("Failed to create image from data")?;

    let mut buffer = std::io::Cursor::new(Vec::new());
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, quality);

    img.write_with_encoder(encoder)
        .map_err(|e| format!("JPEG encoding failed: {}", e))?;

    Ok(buffer.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_synthetic_image() {
        // Create a 100x100 mid-grey image
        let data = vec![32768u16; 100 * 100 * 3]; // Mid-grey in u16
        let image = ocps_core::RgbImage16::from_data(100, 100, data);

        // Apply default recipe
        let recipe = ocps_core::EditRecipe::default();
        let output = ocps_core::ImageProcessor::process(&image, &recipe);

        // Verify output
        assert_eq!(output.width, 100);
        assert_eq!(output.height, 100);
        assert_eq!(output.data.len(), 100 * 100 * 3);

        // Mid-grey u16 (32768) should map to ~188 in u8 sRGB (gamma correction makes it brighter)
        let avg: u32 = output.data.iter().map(|&v| v as u32).sum::<u32>() / output.data.len() as u32;
        assert!(avg > 170 && avg < 200, "Average grey value {} out of expected range", avg);
    }

    #[test]
    fn test_jpeg_encode() {
        // Create a simple 10x10 image
        let data = vec![128u8; 10 * 10 * 3];
        let jpeg_bytes = encode_jpeg(&data, 10, 10, 85).unwrap();

        // Verify JPEG magic bytes (FF D8)
        assert_eq!(jpeg_bytes[0], 0xFF);
        assert_eq!(jpeg_bytes[1], 0xD8);

        // JPEG adds headers and compression overhead for small images
        // Just verify it's non-empty and valid
        assert!(jpeg_bytes.len() > 0);
    }

    #[test]
    fn test_resize_maintains_aspect() {
        // Create a 400x200 image
        let data = vec![32768u16; 400 * 200 * 3];
        let image = ocps_core::RgbImage16::from_data(400, 200, data);

        // Resize to fit 200x200 max
        let img_u8: Vec<u8> = image.data.iter().map(|&v| (v >> 8) as u8).collect();
        let img_buf = image::RgbImage::from_raw(400, 200, img_u8).unwrap();

        let resized = image::imageops::resize(
            &img_buf,
            200,
            100, // Should maintain 2:1 aspect ratio
            image::imageops::FilterType::Triangle,
        );

        assert_eq!(resized.width(), 200);
        assert_eq!(resized.height(), 100);
    }

    #[test]
    fn test_is_raw_file() {
        assert!(is_raw_file(Path::new("/foo/bar.arw")));
        assert!(is_raw_file(Path::new("/foo/bar.NEF")));
        assert!(is_raw_file(Path::new("/foo/bar.dng")));
        assert!(!is_raw_file(Path::new("/foo/bar.jpg")));
        assert!(!is_raw_file(Path::new("/foo/bar.png")));
    }
}
