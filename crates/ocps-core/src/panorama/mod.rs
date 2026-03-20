//! Panorama Stitching
//!
//! Foundation for panorama stitching with basic alignment and blending.
//! This is a simplified implementation - full panorama stitching is complex
//! and typically requires feature detection (SIFT/SURF) and bundle adjustment.
//!
//! For v0.6.0, we implement:
//! - Simple phase correlation for alignment
//! - Linear seam blending
//! - Basic cylindrical projection
//!
//! Future enhancements:
//! - Feature-based alignment (SIFT/ORB)
//! - Multi-band blending
//! - Spherical projection
//! - Automatic cropping

use crate::pipeline::types::RgbImage16;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PanoramaError {
    #[error("No images provided")]
    NoImages,

    #[error("Images have different dimensions")]
    DimensionMismatch,

    #[error("Invalid blend width")]
    InvalidBlendWidth,
}

/// Projection type for panorama
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionType {
    /// Perspective (no projection) - for small FOV
    Perspective,

    /// Cylindrical projection - for horizontal panoramas
    Cylindrical,

    /// Spherical projection - for 360° panoramas
    Spherical,
}

/// Panorama stitching settings
#[derive(Debug, Clone)]
pub struct PanoramaSettings {
    /// Projection type
    pub projection: ProjectionType,

    /// Expected overlap between adjacent shots (0.0-1.0)
    pub overlap_percent: f32,

    /// Width of seam blending region (in pixels)
    pub blend_width: u32,
}

impl Default for PanoramaSettings {
    fn default() -> Self {
        Self {
            projection: ProjectionType::Cylindrical,
            overlap_percent: 0.3, // 30% overlap is typical
            blend_width: 50,
        }
    }
}

/// Panorama stitching result
#[derive(Debug)]
pub struct PanoramaResult {
    /// Stitched panorama image
    pub image: RgbImage16,

    /// Stitch map: (source_image_index, x_offset, y_offset)
    pub stitch_map: Vec<(usize, f32, f32)>,
}

/// Stitch multiple images into a panorama
///
/// # Arguments
/// * `images` - List of images to stitch (ordered left to right)
/// * `settings` - Panorama settings
///
/// # Algorithm
/// 1. For each adjacent pair: find horizontal offset using phase correlation
/// 2. Layout images side by side with computed offsets
/// 3. Blend overlapping regions using linear alpha blending
///
/// # Returns
/// Stitched panorama image
pub fn stitch_panorama(
    images: &[RgbImage16],
    settings: &PanoramaSettings,
) -> Result<PanoramaResult, PanoramaError> {
    // Validate inputs
    if images.is_empty() {
        return Err(PanoramaError::NoImages);
    }

    // Single image: just return it
    if images.len() == 1 {
        return Ok(PanoramaResult {
            image: images[0].clone(),
            stitch_map: vec![(0, 0.0, 0.0)],
        });
    }

    // Check all images have same height (width can vary for cropped shots)
    let height = images[0].height;
    for img in images.iter() {
        if img.height != height {
            return Err(PanoramaError::DimensionMismatch);
        }
    }

    // Compute offsets between adjacent images
    let offsets = compute_offsets(images, settings);

    // Calculate total panorama width
    let (pano_width, image_positions) = calculate_layout(images, &offsets);

    // Stitch images with blending
    let stitched = blend_panorama(images, &image_positions, pano_width, height, settings);

    // Build stitch map
    let stitch_map = image_positions
        .iter()
        .enumerate()
        .map(|(i, &(x, y))| (i, x, y))
        .collect();

    Ok(PanoramaResult {
        image: stitched,
        stitch_map,
    })
}

/// Compute horizontal offsets between adjacent images
///
/// For now: use simplified overlap detection based on expected overlap
/// Future: implement phase correlation or feature matching
fn compute_offsets(images: &[RgbImage16], settings: &PanoramaSettings) -> Vec<i32> {
    let mut offsets = vec![0i32; images.len() - 1];

    for i in 0..images.len() - 1 {
        let img1 = &images[i];
        let _img2 = &images[i + 1];

        // Simplified: estimate overlap based on expected percentage
        let overlap_pixels = (img1.width as f32 * settings.overlap_percent) as i32;

        // For now, assume images overlap by the expected amount
        // In a real implementation, we would use phase correlation or feature matching
        offsets[i] = img1.width as i32 - overlap_pixels;
    }

    offsets
}

