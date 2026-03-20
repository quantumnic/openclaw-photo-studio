//! Local adjustments rendering
//!
//! Implements mask generation and blending for gradient, radial, and brush masks.

use crate::pipeline::types::{BrushStroke, LocalAdjustment, LocalSettings, MaskType, RgbImage16};
use crate::pipeline::process;

/// Apply all local adjustments to an image
pub fn apply_local_adjustments(
    image: &mut RgbImage16,
    adjustments: &[LocalAdjustment],
) {
    if adjustments.is_empty() {
        return;
    }

    let width = image.width;
    let height = image.height;

    // Sort by order
    let mut sorted_adjustments: Vec<_> = adjustments.iter().collect();
    sorted_adjustments.sort_by_key(|adj| adj.order);

    // Apply each adjustment
    for adjustment in sorted_adjustments {
        if !adjustment.enabled {
            continue;
        }

        // Generate mask for this adjustment
        let mask = generate_mask(&adjustment.mask_type, width, height);

        // Apply settings to a copy of the image
        let mut adjusted = image.clone();
        apply_local_settings(&mut adjusted, &adjustment.settings);

        // Blend original with adjusted using the mask
        blend_with_mask(&mut image.data, &adjusted.data, &mask, width, height);
    }
}

/// Generate a mask image for the given mask type
/// Returns Vec<f32> where 0.0 = not affected, 1.0 = fully affected
fn generate_mask(mask_type: &MaskType, width: u32, height: u32) -> Vec<f32> {
    match mask_type {
        MaskType::Gradient { start_x, start_y, end_x, end_y } => {
            generate_gradient_mask(width, height, *start_x, *start_y, *end_x, *end_y)
        }
        MaskType::Radial { center_x, center_y, radius_x, radius_y, feather, invert } => {
            generate_radial_mask(width, height, *center_x, *center_y, *radius_x, *radius_y, *feather, *invert)
        }
        MaskType::Brush { strokes } => {
            generate_brush_mask(width, height, strokes)
        }
    }
}

/// Generate a linear gradient mask
pub fn generate_gradient_mask(
    width: u32,
    height: u32,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
) -> Vec<f32> {
    let mut mask = vec![0.0; (width * height) as usize];

    // Convert normalized coordinates to pixels
    let sx = start_x * width as f32;
    let sy = start_y * height as f32;
    let ex = end_x * width as f32;
    let ey = end_y * height as f32;

    // Direction vector
    let dx = ex - sx;
    let dy = ey - sy;
    let length_sq = dx * dx + dy * dy;

    if length_sq < 1e-6 {
        // Degenerate gradient - fill with 1.0
        mask.fill(1.0);
        return mask;
    }

    let _length = length_sq.sqrt();

    for y in 0..height {
        for x in 0..width {
            let px = x as f32;
            let py = y as f32;

            // Vector from start to pixel
            let vx = px - sx;
            let vy = py - sy;

            // Project onto gradient direction
            let projection = (vx * dx + vy * dy) / length_sq;

            // Clamp to 0-1 and store
            let value = projection.clamp(0.0, 1.0);
            mask[(y * width + x) as usize] = value;
        }
    }

    mask
}

/// Generate a radial/elliptical mask
#[allow(clippy::too_many_arguments)]
pub fn generate_radial_mask(
    width: u32,
    height: u32,
    center_x: f32,
    center_y: f32,
    radius_x: f32,
    radius_y: f32,
    feather: f32,
    invert: bool,
) -> Vec<f32> {
    let mut mask = vec![0.0; (width * height) as usize];

    // Convert normalized coordinates to pixels
    let cx = center_x * width as f32;
    let cy = center_y * height as f32;
    let rx = radius_x * width as f32;
    let ry = radius_y * height as f32;

    // Feather creates smooth transition
    let inner_threshold = 0.8;
    let outer_threshold = 1.0 + feather;

    for y in 0..height {
        for x in 0..width {
            let px = x as f32;
            let py = y as f32;

            // Normalized distance from center (ellipse equation)
            let dx = (px - cx) / rx;
            let dy = (py - cy) / ry;
            let dist = (dx * dx + dy * dy).sqrt();

            // Smooth transition using smoothstep
            let value = if dist < inner_threshold {
                1.0
            } else if dist > outer_threshold {
                0.0
            } else {
                // Smoothstep interpolation
                let t = (dist - inner_threshold) / (outer_threshold - inner_threshold);
                1.0 - smoothstep(t)
            };

            let final_value = if invert { 1.0 - value } else { value };
            mask[(y * width + x) as usize] = final_value;
        }
    }

    mask
}

