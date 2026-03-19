//! Core image processing functions (CPU implementation)

use super::color::{calculate_wb_multipliers, hsv_to_rgb, rgb_to_hsv};
use super::types::{CropSettings, RgbImage16};

/// Apply exposure adjustment (multiply by 2^ev)
pub fn apply_exposure(data: &mut [u16], ev: f32) {
    if ev == 0.0 {
        return;
    }

    let multiplier = 2.0_f32.powf(ev);

    for pixel in data.iter_mut() {
        let value = (*pixel as f32) * multiplier;
        *pixel = value.min(65535.0) as u16;
    }
}

/// Apply white balance (multiply R/G/B channels by respective multipliers)
pub fn apply_white_balance(data: &mut [u16], temp: u32, tint: i32) {
    if temp == 5500 && tint == 0 {
        return; // Neutral, no change
    }

    let [r_mult, g_mult, b_mult] = calculate_wb_multipliers(temp, tint);

    for chunk in data.chunks_exact_mut(3) {
        chunk[0] = ((chunk[0] as f32) * r_mult).min(65535.0) as u16;
        chunk[1] = ((chunk[1] as f32) * g_mult).min(65535.0) as u16;
        chunk[2] = ((chunk[2] as f32) * b_mult).min(65535.0) as u16;
    }
}

/// Apply contrast adjustment using S-curve
/// amount: -100 to +100
pub fn apply_contrast(data: &mut [u16], amount: i32) {
    if amount == 0 {
        return;
    }

    let factor = 1.0 + (amount as f32) / 100.0;
    let midpoint = 32768.0; // Middle of 16-bit range

    for pixel in data.iter_mut() {
        let value = *pixel as f32;
        // Apply contrast around midpoint
        let adjusted = midpoint + (value - midpoint) * factor;
        *pixel = adjusted.clamp(0.0, 65535.0) as u16;
    }
}

/// Apply highlights/shadows/whites/blacks adjustments
/// These are tone range adjustments that target specific luminance ranges
pub fn apply_highlights_shadows(
    data: &mut [u16],
    highlights: i32,
    shadows: i32,
    whites: i32,
    blacks: i32,
) {
    if highlights == 0 && shadows == 0 && whites == 0 && blacks == 0 {
        return;
    }

    // Convert parameters to multipliers
    let highlights_mult = 1.0 + (highlights as f32) / 100.0;
    let shadows_mult = 1.0 + (shadows as f32) / 100.0;
    let whites_mult = 1.0 + (whites as f32) / 100.0;
    let blacks_mult = 1.0 + (blacks as f32) / 100.0;

    for chunk in data.chunks_exact_mut(3) {
        // Calculate luminance (simple average)
        let luma = ((chunk[0] as f32 + chunk[1] as f32 + chunk[2] as f32) / 3.0) / 65535.0;

        // Calculate blend weights for each range
        // Shadows: affects darker tones (0.0-0.3)
        let shadow_weight = if luma < 0.3 {
            (0.3 - luma) / 0.3
        } else {
            0.0
        };

        // Highlights: affects brighter tones (0.7-1.0)
        let highlight_weight = if luma > 0.7 {
            (luma - 0.7) / 0.3
        } else {
            0.0
        };

        // Blacks: affects very dark tones (0.0-0.15)
        let black_weight = if luma < 0.15 {
            (0.15 - luma) / 0.15
        } else {
            0.0
        };

        // Whites: affects very bright tones (0.85-1.0)
        let white_weight = if luma > 0.85 {
            (luma - 0.85) / 0.15
        } else {
            0.0
        };

        // Apply adjustments per channel
        for i in 0..3 {
            let mut value = chunk[i] as f32;

            // Apply shadows
            if shadow_weight > 0.0 {
                value *= 1.0 + (shadows_mult - 1.0) * shadow_weight;
            }

            // Apply highlights
            if highlight_weight > 0.0 {
                value *= 1.0 + (highlights_mult - 1.0) * highlight_weight;
            }

            // Apply blacks
            if black_weight > 0.0 {
                value *= 1.0 + (blacks_mult - 1.0) * black_weight;
            }

            // Apply whites
            if white_weight > 0.0 {
                value *= 1.0 + (whites_mult - 1.0) * white_weight;
            }

            chunk[i] = value.clamp(0.0, 65535.0) as u16;
        }
    }
}

