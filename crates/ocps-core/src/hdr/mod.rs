//! HDR Merge — Exposure Fusion
//!
//! Merges multiple exposures using Mertens exposure fusion algorithm.
//! This is simpler than true HDR tone mapping but produces good results
//! for typical bracketed exposure sequences.
//!
//! References:
//! - Mertens et al. "Exposure Fusion" (2007)
//! - https://mericam.github.io/exposure_fusion/index.html

use crate::pipeline::types::{RgbImage16, RgbImage8};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HdrError {
    #[error("No exposures provided")]
    NoExposures,

    #[error("Base exposure index {0} out of range")]
    InvalidBaseIndex(usize),

    #[error("Images have different dimensions")]
    DimensionMismatch,

    #[error("Invalid deghosting value (must be 0.0-1.0)")]
    InvalidDeghosting,
}

/// HDR merge settings
#[derive(Debug, Clone)]
pub struct HdrMergeSettings {
    /// Deghosting strength (0.0 = none, 1.0 = aggressive)
    pub deghosting: f32,

    /// Enable auto-alignment (for handheld shots)
    pub auto_align: bool,

    /// Which input to use as base exposure (for alignment reference)
    pub base_exposure_index: usize,
}

impl Default for HdrMergeSettings {
    fn default() -> Self {
        Self {
            deghosting: 0.0,
            auto_align: false,
            base_exposure_index: 0,
        }
    }
}

/// HDR merge result
#[derive(Debug)]
pub struct HdrResult {
    /// Merged image
    pub image: RgbImage16,

    /// Total exposure range covered (in EV stops)
    pub ev_range: f32,

    /// Per-pixel ghosting probability (optional, for debugging)
    pub ghost_map: Option<Vec<f32>>,
}

/// Merge multiple exposures into an HDR image using Mertens exposure fusion
///
/// # Arguments
/// * `exposures` - List of (image, ev_offset) pairs, sorted from darkest to brightest
/// * `settings` - HDR merge settings
///
/// # Algorithm
/// 1. Compute quality metrics for each pixel in each exposure:
///    - Contrast: Laplacian magnitude (edge strength)
///    - Saturation: Standard deviation of RGB channels
///    - Well-exposedness: Gaussian centered at 0.5 (prefers mid-tones)
/// 2. Normalize weights across exposures
/// 3. Blend images using weighted average
///
/// # Returns
/// HDR merged image with extended dynamic range
pub fn merge_hdr(
    exposures: &[(RgbImage16, f32)],
    settings: &HdrMergeSettings,
) -> Result<HdrResult, HdrError> {
    // Validate inputs
    if exposures.is_empty() {
        return Err(HdrError::NoExposures);
    }

    if settings.base_exposure_index >= exposures.len() {
        return Err(HdrError::InvalidBaseIndex(settings.base_exposure_index));
    }

    if !(0.0..=1.0).contains(&settings.deghosting) {
        return Err(HdrError::InvalidDeghosting);
    }

    // Check all images have same dimensions
    let (width, height) = (exposures[0].0.width, exposures[0].0.height);
    for (img, _) in exposures.iter() {
        if img.width != width || img.height != height {
            return Err(HdrError::DimensionMismatch);
        }
    }

    // Single image: just return it
    if exposures.len() == 1 {
        return Ok(HdrResult {
            image: exposures[0].0.clone(),
            ev_range: 0.0,
            ghost_map: None,
        });
    }

    // Calculate EV range
    let ev_values: Vec<f32> = exposures.iter().map(|(_, ev)| *ev).collect();
    let ev_range = ev_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
        - ev_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

    // Compute weight maps for each exposure
    let weight_maps = compute_weight_maps(exposures);

    // Normalize weights (sum to 1.0 across exposures for each pixel)
    let normalized_weights = normalize_weights(&weight_maps, width, height);

    // Blend images using normalized weights
    let merged = blend_images(exposures, &normalized_weights, width, height);

    Ok(HdrResult {
        image: merged,
        ev_range,
        ghost_map: None,
    })
}

/// Compute quality metric weights for each exposure
///
/// For each pixel in each image, compute:
/// - Contrast: Laplacian magnitude
/// - Saturation: RGB standard deviation
/// - Well-exposedness: Gaussian centered at 0.5
fn compute_weight_maps(exposures: &[(RgbImage16, f32)]) -> Vec<Vec<f32>> {
    exposures
        .iter()
        .map(|(img, _)| compute_weight_map(img))
        .collect()
}