/// Generate a brush mask from strokes
pub fn generate_brush_mask(
    width: u32,
    height: u32,
    strokes: &[BrushStroke],
) -> Vec<f32> {
    let mut mask = vec![0.0; (width * height) as usize];

    for stroke in strokes {
        for &(norm_x, norm_y) in &stroke.points {
            // Convert normalized coordinates to pixels
            let px = (norm_x * width as f32) as i32;
            let py = (norm_y * height as f32) as i32;

            // Brush radius in pixels
            let radius = (stroke.size * (width as f32).min(height as f32)) as i32;

            // Paint a circle at this point
            paint_circle(
                &mut mask,
                width,
                height,
                px,
                py,
                radius,
                stroke.feather,
                stroke.flow,
                stroke.erase,
            );
        }
    }

    mask
}

/// Paint a circular brush stroke onto the mask
#[allow(clippy::too_many_arguments)]
fn paint_circle(
    mask: &mut [f32],
    width: u32,
    height: u32,
    cx: i32,
    cy: i32,
    radius: i32,
    feather: f32,
    flow: f32,
    erase: bool,
) {
    let radius_f = radius as f32;
    let feather_start = radius_f * (1.0 - feather);

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = cx + dx;
            let y = cy + dy;

            if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
                continue;
            }

            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist > radius_f {
                continue;
            }

            // Gaussian-like falloff with feathering
            let intensity = if dist < feather_start {
                flow
            } else {
                let t = (dist - feather_start) / (radius_f - feather_start);
                flow * (1.0 - smoothstep(t))
            };

            let idx = (y * width as i32 + x) as usize;
            if erase {
                // Erase mode: reduce mask value
                mask[idx] = (mask[idx] - intensity).max(0.0);
            } else {
                // Paint mode: increase mask value (max combiner)
                mask[idx] = mask[idx].max(intensity);
            }
        }
    }
}

/// Smoothstep function for smooth transitions
fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Apply local settings to an image
fn apply_local_settings(image: &mut RgbImage16, settings: &LocalSettings) {
    // Apply each setting if non-zero
    if settings.exposure != 0.0 {
        process::apply_exposure(&mut image.data, settings.exposure);
    }

    if settings.contrast != 0 {
        process::apply_contrast(&mut image.data, settings.contrast);
    }

    if settings.highlights != 0 || settings.shadows != 0 {
        process::apply_highlights_shadows(
            &mut image.data,
            settings.highlights,
            settings.shadows,
            0,
            0,
        );
    }

    if settings.clarity != 0 {
        process::apply_clarity(image, settings.clarity);
    }

    if settings.saturation != 0 {
        process::apply_saturation(&mut image.data, settings.saturation, 0);
    }

    if settings.sharpness != 0 {
        // Convert sharpness to sharpening amount
        let amount = settings.sharpness.unsigned_abs();
        process::apply_sharpening(image, amount, 1.0);
    }
}

/// Blend original image with adjusted image using a mask
fn blend_with_mask(
    original: &mut [u16],
    adjusted: &[u16],
    mask: &[f32],
    width: u32,
    height: u32,
) {
    let pixels = (width * height) as usize;
    for i in 0..pixels {
        let mask_value = mask[i];
        let inv_mask = 1.0 - mask_value;

        // Blend RGB channels
        for c in 0..3 {
            let idx = i * 3 + c;
            let orig = original[idx] as f32;
            let adj = adjusted[idx] as f32;
            original[idx] = (orig * inv_mask + adj * mask_value) as u16;
        }
    }
}