/// Apply saturation and vibrance adjustments
/// saturation: -100 to +100 (affects all colors equally)
/// vibrance: -100 to +100 (protects skin tones, affects muted colors more)
pub fn apply_saturation(data: &mut [u16], saturation: i32, vibrance: i32) {
    if saturation == 0 && vibrance == 0 {
        return;
    }

    let sat_mult = 1.0 + (saturation as f32) / 100.0;
    let vib_mult = vibrance as f32 / 100.0;

    for chunk in data.chunks_exact_mut(3) {
        // Normalize to 0.0-1.0
        let r = (chunk[0] as f32) / 65535.0;
        let g = (chunk[1] as f32) / 65535.0;
        let b = (chunk[2] as f32) / 65535.0;

        // Convert to HSV
        let (h, s, v) = rgb_to_hsv(r, g, b);

        // Apply saturation
        let mut new_s = s * sat_mult;

        // Apply vibrance (more effect on less saturated colors)
        if vibrance != 0 {
            let sat_boost = vib_mult * (1.0 - s); // More effect on muted colors
            new_s += sat_boost;
        }

        new_s = new_s.clamp(0.0, 1.0);

        // Convert back to RGB
        let (r2, g2, b2) = hsv_to_rgb(h, new_s, v);

        chunk[0] = (r2 * 65535.0).round() as u16;
        chunk[1] = (g2 * 65535.0).round() as u16;
        chunk[2] = (b2 * 65535.0).round() as u16;
    }
}

/// Apply clarity (local contrast enhancement using simplified unsharp mask)
/// amount: -100 to +100
pub fn apply_clarity(image: &mut RgbImage16, amount: i32) {
    if amount == 0 {
        return;
    }

    let strength = (amount as f32) / 100.0;

    // Create a simplified blur (box blur for speed)
    let radius = 5; // Fixed radius for clarity
    let width = image.width as usize;
    let height = image.height as usize;

    // Process luminance only for clarity
    for y in radius..(height - radius) {
        for x in radius..(width - radius) {
            let idx = (y * width + x) * 3;

            // Calculate local average (simple box blur)
            let mut sum = [0.0_f32; 3];
            let mut count = 0;

            for dy in -(radius as isize)..=(radius as isize) {
                for dx in -(radius as isize)..=(radius as isize) {
                    let ny = (y as isize + dy) as usize;
                    let nx = (x as isize + dx) as usize;
                    let nidx = (ny * width + nx) * 3;

                    sum[0] += image.data[nidx] as f32;
                    sum[1] += image.data[nidx + 1] as f32;
                    sum[2] += image.data[nidx + 2] as f32;
                    count += 1;
                }
            }

            let avg = [
                sum[0] / count as f32,
                sum[1] / count as f32,
                sum[2] / count as f32,
            ];

            // Apply unsharp mask: original + strength * (original - blurred)
            for i in 0..3 {
                let original = image.data[idx + i] as f32;
                let difference = original - avg[i];
                let enhanced = original + strength * difference;
                image.data[idx + i] = enhanced.clamp(0.0, 65535.0) as u16;
            }
        }
    }
}

