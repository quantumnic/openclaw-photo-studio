//! Thumbnail generation service
//!
//! Generates thumbnails from RAW files and regular images (JPEG/PNG).
//! Outputs base64-encoded JPEG for web display.

use crate::pipeline::types::{EditRecipe, RgbImage16, RgbImage8};
use crate::pipeline::ImageProcessor;
use crate::raw::{decode, demosaic::demosaic_bilinear};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ThumbnailError {
    #[error("RAW decode error: {0}")]
    RawDecode(String),

    #[error("Image load error: {0}")]
    ImageLoad(String),

    #[error("JPEG encode error: {0}")]
    JpegEncode(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Request for thumbnail generation
#[derive(Debug, Clone)]
pub struct ThumbnailRequest {
    pub photo_path: String,
    pub max_size: u32,   // max dimension (width or height)
    pub quality: u32,    // JPEG quality 0-100
}

/// Generated thumbnail result
#[derive(Debug, Clone)]
pub struct ThumbnailResult {
    pub data_base64: String,
    pub width: u32,
    pub height: u32,
    pub from_cache: bool,
}

/// Convert image crate's DynamicImage to RgbImage16 for pipeline processing
fn dynamic_image_to_rgb16(img: image::DynamicImage) -> RgbImage16 {
    let rgb = img.to_rgb16();
    RgbImage16 {
        width: rgb.width(),
        height: rgb.height(),
        data: rgb.into_raw(),
    }
}

/// Resize an RGB8 image to fit within max_size while preserving aspect ratio
fn resize_rgb8(image: &RgbImage8, max_size: u32) -> RgbImage8 {
    let width = image.width;
    let height = image.height;

    // Check if resize is needed
    if width <= max_size && height <= max_size {
        return image.clone();
    }

    // Calculate new dimensions preserving aspect ratio
    let (new_width, new_height) = if width > height {
        let ratio = max_size as f32 / width as f32;
        (max_size, (height as f32 * ratio).round() as u32)
    } else {
        let ratio = max_size as f32 / height as f32;
        ((width as f32 * ratio).round() as u32, max_size)
    };

    // Ensure dimensions are at least 1
    let new_width = new_width.max(1);
    let new_height = new_height.max(1);

    // Use image crate for resizing (fast bilinear)
    let img = image::RgbImage::from_raw(width, height, image.data.clone())
        .expect("Failed to create RgbImage from data");

    let resized = image::imageops::resize(
        &img,
        new_width,
        new_height,
        image::imageops::FilterType::Triangle, // Fast bilinear filter
    );

    RgbImage8 {
        width: new_width,
        height: new_height,
        data: resized.into_raw(),
    }
}

/// Encode RGB8 image as JPEG and return base64
fn encode_jpeg_base64(image: &RgbImage8, quality: u32) -> Result<String, ThumbnailError> {
    use std::io::Cursor;

    let img = image::RgbImage::from_raw(image.width, image.height, image.data.clone())
        .ok_or_else(|| ThumbnailError::JpegEncode("Invalid image data".to_string()))?;

    let mut buffer = Cursor::new(Vec::new());

    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, quality as u8);

    img.write_with_encoder(encoder)
        .map_err(|e| ThumbnailError::JpegEncode(format!("JPEG encode failed: {}", e)))?;

    let jpeg_data = buffer.into_inner();
    let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &jpeg_data);

    Ok(base64_data)
}