/// Compute weight map for a single exposure
fn compute_weight_map(img: &RgbImage16) -> Vec<f32> {
    let size = (img.width * img.height) as usize;
    let mut weights = vec![0.0f32; size];

    for y in 0..img.height {
        for x in 0..img.width {
            let idx = (y * img.width + x) as usize;
            let rgb = img.get_pixel(x, y);

            // Convert to [0.0, 1.0] range
            let r = rgb[0] as f32 / 65535.0;
            let g = rgb[1] as f32 / 65535.0;
            let b = rgb[2] as f32 / 65535.0;

            // 1. Contrast (simplified: use local variance)
            let contrast = compute_local_contrast(img, x, y);

            // 2. Saturation (standard deviation of RGB)
            let mean = (r + g + b) / 3.0;
            let saturation = ((r - mean).powi(2) + (g - mean).powi(2) + (b - mean).powi(2)).sqrt();

            // 3. Well-exposedness (Gaussian centered at 0.5)
            let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            let sigma = 0.2;
            let well_exposed = (-(luminance - 0.5).powi(2) / (2.0 * sigma * sigma)).exp();

            // Combine metrics (multiplication as in Mertens paper)
            weights[idx] = (contrast + 0.01) * (saturation + 0.01) * (well_exposed + 0.01);
        }
    }

    weights
}

/// Compute local contrast at a pixel using simplified Laplacian
fn compute_local_contrast(img: &RgbImage16, x: u32, y: u32) -> f32 {
    let center = img.get_pixel(x, y);
    let center_lum = rgb_to_luminance(center);

    let mut laplacian = 0.0f32;
    let mut count = 0;

    // Sample neighbors
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx >= 0 && nx < img.width as i32 && ny >= 0 && ny < img.height as i32 {
                let neighbor = img.get_pixel(nx as u32, ny as u32);
                let neighbor_lum = rgb_to_luminance(neighbor);
                laplacian += (center_lum - neighbor_lum).abs();
                count += 1;
            }
        }
    }

    if count > 0 {
        laplacian / count as f32
    } else {
        0.0
    }
}

/// Convert RGB to luminance
fn rgb_to_luminance(rgb: [u16; 3]) -> f32 {
    let r = rgb[0] as f32 / 65535.0;
    let g = rgb[1] as f32 / 65535.0;
    let b = rgb[2] as f32 / 65535.0;
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Normalize weight maps so they sum to 1.0 across exposures for each pixel
fn normalize_weights(weight_maps: &[Vec<f32>], width: u32, height: u32) -> Vec<Vec<f32>> {
    let size = (width * height) as usize;
    let num_exposures = weight_maps.len();

    let mut normalized = vec![vec![0.0f32; size]; num_exposures];

    for i in 0..size {
        // Sum weights across all exposures
        let sum: f32 = weight_maps.iter().map(|w| w[i]).sum();

        // Normalize (avoid division by zero)
        let sum = if sum > 0.0 { sum } else { 1.0 };

        for (exposure_idx, weights) in weight_maps.iter().enumerate() {
            normalized[exposure_idx][i] = weights[i] / sum;
        }
    }

    normalized
}

/// Blend images using normalized weights
fn blend_images(
    exposures: &[(RgbImage16, f32)],
    normalized_weights: &[Vec<f32>],
    width: u32,
    height: u32,
) -> RgbImage16 {
    let mut result = RgbImage16::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;

            let mut blended_r = 0.0f32;
            let mut blended_g = 0.0f32;
            let mut blended_b = 0.0f32;

            for (exposure_idx, (img, _)) in exposures.iter().enumerate() {
                let weight = normalized_weights[exposure_idx][idx];
                let rgb = img.get_pixel(x, y);

                blended_r += rgb[0] as f32 * weight;
                blended_g += rgb[1] as f32 * weight;
                blended_b += rgb[2] as f32 * weight;
            }

            result.set_pixel(
                x,
                y,
                [
                    blended_r.round().min(65535.0) as u16,
                    blended_g.round().min(65535.0) as u16,
                    blended_b.round().min(65535.0) as u16,
                ],
            );
        }
    }

    result
}