/// Apply sharpening using unsharp mask
/// amount: 0-150, radius: 0.5-3.0
pub fn apply_sharpening(image: &mut RgbImage16, amount: u32, radius: f32) {
    if amount == 0 {
        return;
    }

    let strength = (amount as f32) / 100.0;
    let blur_radius = (radius * 2.0).round() as usize;

    if blur_radius == 0 {
        return;
    }

    let width = image.width as usize;
    let height = image.height as usize;

    // Simple box blur for speed (proper would be Gaussian)
    for y in blur_radius..(height - blur_radius) {
        for x in blur_radius..(width - blur_radius) {
            let idx = (y * width + x) * 3;

            let mut sum = [0.0_f32; 3];
            let mut count = 0;

            for dy in -(blur_radius as isize)..=(blur_radius as isize) {
                for dx in -(blur_radius as isize)..=(blur_radius as isize) {
                    let ny = (y as isize + dy) as usize;
                    let nx = (x as isize + dx) as usize;
                    let nidx = (ny * width + nx) * 3;

                    sum[0] += image.data[nidx] as f32;
                    sum[1] += image.data[nidx + 1] as f32;
                    sum[2] += image.data[nidx + 2] as f32;
                    count += 1;
                }
            }

            let avg = [
                sum[0] / count as f32,
                sum[1] / count as f32,
                sum[2] / count as f32,
            ];

            // Unsharp mask
            for i in 0..3 {
                let original = image.data[idx + i] as f32;
                let difference = original - avg[i];
                let sharpened = original + strength * difference;
                image.data[idx + i] = sharpened.clamp(0.0, 65535.0) as u16;
            }
        }
    }
}