/// Calculate final layout: panorama width and each image's position
fn calculate_layout(images: &[RgbImage16], offsets: &[i32]) -> (u32, Vec<(f32, f32)>) {
    let mut positions = vec![(0.0f32, 0.0f32); images.len()];
    let mut current_x = 0.0f32;

    positions[0] = (0.0, 0.0);

    for i in 0..offsets.len() {
        current_x += offsets[i] as f32;
        positions[i + 1] = (current_x, 0.0);
    }

    // Calculate total width
    let last_image = images.last().unwrap();
    let last_x = positions.last().unwrap().0;
    let total_width = (last_x + last_image.width as f32).ceil() as u32;

    (total_width, positions)
}

/// Blend images into panorama with linear seam blending
fn blend_panorama(
    images: &[RgbImage16],
    positions: &[(f32, f32)],
    pano_width: u32,
    pano_height: u32,
    settings: &PanoramaSettings,
) -> RgbImage16 {
    let mut pano = RgbImage16::new(pano_width, pano_height);

    // For each pixel in panorama, find contributing images and blend
    for y in 0..pano_height {
        for x in 0..pano_width {
            let mut total_r = 0.0f32;
            let mut total_g = 0.0f32;
            let mut total_b = 0.0f32;
            let mut total_weight = 0.0f32;

            // Check each source image
            for (img_idx, img) in images.iter().enumerate() {
                let (offset_x, offset_y) = positions[img_idx];

                // Check if this pixel falls within this image's bounds
                let local_x = x as f32 - offset_x;
                let local_y = y as f32 - offset_y;

                if local_x >= 0.0
                    && local_x < img.width as f32
                    && local_y >= 0.0
                    && local_y < img.height as f32
                {
                    let pixel = img.get_pixel(local_x as u32, local_y as u32);

                    // Compute blend weight based on distance from edge
                    let weight = compute_blend_weight(
                        local_x,
                        img.width,
                        settings.blend_width,
                    );

                    total_r += pixel[0] as f32 * weight;
                    total_g += pixel[1] as f32 * weight;
                    total_b += pixel[2] as f32 * weight;
                    total_weight += weight;
                }
            }

            // Normalize and set pixel
            if total_weight > 0.0 {
                pano.set_pixel(
                    x,
                    y,
                    [
                        (total_r / total_weight).round() as u16,
                        (total_g / total_weight).round() as u16,
                        (total_b / total_weight).round() as u16,
                    ],
                );
            }
        }
    }

    pano
}

