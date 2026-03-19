//! Image resizing functionality

use image::{imageops::FilterType, RgbImage};

/// Resize an image to fit within a maximum long edge dimension
///
/// Uses Lanczos3 filtering for high-quality downsampling.
/// Does NOT enlarge images - if the image is already smaller, it's returned unchanged.
///
/// # Arguments
/// * `rgb_data` - 8-bit sRGB data, RGB interleaved
/// * `width` - Current image width
/// * `height` - Current image height
/// * `long_edge` - Maximum dimension for the long edge
///
/// # Returns
/// Tuple of (resized_data, new_width, new_height)
pub fn resize_long_edge(
    rgb_data: &[u8],
    width: u32,
    height: u32,
    long_edge: u32,
) -> (Vec<u8>, u32, u32) {
    // Determine current long edge
    let current_long_edge = width.max(height);

    // Don't enlarge - if already smaller, return unchanged
    if current_long_edge <= long_edge {
        return (rgb_data.to_vec(), width, height);
    }

    // Calculate new dimensions maintaining aspect ratio
    let (new_width, new_height) = if width > height {
        // Landscape: width is the long edge
        let scale = long_edge as f64 / width as f64;
        (long_edge, (height as f64 * scale).round() as u32)
    } else {
        // Portrait or square: height is the long edge
        let scale = long_edge as f64 / height as f64;
        ((width as f64 * scale).round() as u32, long_edge)
    };

    // Create image from raw data
    let img = match RgbImage::from_raw(width, height, rgb_data.to_vec()) {
        Some(img) => img,
        None => return (rgb_data.to_vec(), width, height), // Fallback
    };

    // Resize with Lanczos3 filter
    let resized = image::imageops::resize(&img, new_width, new_height, FilterType::Lanczos3);

    (resized.into_raw(), new_width, new_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize_landscape_long_edge() {
        // 4000x2000 landscape image
        let width = 4000;
        let height = 2000;
        let rgb_data = vec![128u8; (width * height * 3) as usize];

        let (resized, new_w, new_h) = resize_long_edge(&rgb_data, width, height, 2000);

        // Should resize to 2000x1000
        assert_eq!(new_w, 2000);
        assert_eq!(new_h, 1000);
        assert_eq!(resized.len(), (2000 * 1000 * 3) as usize);
    }

    #[test]
    fn test_resize_portrait_long_edge() {
        // 2000x4000 portrait image
        let width = 2000;
        let height = 4000;
        let rgb_data = vec![128u8; (width * height * 3) as usize];

        let (resized, new_w, new_h) = resize_long_edge(&rgb_data, width, height, 2000);

        // Should resize to 1000x2000
        assert_eq!(new_w, 1000);
        assert_eq!(new_h, 2000);
        assert_eq!(resized.len(), (1000 * 2000 * 3) as usize);
    }

    #[test]
    fn test_resize_no_enlarge() {
        // 800x600 image
        let width = 800;
        let height = 600;
        let rgb_data = vec![64u8; (width * height * 3) as usize];

        let (resized, new_w, new_h) = resize_long_edge(&rgb_data, width, height, 2000);

        // Should stay 800x600 (no enlargement)
        assert_eq!(new_w, 800);
        assert_eq!(new_h, 600);
        assert_eq!(resized.len(), (800 * 600 * 3) as usize);
    }

    #[test]
    fn test_resize_square() {
        // 1000x1000 square image
        let width = 1000;
        let height = 1000;
        let rgb_data = vec![200u8; (width * height * 3) as usize];

        let (resized, new_w, new_h) = resize_long_edge(&rgb_data, width, height, 500);

        // Should resize to 500x500
        assert_eq!(new_w, 500);
        assert_eq!(new_h, 500);
        assert_eq!(resized.len(), (500 * 500 * 3) as usize);
    }

    #[test]
    fn test_resize_exact_match() {
        // 2000x1000 image with long_edge = 2000
        let width = 2000;
        let height = 1000;
        let rgb_data = vec![100u8; (width * height * 3) as usize];

        let (resized, new_w, new_h) = resize_long_edge(&rgb_data, width, height, 2000);

        // Should stay unchanged (already at target size)
        assert_eq!(new_w, 2000);
        assert_eq!(new_h, 1000);
    }
}
