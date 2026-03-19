//! Thumbnail extraction from RAW files
//!
//! Many RAW files contain embedded JPEG thumbnails for fast preview.
//! This module extracts them when available, or generates them via
//! fast demosaicing + downscaling.

use super::{decode, demosaic::{demosaic, DemosaicAlgorithm, RgbImage}, RawDecodeError};
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb};
use std::io::Cursor;
use std::path::Path;

/// Thumbnail extraction error
#[derive(Debug, thiserror::Error)]
pub enum ThumbnailError {
    #[error("RAW decode error: {0}")]
    RawDecode(#[from] RawDecodeError),

    #[error("Image encoding error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("No thumbnail available")]
    NoThumbnail,
}

/// Extract or generate a thumbnail from a RAW file
///
/// # Strategy
/// 1. Try to extract embedded JPEG thumbnail (fastest)
/// 2. If none available, decode RAW + demosaic + downscale
///
/// # Arguments
/// * `path` - Path to RAW file
/// * `max_size` - Maximum dimension (width or height) of the output thumbnail
///
/// # Returns
/// * JPEG bytes ready for display or caching
pub fn extract_thumbnail(path: &Path, max_size: u32) -> Result<Vec<u8>, ThumbnailError> {
    // Try embedded thumbnail first via rawloader
    if let Ok(embedded) = try_extract_embedded_thumbnail(path, max_size) {
        return Ok(embedded);
    }

    // Fallback: decode + demosaic + downscale
    generate_thumbnail_from_raw(path, max_size)
}

/// Try to extract an embedded JPEG thumbnail from the RAW file
///
/// Many RAW formats (especially DNG, ARW, NEF) include full-resolution
/// or medium-resolution JPEG previews embedded in the file.
fn try_extract_embedded_thumbnail(path: &Path, max_size: u32) -> Result<Vec<u8>, ThumbnailError> {
    // Use rawloader to get the full raw file structure
    let raw_file = rawloader::decode_file(path)
        .map_err(|e| RawDecodeError::RawloaderError(e.to_string()))?;

    // Check if there's a thumbnail available
    // rawloader provides thumbnail as a JPEG in some formats
    // Note: rawloader 0.37 doesn't expose embedded thumbnails directly,
    // so we'll return an error to trigger the fallback path
    //
    // TODO: In a future version, we could use a lower-level RAW parser
    // to extract embedded JPEGs directly without full decode

    Err(ThumbnailError::NoThumbnail)
}

/// Generate a thumbnail by decoding the RAW, demosaicing, and downscaling
fn generate_thumbnail_from_raw(path: &Path, max_size: u32) -> Result<Vec<u8>, ThumbnailError> {
    // Decode the RAW file
    let raw = decode(path)?;

    // For thumbnail generation, we can use a fast demosaic algorithm
    // Bilinear is good enough for small previews
    let rgb = demosaic(&raw, DemosaicAlgorithm::Bilinear);

    // Convert to image crate's format
    let img_buffer = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(
        rgb.width,
        rgb.height,
        rgb.data,
    )
    .ok_or_else(|| {
        ThumbnailError::ImageError(image::ImageError::Parameter(
            image::error::ParameterError::from_kind(
                image::error::ParameterErrorKind::DimensionMismatch,
            ),
        ))
    })?;

    let img = DynamicImage::ImageRgb8(img_buffer);

    // Calculate thumbnail dimensions maintaining aspect ratio
    let (orig_width, orig_height) = (img.width(), img.height());
    let (thumb_width, thumb_height) = if orig_width > orig_height {
        let ratio = max_size as f32 / orig_width as f32;
        (max_size, (orig_height as f32 * ratio).round() as u32)
    } else {
        let ratio = max_size as f32 / orig_height as f32;
        ((orig_width as f32 * ratio).round() as u32, max_size)
    };

    // Resize using fast algorithm (Nearest for speed, or Triangle for better quality)
    let thumbnail = img.resize(
        thumb_width,
        thumb_height,
        image::imageops::FilterType::Triangle,
    );

    // Encode as JPEG
    let mut jpeg_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut jpeg_bytes);

    thumbnail.write_to(&mut cursor, ImageFormat::Jpeg)?;

    Ok(jpeg_bytes)
}

/// Quick thumbnail generation for bulk operations
///
/// Uses the fastest possible demosaic (center pixel) for speed.
/// Quality is lower but acceptable for grid view thumbnails.
pub fn quick_thumbnail(path: &Path, max_size: u32) -> Result<Vec<u8>, ThumbnailError> {
    // Try embedded first
    if let Ok(embedded) = try_extract_embedded_thumbnail(path, max_size) {
        return Ok(embedded);
    }

    // Decode RAW
    let raw = decode(path)?;

    // Use center pixel demosaic (fastest)
    let rgb = demosaic(&raw, DemosaicAlgorithm::CenterPixel);

    // Convert to image
    let img_buffer = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(
        rgb.width,
        rgb.height,
        rgb.data,
    )
    .ok_or_else(|| {
        ThumbnailError::ImageError(image::ImageError::Parameter(
            image::error::ParameterError::from_kind(
                image::error::ParameterErrorKind::DimensionMismatch,
            ),
        ))
    })?;

    let img = DynamicImage::ImageRgb8(img_buffer);

    // Calculate dimensions
    let (orig_width, orig_height) = (img.width(), img.height());
    let (thumb_width, thumb_height) = if orig_width > orig_height {
        let ratio = max_size as f32 / orig_width as f32;
        (max_size, (orig_height as f32 * ratio).round() as u32)
    } else {
        let ratio = max_size as f32 / orig_height as f32;
        ((orig_width as f32 * ratio).round() as u32, max_size)
    };

    // Resize with fastest filter
    let thumbnail = img.resize(
        thumb_width,
        thumb_height,
        image::imageops::FilterType::Nearest,
    );

    // Encode as JPEG with lower quality for speed
    let mut jpeg_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut jpeg_bytes);

    thumbnail.write_to(&mut cursor, ImageFormat::Jpeg)?;

    Ok(jpeg_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_thumbnail_nonexistent() {
        let result = extract_thumbnail(Path::new("/tmp/nonexistent.dng"), 256);
        assert!(result.is_err());
    }

    #[test]
    fn test_quick_thumbnail_nonexistent() {
        let result = quick_thumbnail(Path::new("/tmp/nonexistent.dng"), 256);
        assert!(result.is_err());
    }

    // Note: Real thumbnail extraction tests require actual RAW files
    // which we don't commit to the repo. Manual testing with real files
    // should verify this functionality.
}