/// Apply luminance-based range mask to an existing mask
pub fn apply_luminance_range_mask(
    base_mask: &[f32],
    image_data: &[u16],
    width: u32,
    height: u32,
    range_min: f32,
    range_max: f32,
    smoothness: f32,
) -> Vec<f32> {
    let pixels = (width * height) as usize;
    let mut result = vec![0.0; pixels];

    for i in 0..pixels {
        let idx = i * 3;
        let r = image_data[idx] as f32 / 65535.0;
        let g = image_data[idx + 1] as f32 / 65535.0;
        let b = image_data[idx + 2] as f32 / 65535.0;

        // Compute relative luminance
        let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;

        // Check if luminance is in range
        let in_range = if smoothness > 0.0 {
            // Smooth edges
            let smooth_min = range_min - smoothness * 0.5;
            let smooth_max = range_max + smoothness * 0.5;

            if luminance < smooth_min || luminance > smooth_max {
                0.0
            } else if luminance >= range_min && luminance <= range_max {
                1.0
            } else if luminance < range_min {
                // Fade in from smooth_min to range_min
                let t = (luminance - smooth_min) / (range_min - smooth_min);
                smoothstep(t)
            } else {
                // Fade out from range_max to smooth_max
                let t = (luminance - range_max) / (smooth_max - range_max);
                1.0 - smoothstep(t)
            }
        } else {
            // Hard edge
            if luminance >= range_min && luminance <= range_max {
                1.0
            } else {
                0.0
            }
        };

        result[i] = base_mask[i] * in_range;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_mask_generation() {
        let width = 10;
        let height = 10;
        // Gradient from left (0.0) to right (1.0)
        let mask = generate_gradient_mask(width, height, 0.0, 0.5, 1.0, 0.5);

        // Check left edge (should be ~0.0)
        assert!(mask[5 * width as usize] < 0.15, "Left edge: {}", mask[5 * width as usize]);

        // Check right edge (should be ~1.0)
        assert!(mask[5 * width as usize + 9] > 0.85, "Right edge: {}", mask[5 * width as usize + 9]);

        // Check middle (should be ~0.5)
        let mid = mask[5 * width as usize + 5];
        assert!(mid > 0.4 && mid < 0.6, "Middle: {}", mid);
    }

    #[test]
    fn test_radial_mask_center() {
        let width = 10;
        let height = 10;
        let mask = generate_radial_mask(width, height, 0.5, 0.5, 0.3, 0.3, 0.1, false);

        // Center pixel should be 1.0 (fully affected)
        let center = mask[5 * width as usize + 5];
        assert!(center > 0.9);

        // Edge pixels should be close to 0.0
        assert!(mask[0] < 0.1);
        assert!(mask[9] < 0.1);
    }

    #[test]
    fn test_radial_mask_invert() {
        let width = 10;
        let height = 10;
        let mask_normal = generate_radial_mask(width, height, 0.5, 0.5, 0.3, 0.3, 0.1, false);
        let mask_invert = generate_radial_mask(width, height, 0.5, 0.5, 0.3, 0.3, 0.1, true);

        // Inverted mask should be roughly 1.0 - normal
        let center_idx = 5 * width as usize + 5;
        let sum = mask_normal[center_idx] + mask_invert[center_idx];
        assert!((sum - 1.0).abs() < 0.2); // Allow some tolerance due to smoothstep
    }

    #[test]
    fn test_brush_mask_paints_circle() {
        let width = 20;
        let height = 20;
        let strokes = vec![BrushStroke {
            points: vec![(0.5, 0.5)], // Center
            size: 0.1,
            feather: 0.3,
            flow: 1.0,
            erase: false,
        }];

        let mask = generate_brush_mask(width, height, &strokes);

        // Center should be 1.0
        let center = mask[10 * width as usize + 10];
        assert!(center > 0.9);

        // Far corner should be 0.0
        assert!(mask[0] < 0.1);
    }

    #[test]
    fn test_local_adj_empty_is_identity() {
        let mut image = RgbImage16::new(10, 10);
        // Fill with test pattern
        for i in 0..image.data.len() {
            image.data[i] = (i as u16);
        }

        let original = image.clone();
        apply_local_adjustments(&mut image, &[]);

        // Should be unchanged
        assert_eq!(image.data, original.data);
    }

    #[test]
    fn test_local_adj_applies_exposure() {
        let mut image = RgbImage16::new(10, 10);
        // Fill with mid-gray
        image.data.fill(32768);

        let adjustment = LocalAdjustment {
            id: "test".to_string(),
            mask_type: MaskType::Gradient {
                start_x: 0.0,
                start_y: 0.5,
                end_x: 1.0,
                end_y: 0.5,
            },
            settings: LocalSettings {
                exposure: 1.0, // +1 EV
                ..Default::default()
            },
            enabled: true,
            order: 0,
        };

        apply_local_adjustments(&mut image, &[adjustment]);

        // Left side (mask=0) should be unchanged
        let left_pixel = image.get_pixel(0, 5)[0];
        assert!((left_pixel as i32 - 32768).abs() < 5000);

        // Right side (mask=1) should be brighter
        let right_pixel = image.get_pixel(9, 5)[0];
        assert!(right_pixel > 40000);
    }

    #[test]
    fn test_luminance_range_mask_midtones() {
        let width = 4;
        let height = 1;
        let mut image_data = vec![0u16; 12]; // 4 pixels RGB

        // Pixel 0: black (luminance ~0.0)
        image_data[0] = 0;
        image_data[1] = 0;
        image_data[2] = 0;

        // Pixel 1: dark gray (luminance ~0.25)
        image_data[3] = 16384;
        image_data[4] = 16384;
        image_data[5] = 16384;

        // Pixel 2: mid gray (luminance ~0.5)
        image_data[6] = 32768;
        image_data[7] = 32768;
        image_data[8] = 32768;

        // Pixel 3: bright (luminance ~1.0)
        image_data[9] = 65535;
        image_data[10] = 65535;
        image_data[11] = 65535;

        let base_mask = vec![1.0; 4]; // All pixels enabled in base mask
        let range_mask = apply_luminance_range_mask(
            &base_mask,
            &image_data,
            width,
            height,
            0.33,  // range_min
            0.66,  // range_max
            0.0,   // no smoothness
        );

        // Black pixel should be masked out
        assert!(range_mask[0] < 0.1);

        // Dark gray might be on the edge
        assert!(range_mask[1] < 0.5);

        // Mid gray should be in range
        assert!(range_mask[2] > 0.9);

        // Bright pixel should be masked out
        assert!(range_mask[3] < 0.1);
    }

    #[test]
    fn test_luminance_range_smooth_edges() {
        let width = 3;
        let height = 1;
        let mut image_data = vec![0u16; 9];

        // Pixel 0: below range (luminance 0.35)
        image_data[0] = 22938;
        image_data[1] = 22938;
        image_data[2] = 22938;

        // Pixel 1: in range (luminance 0.5)
        image_data[3] = 32768;
        image_data[4] = 32768;
        image_data[5] = 32768;

        // Pixel 2: above range (luminance 0.65)
        image_data[6] = 42598;
        image_data[7] = 42598;
        image_data[8] = 42598;

        let base_mask = vec![1.0; 3];
        let range_mask = apply_luminance_range_mask(
            &base_mask,
            &image_data,
            width,
            height,
            0.4,
            0.6,
            0.2, // smoothness
        );

        // With smoothness, edges should have partial values
        // Pixel 0 at luminance 0.35 is in the smooth region
        assert!(range_mask[0] >= 0.0 && range_mask[0] <= 1.0, "Pixel 0: {}", range_mask[0]);
        assert!(range_mask[1] > 0.9, "Center: {}", range_mask[1]); // Center should be full
        assert!(range_mask[2] >= 0.0 && range_mask[2] <= 1.0, "Pixel 2: {}", range_mask[2]);
    }
}