/// Apply crop to image
pub fn apply_crop(image: &RgbImage16, crop: &CropSettings) -> RgbImage16 {
    if crop.is_identity() {
        return image.clone();
    }

    let src_width = image.width as f32;
    let src_height = image.height as f32;

    let left_px = (crop.left * src_width) as u32;
    let top_px = (crop.top * src_height) as u32;
    let right_px = (crop.right * src_width) as u32;
    let bottom_px = (crop.bottom * src_height) as u32;

    let new_width = right_px - left_px;
    let new_height = bottom_px - top_px;

    let mut result = RgbImage16::new(new_width, new_height);

    for y in 0..new_height {
        for x in 0..new_width {
            let src_x = left_px + x;
            let src_y = top_px + y;

            if src_x < image.width && src_y < image.height {
                let pixel = image.get_pixel(src_x, src_y);
                result.set_pixel(x, y, pixel);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exposure_zero_is_identity() {
        let mut data = vec![1000, 2000, 3000, 4000];
        let original = data.clone();
        apply_exposure(&mut data, 0.0);
        assert_eq!(data, original);
    }

    #[test]
    fn test_exposure_positive() {
        let mut data = vec![1000_u16, 2000, 3000];
        apply_exposure(&mut data, 1.0); // +1 EV = 2x
        assert_eq!(data[0], 2000);
        assert_eq!(data[1], 4000);
        assert_eq!(data[2], 6000);
    }

    #[test]
    fn test_exposure_negative() {
        let mut data = vec![2000_u16, 4000, 6000];
        apply_exposure(&mut data, -1.0); // -1 EV = 0.5x
        assert_eq!(data[0], 1000);
        assert_eq!(data[1], 2000);
        assert_eq!(data[2], 3000);
    }

    #[test]
    fn test_exposure_clamps_at_white() {
        let mut data = vec![60000_u16];
        apply_exposure(&mut data, 1.0);
        assert_eq!(data[0], 65535); // Should clamp at max
    }

    #[test]
    fn test_white_balance_neutral_is_identity() {
        let mut data = vec![1000, 2000, 3000, 4000, 5000, 6000];
        let original = data.clone();
        apply_white_balance(&mut data, 5500, 0);

        // Should be approximately the same (within rounding)
        for (a, b) in data.iter().zip(original.iter()) {
            assert!((*a as i32 - *b as i32).abs() < 100);
        }
    }

    #[test]
    fn test_white_balance_warm_increases_red() {
        let mut data = vec![10000, 10000, 10000]; // R, G, B
        let original_r = data[0];

        apply_white_balance(&mut data, 8000, 0); // Warm temperature

        // Red channel should increase or stay same
        assert!(data[0] >= original_r);
    }

    #[test]
    fn test_contrast_zero_is_identity() {
        let mut data = vec![1000, 20000, 40000, 60000];
        let original = data.clone();
        apply_contrast(&mut data, 0);
        assert_eq!(data, original);
    }

    #[test]
    fn test_contrast_positive_increases_range() {
        let mut data = vec![20000, 32768, 45000]; // Dark, mid, bright
        let mid_before = data[1];

        apply_contrast(&mut data, 50);

        // Dark should get darker
        assert!(data[0] < 20000);
        // Mid should stay roughly the same
        assert!((data[1] as i32 - mid_before as i32).abs() < 1000);
        // Bright should get brighter
        assert!(data[2] > 45000);
    }

    #[test]
    fn test_highlights_shadows_neutral_is_identity() {
        let mut data = vec![1000, 20000, 40000, 60000, 10000, 50000];
        let original = data.clone();
        apply_highlights_shadows(&mut data, 0, 0, 0, 0);
        assert_eq!(data, original);
    }

    #[test]
    fn test_saturation_zero_is_identity() {
        let mut data = vec![30000, 20000, 10000]; // Some color
        let original = data.clone();
        apply_saturation(&mut data, 0, 0);

        // Should be very close (may have small rounding errors from HSV conversion)
        for (a, b) in data.iter().zip(original.iter()) {
            assert!((*a as i32 - *b as i32).abs() < 100);
        }
    }

    #[test]
    fn test_saturation_increase() {
        let mut data = vec![40000, 30000, 20000]; // RGB with some color
        let original = data.clone();

        apply_saturation(&mut data, 50, 0);

        // Color differences should increase
        let orig_range = original[0] as i32 - original[2] as i32;
        let new_range = data[0] as i32 - data[2] as i32;
        assert!(new_range > orig_range);
    }

    #[test]
    fn test_saturation_desaturate() {
        let mut data = vec![50000, 30000, 10000]; // Colorful
        apply_saturation(&mut data, -100, 0); // Full desaturation

        // Should converge toward gray (all channels similar)
        let range = data[0] as i32 - data[2] as i32;
        assert!(range.abs() < 5000);
    }

    #[test]
    fn test_clarity_zero_is_identity() {
        let mut img = RgbImage16::new(10, 10);
        for i in 0..img.data.len() {
            img.data[i] = (i * 100) as u16;
        }
        let original = img.clone();

        apply_clarity(&mut img, 0);

        assert_eq!(img.data, original.data);
    }

    #[test]
    fn test_sharpening_zero_is_identity() {
        let mut img = RgbImage16::new(10, 10);
        for i in 0..img.data.len() {
            img.data[i] = (i * 100) as u16;
        }
        let original = img.clone();

        apply_sharpening(&mut img, 0, 1.0);

        assert_eq!(img.data, original.data);
    }

    #[test]
    fn test_crop_identity() {
        let img = RgbImage16::new(100, 100);
        let crop = CropSettings::default();
        let result = apply_crop(&img, &crop);

        assert_eq!(result.width, 100);
        assert_eq!(result.height, 100);
    }

    #[test]
    fn test_crop_half_image() {
        let mut img = RgbImage16::new(100, 100);
        // Fill with pattern
        for y in 0..100 {
            for x in 0..100 {
                img.set_pixel(x, y, [x as u16, y as u16, 0]);
            }
        }

        let crop = CropSettings {
            left: 0.0,
            top: 0.0,
            right: 0.5,
            bottom: 0.5,
            angle: 0.0,
        };

        let result = apply_crop(&img, &crop);

        assert_eq!(result.width, 50);
        assert_eq!(result.height, 50);

        // Check a pixel
        let pixel = result.get_pixel(10, 10);
        assert_eq!(pixel[0], 10); // x value
        assert_eq!(pixel[1], 10); // y value
    }

    #[test]
    fn test_crop_offset() {
        let mut img = RgbImage16::new(100, 100);
        for y in 0..100 {
            for x in 0..100 {
                img.set_pixel(x, y, [x as u16, y as u16, 0]);
            }
        }

        let crop = CropSettings {
            left: 0.5,
            top: 0.5,
            right: 1.0,
            bottom: 1.0,
            angle: 0.0,
        };

        let result = apply_crop(&img, &crop);

        assert_eq!(result.width, 50);
        assert_eq!(result.height, 50);

        // Check that we got the right region
        let pixel = result.get_pixel(0, 0);
        assert_eq!(pixel[0], 50); // Should be from x=50 in original
        assert_eq!(pixel[1], 50); // Should be from y=50 in original
    }
}