/// Generate thumbnail from a photo file
///
/// Supports:
/// - RAW files (via rawloader + demosaic)
/// - JPEG/PNG files (via image crate)
///
/// Process:
/// 1. Try to decode as RAW
/// 2. If RAW decode fails, try as regular image (JPEG/PNG)
/// 3. Apply default EditRecipe (no edits for thumbnails)
/// 4. Resize to max_size
/// 5. Encode as JPEG
/// 6. Base64 encode
pub fn generate_thumbnail(req: &ThumbnailRequest) -> Result<ThumbnailResult, ThumbnailError> {
    let path = Path::new(&req.photo_path);

    // Try RAW decode first
    let rgb16 = match decode(path) {
        Ok(raw) => {
            // RAW decode successful - demosaic and convert to RGB16
            let rgb = demosaic_bilinear(&raw);

            // Convert u8 → u16 for pipeline (multiply by 257)
            let data_u16: Vec<u16> = rgb.data.iter().map(|&v| (v as u16) * 257).collect();

            RgbImage16::from_data(rgb.width, rgb.height, data_u16)
        }
        Err(_) => {
            // RAW decode failed - try as regular image
            let img = image::open(path)
                .map_err(|e| ThumbnailError::ImageLoad(format!("Failed to load image: {}", e)))?;

            dynamic_image_to_rgb16(img)
        }
    };

    // Apply default EditRecipe (no edits for thumbnails)
    let recipe = EditRecipe::default();
    let rgb8 = ImageProcessor::process(&rgb16, &recipe);

    // Resize to fit max_size
    let resized = resize_rgb8(&rgb8, req.max_size);

    // Encode as JPEG and base64
    let base64_data = encode_jpeg_base64(&resized, req.quality)?;

    Ok(ThumbnailResult {
        data_base64: base64_data,
        width: resized.width,
        height: resized.height,
        from_cache: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_png() -> std::path::PathBuf {
        use std::io::Write;
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("ocps_test_thumb.png");

        // Create a 4x4 RGB PNG
        let img = image::RgbImage::from_fn(4, 4, |x, y| {
            image::Rgb([
                ((x * 64) % 256) as u8,
                ((y * 64) % 256) as u8,
                128,
            ])
        });

        img.save(&path).unwrap();
        path
    }

    #[test]
    fn test_generate_thumbnail_from_png() {
        let png_path = create_test_png();

        let req = ThumbnailRequest {
            photo_path: png_path.to_string_lossy().to_string(),
            max_size: 256,
            quality: 80,
        };

        let result = generate_thumbnail(&req);
        assert!(result.is_ok());

        let thumb = result.unwrap();
        assert!(!thumb.data_base64.is_empty());
        assert!(thumb.width <= 256);
        assert!(thumb.height <= 256);

        // Cleanup
        let _ = std::fs::remove_file(png_path);
    }

    #[test]
    fn test_generate_thumbnail_size_limit() {
        let png_path = create_test_png();

        let req = ThumbnailRequest {
            photo_path: png_path.to_string_lossy().to_string(),
            max_size: 2, // Very small
            quality: 80,
        };

        let result = generate_thumbnail(&req);
        assert!(result.is_ok());

        let thumb = result.unwrap();
        assert!(thumb.width <= 2);
        assert!(thumb.height <= 2);

        // Cleanup
        let _ = std::fs::remove_file(png_path);
    }

    #[test]
    fn test_generate_thumbnail_nonexistent() {
        let req = ThumbnailRequest {
            photo_path: "/tmp/nonexistent_file_12345.jpg".to_string(),
            max_size: 256,
            quality: 80,
        };

        let result = generate_thumbnail(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_resize_rgb8_no_resize_needed() {
        let img = RgbImage8 {
            width: 100,
            height: 100,
            data: vec![128; 100 * 100 * 3],
        };

        let resized = resize_rgb8(&img, 200);
        assert_eq!(resized.width, 100);
        assert_eq!(resized.height, 100);
    }

    #[test]
    fn test_resize_rgb8_width_larger() {
        let img = RgbImage8 {
            width: 200,
            height: 100,
            data: vec![128; 200 * 100 * 3],
        };

        let resized = resize_rgb8(&img, 100);
        assert_eq!(resized.width, 100);
        assert_eq!(resized.height, 50);
    }

    #[test]
    fn test_resize_rgb8_height_larger() {
        let img = RgbImage8 {
            width: 100,
            height: 200,
            data: vec![128; 100 * 200 * 3],
        };

        let resized = resize_rgb8(&img, 100);
        assert_eq!(resized.width, 50);
        assert_eq!(resized.height, 100);
    }

    #[test]
    fn test_encode_jpeg_base64() {
        let img = RgbImage8 {
            width: 2,
            height: 2,
            data: vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255],
        };

        let result = encode_jpeg_base64(&img, 80);
        assert!(result.is_ok());

        let base64 = result.unwrap();
        assert!(!base64.is_empty());
        // Base64 should be valid
        assert!(base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &base64).is_ok());
    }

    #[test]
    fn test_dynamic_image_to_rgb16() {
        use image::GenericImageView;

        let img = image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(4, 4, |_, _| {
            image::Rgb([128, 128, 128])
        }));

        let rgb16 = dynamic_image_to_rgb16(img);
        assert_eq!(rgb16.width, 4);
        assert_eq!(rgb16.height, 4);
        assert_eq!(rgb16.data.len(), 4 * 4 * 3);
    }
}