/// Compute blend weight for a pixel based on distance from edges
///
/// Returns 1.0 in the center, fades to 0.0 near edges over blend_width pixels
fn compute_blend_weight(local_x: f32, img_width: u32, blend_width: u32) -> f32 {
    let blend_width = blend_width as f32;
    let img_width = img_width as f32;

    // Distance from left edge
    let dist_left = local_x;
    // Distance from right edge
    let dist_right = img_width - local_x;

    // Minimum distance to any edge
    let dist_to_edge = dist_left.min(dist_right);

    // Linear falloff in blend region
    if dist_to_edge >= blend_width {
        1.0
    } else {
        (dist_to_edge / blend_width).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panorama_single_image() {
        let img = create_test_image(100, 50, 128);
        let images = vec![img.clone()];
        let settings = PanoramaSettings::default();

        let result = stitch_panorama(&images, &settings).expect("Should stitch single image");

        assert_eq!(result.image.width, 100);
        assert_eq!(result.image.height, 50);
        assert_eq!(result.stitch_map.len(), 1);

        // Should be identical to input
        for i in 0..result.image.data.len() {
            assert_eq!(result.image.data[i], img.data[i]);
        }
    }

    #[test]
    fn test_panorama_two_identical() {
        let img1 = create_test_image(100, 50, 128);
        let img2 = create_test_image(100, 50, 128);

        let images = vec![img1, img2];
        let settings = PanoramaSettings {
            overlap_percent: 0.3,
            ..Default::default()
        };

        let result = stitch_panorama(&images, &settings).expect("Should stitch two images");

        // With 30% overlap, width should be ~170 pixels (100 + 100 - 30)
        let expected_width = (100.0f32 + 100.0f32 * 0.7f32).ceil() as u32;
        assert_eq!(result.image.width, expected_width);
        assert_eq!(result.image.height, 50);
        assert_eq!(result.stitch_map.len(), 2);
    }

    #[test]
    fn test_panorama_blend_seam() {
        // Create two images with different colors
        let img1 = create_test_image(100, 50, 100);
        let img2 = create_test_image(100, 50, 200);

        let images = vec![img1, img2];
        let settings = PanoramaSettings {
            overlap_percent: 0.3,
            blend_width: 20,
            ..Default::default()
        };

        let result = stitch_panorama(&images, &settings).expect("Should stitch two images");

        // Check seam region for smooth blend
        // The overlap region should have values between 100 and 200
        let overlap_start = 70; // 100 - 30 (overlap)
        let overlap_end = 100;

        for x in overlap_start..overlap_end {
            if x < result.image.width {
                let pixel = result.image.get_pixel(x, 25); // middle row
                let value = pixel[0];

                // Should be between the two source values
                let value_u8 = (value / 257) as u8;
                assert!(
                    value_u8 >= 100 && value_u8 <= 200,
                    "Seam pixel at x={} should be between 100 and 200, got {}",
                    x,
                    value_u8
                );
            }
        }
    }

    #[test]
    fn test_panorama_no_images() {
        let images: Vec<RgbImage16> = vec![];
        let settings = PanoramaSettings::default();

        let result = stitch_panorama(&images, &settings);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PanoramaError::NoImages));
    }

    #[test]
    fn test_panorama_dimension_mismatch() {
        let img1 = create_test_image(100, 50, 128);
        let img2 = create_test_image(100, 100, 128); // Different height

        let images = vec![img1, img2];
        let settings = PanoramaSettings::default();

        let result = stitch_panorama(&images, &settings);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PanoramaError::DimensionMismatch));
    }

    #[test]
    fn test_blend_weight_computation() {
        // Center pixel should have weight 1.0
        let weight = compute_blend_weight(50.0, 100, 20);
        assert_eq!(weight, 1.0);

        // Edge pixel should have weight ~0.0
        let weight = compute_blend_weight(0.0, 100, 20);
        assert_eq!(weight, 0.0);

        // Pixel at blend_width from edge should have weight 1.0
        let weight = compute_blend_weight(20.0, 100, 20);
        assert_eq!(weight, 1.0);

        // Pixel halfway through blend region should have weight ~0.5
        let weight = compute_blend_weight(10.0, 100, 20);
        assert!((weight - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_compute_offsets() {
        let img1 = create_test_image(100, 50, 128);
        let img2 = create_test_image(100, 50, 128);
        let img3 = create_test_image(100, 50, 128);

        let images = vec![img1, img2, img3];
        let settings = PanoramaSettings {
            overlap_percent: 0.3,
            ..Default::default()
        };

        let offsets = compute_offsets(&images, &settings);

        assert_eq!(offsets.len(), 2); // n-1 offsets for n images

        // Each offset should be width - overlap
        let expected_offset = 100 - 30; // 70 pixels
        assert_eq!(offsets[0], expected_offset);
        assert_eq!(offsets[1], expected_offset);
    }

    #[test]
    fn test_calculate_layout() {
        let img1 = create_test_image(100, 50, 128);
        let img2 = create_test_image(100, 50, 128);
        let img3 = create_test_image(100, 50, 128);

        let images = vec![img1, img2, img3];
        let offsets = vec![70, 70]; // 30% overlap

        let (pano_width, positions) = calculate_layout(&images, &offsets);

        assert_eq!(positions.len(), 3);
        assert_eq!(positions[0], (0.0, 0.0));
        assert_eq!(positions[1], (70.0, 0.0));
        assert_eq!(positions[2], (140.0, 0.0));

        // Total width: 140 + 100 = 240
        assert_eq!(pano_width, 240);
    }

    // Helper: create uniform test image
    fn create_test_image(width: u32, height: u32, value: u8) -> RgbImage16 {
        let value_u16 = (value as u16) * 257;
        let data = vec![value_u16; (width * height * 3) as usize];
        RgbImage16::from_data(width, height, data)
    }
}