/// Convert RgbImage8 to RgbImage16 (for testing)
pub fn image8_to_image16(img: &RgbImage8) -> RgbImage16 {
    let data_u16: Vec<u16> = img.data.iter().map(|&v| (v as u16) * 257).collect();
    RgbImage16::from_data(img.width, img.height, data_u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hdr_merge_single_image() {
        let img = create_test_image(10, 10, 128);
        let exposures = vec![(img.clone(), 0.0)];
        let settings = HdrMergeSettings::default();

        let result = merge_hdr(&exposures, &settings).expect("Should merge single image");

        assert_eq!(result.image.width, 10);
        assert_eq!(result.image.height, 10);
        assert_eq!(result.ev_range, 0.0);

        // Should be identical to input
        for i in 0..result.image.data.len() {
            assert_eq!(result.image.data[i], img.data[i]);
        }
    }

    #[test]
    fn test_hdr_merge_two_exposures() {
        // Create dark and bright exposures
        let dark = create_test_image(10, 10, 64); // Underexposed
        let bright = create_test_image(10, 10, 192); // Overexposed

        let exposures = vec![(dark.clone(), -1.0), (bright.clone(), 1.0)];
        let settings = HdrMergeSettings::default();

        let result = merge_hdr(&exposures, &settings).expect("Should merge two exposures");

        assert_eq!(result.image.width, 10);
        assert_eq!(result.image.height, 10);
        assert_eq!(result.ev_range, 2.0);

        // Result should be somewhere between dark and bright
        // Sample a few pixels
        let center_pixel = result.image.get_pixel(5, 5);
        let dark_value = (64 * 257) as u16;
        let bright_value = (192 * 257) as u16;

        // Should be between dark and bright
        for i in 0..3 {
            assert!(
                center_pixel[i] >= dark_value && center_pixel[i] <= bright_value,
                "Pixel channel {} value {} should be between {} and {}",
                i,
                center_pixel[i],
                dark_value,
                bright_value
            );
        }

        // Calculate variance in shadows and highlights
        let dark_pixel = dark.get_pixel(5, 5);
        let bright_pixel = bright.get_pixel(5, 5);
        let merged_pixel = result.image.get_pixel(5, 5);

        // Merged should have detail from both
        // (This is a simplified check - in reality, we'd check across the entire tonal range)
        let dark_lum = rgb_to_luminance(dark_pixel);
        let bright_lum = rgb_to_luminance(bright_pixel);
        let merged_lum = rgb_to_luminance(merged_pixel);

        // Merged luminance should be between dark and bright
        assert!(
            merged_lum >= dark_lum && merged_lum <= bright_lum,
            "Merged luminance should be between dark and bright"
        );
    }

    #[test]
    fn test_hdr_merge_zero_exposures() {
        let exposures: Vec<(RgbImage16, f32)> = vec![];
        let settings = HdrMergeSettings::default();

        let result = merge_hdr(&exposures, &settings);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HdrError::NoExposures));
    }

    #[test]
    fn test_hdr_merge_dimension_mismatch() {
        let img1 = create_test_image(10, 10, 128);
        let img2 = create_test_image(20, 20, 128);

        let exposures = vec![(img1, 0.0), (img2, 1.0)];
        let settings = HdrMergeSettings::default();

        let result = merge_hdr(&exposures, &settings);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HdrError::DimensionMismatch));
    }

    #[test]
    fn test_hdr_merge_invalid_base_index() {
        let img = create_test_image(10, 10, 128);
        let exposures = vec![(img.clone(), 0.0), (img, 1.0)];
        let settings = HdrMergeSettings {
            base_exposure_index: 5,
            ..Default::default()
        };

        let result = merge_hdr(&exposures, &settings);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HdrError::InvalidBaseIndex(5)));
    }

    #[test]
    fn test_hdr_merge_invalid_deghosting() {
        let img = create_test_image(10, 10, 128);
        let exposures = vec![(img, 0.0)];
        let settings = HdrMergeSettings {
            deghosting: 1.5,
            ..Default::default()
        };

        let result = merge_hdr(&exposures, &settings);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HdrError::InvalidDeghosting));
    }

    #[test]
    fn test_weight_map_computation() {
        let img = create_gradient_image(10, 10);
        let weights = compute_weight_map(&img);

        assert_eq!(weights.len(), 100); // 10x10 pixels

        // All weights should be positive
        for w in weights.iter() {
            assert!(*w >= 0.0, "Weight should be non-negative");
        }
    }

    #[test]
    fn test_weight_normalization() {
        let weight1 = vec![1.0; 100];
        let weight2 = vec![2.0; 100];
        let weight_maps = vec![weight1, weight2];

        let normalized = normalize_weights(&weight_maps, 10, 10);

        // Check each pixel's weights sum to 1.0
        for i in 0..100 {
            let sum = normalized[0][i] + normalized[1][i];
            assert!((sum - 1.0).abs() < 0.001, "Weights should sum to 1.0");
        }

        // Weight ratio should be preserved (1:2)
        assert!((normalized[0][0] - 1.0 / 3.0).abs() < 0.001);
        assert!((normalized[1][0] - 2.0 / 3.0).abs() < 0.001);
    }

    // Helper: create uniform test image
    fn create_test_image(width: u32, height: u32, value: u8) -> RgbImage16 {
        let value_u16 = (value as u16) * 257;
        let data = vec![value_u16; (width * height * 3) as usize];
        RgbImage16::from_data(width, height, data)
    }

    // Helper: create gradient image (dark to bright)
    fn create_gradient_image(width: u32, height: u32) -> RgbImage16 {
        let mut data = vec![0u16; (width * height * 3) as usize];

        for y in 0..height {
            for x in 0..width {
                let value = ((x as f32 / width as f32) * 65535.0) as u16;
                let idx = ((y * width + x) * 3) as usize;
                data[idx] = value;
                data[idx + 1] = value;
                data[idx + 2] = value;
            }
        }

        RgbImage16::from_data(width, height, data)
    }
}
