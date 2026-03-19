//! Core image processing functions (CPU implementation)

use super::color::{calculate_wb_multipliers, hsv_to_rgb, rgb_to_hsv, rgb_to_hsl, hsl_to_rgb};
use super::types::{ColorGrading, CropSettings, HealingSpot, HslAdjustments, NoiseReductionSettings, RgbImage16, SpotType, ToneCurve};

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
        for value_ref in chunk.iter_mut().take(3) {
            let mut value = *value_ref as f32;

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

            *value_ref = value.clamp(0.0, 65535.0) as u16;
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
            for (i, &avg_val) in avg.iter().enumerate() {
                let original = image.data[idx + i] as f32;
                let difference = original - avg_val;
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
            for (i, &avg_val) in avg.iter().enumerate() {
                let original = image.data[idx + i] as f32;
                let difference = original - avg_val;
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

/// Apply tone curve using lookup table (LUT)
/// Builds a 65536-entry LUT via linear interpolation between curve points
pub fn apply_tone_curve(data: &mut [u16], curve: &ToneCurve) {
    if curve.points.len() < 2 {
        return;
    }

    // Build LUT
    let mut lut = [0u16; 65536];

    for input in 0..=65535 {
        let input_norm = (input as f32) / 65535.0;

        // Find the two curve points to interpolate between
        let mut found = false;
        for i in 0..curve.points.len() - 1 {
            let p0 = &curve.points[i];
            let p1 = &curve.points[i + 1];

            if input_norm >= p0.x && input_norm <= p1.x {
                // Linear interpolation
                let t = if p1.x > p0.x {
                    (input_norm - p0.x) / (p1.x - p0.x)
                } else {
                    0.0
                };

                let output_norm = p0.y + t * (p1.y - p0.y);
                lut[input as usize] = (output_norm.clamp(0.0, 1.0) * 65535.0) as u16;
                found = true;
                break;
            }
        }

        if !found {
            // Extrapolate if outside curve range
            if input_norm < curve.points[0].x {
                lut[input as usize] = (curve.points[0].y.clamp(0.0, 1.0) * 65535.0) as u16;
            } else {
                let last = curve.points.len() - 1;
                lut[input as usize] = (curve.points[last].y.clamp(0.0, 1.0) * 65535.0) as u16;
            }
        }
    }

    // Apply LUT to each pixel value
    for pixel in data.iter_mut() {
        *pixel = lut[*pixel as usize];
    }
}

/// Apply HSL adjustments per color channel
/// 8 channels by hue angle: Red, Orange, Yellow, Green, Aqua, Blue, Purple, Magenta
pub fn apply_hsl(data: &mut [u16], _width: u32, _height: u32, hsl: &HslAdjustments) {
    // Check if any adjustments are non-zero
    let has_adjustments = hsl.hue.iter().any(|&h| h != 0)
        || hsl.saturation.iter().any(|&s| s != 0)
        || hsl.luminance.iter().any(|&l| l != 0);

    if !has_adjustments {
        return;
    }

    for chunk in data.chunks_exact_mut(3) {
        // Normalize to 0.0-1.0
        let r = (chunk[0] as f32) / 65535.0;
        let g = (chunk[1] as f32) / 65535.0;
        let b = (chunk[2] as f32) / 65535.0;

        // Convert to HSL
        let (h, s, l) = rgb_to_hsl(r, g, b);

        // Determine channel based on hue angle (in degrees)
        // Red: 315-15, Orange: 15-45, Yellow: 45-75, Green: 75-165,
        // Aqua: 165-195, Blue: 195-255, Purple: 255-285, Magenta: 285-315
        let hue_deg = h * 360.0;
        let channel = if !(15.0..315.0).contains(&hue_deg) {
            0 // Red
        } else if hue_deg < 45.0 {
            1 // Orange
        } else if hue_deg < 75.0 {
            2 // Yellow
        } else if hue_deg < 165.0 {
            3 // Green
        } else if hue_deg < 195.0 {
            4 // Aqua
        } else if hue_deg < 255.0 {
            5 // Blue
        } else if hue_deg < 285.0 {
            6 // Purple
        } else {
            7 // Magenta
        };

        // Apply adjustments
        let mut new_h = h * 360.0 + (hsl.hue[channel] as f32);
        new_h = new_h.rem_euclid(360.0); // Wrap around

        let sat_adjust = 1.0 + (hsl.saturation[channel] as f32) / 100.0;
        let mut new_s = s * sat_adjust;
        new_s = new_s.clamp(0.0, 1.0);

        let lum_adjust = (hsl.luminance[channel] as f32) / 100.0;
        let mut new_l = l + lum_adjust;
        new_l = new_l.clamp(0.0, 1.0);

        // Convert back to RGB
        let (r2, g2, b2) = hsl_to_rgb(new_h / 360.0, new_s, new_l);

        chunk[0] = (r2 * 65535.0).round() as u16;
        chunk[1] = (g2 * 65535.0).round() as u16;
        chunk[2] = (b2 * 65535.0).round() as u16;
    }
}

/// Apply color grading (3-way color wheels: shadows, midtones, highlights)
pub fn apply_color_grading(data: &mut [u16], _width: u32, _height: u32, cg: &ColorGrading) {
    // Check if any adjustments are non-zero
    if cg.shadows_sat == 0 && cg.midtones_sat == 0 && cg.highlights_sat == 0 {
        return;
    }

    for chunk in data.chunks_exact_mut(3) {
        // Normalize to 0.0-1.0
        let r = (chunk[0] as f32) / 65535.0;
        let g = (chunk[1] as f32) / 65535.0;
        let b = (chunk[2] as f32) / 65535.0;

        // Calculate luminance
        let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;

        // Determine zone and blend weights
        let (shadow_weight, midtone_weight, highlight_weight) = if lum < 0.33 {
            // Shadows zone
            let t = lum / 0.33;
            (1.0 - t, t, 0.0)
        } else if lum < 0.66 {
            // Midtones zone
            let t = (lum - 0.33) / 0.33;
            (0.0, 1.0 - t, t)
        } else {
            // Highlights zone
            let _t = (lum - 0.66) / 0.34;
            (0.0, 0.0, 1.0)
        };

        let mut r2 = r;
        let mut g2 = g;
        let mut b2 = b;

        // Apply shadows tint
        if shadow_weight > 0.0 && cg.shadows_sat > 0 {
            let hue = (cg.shadows_hue as f32) / 360.0;
            let sat = (cg.shadows_sat as f32) / 100.0;
            let (tr, tg, tb) = hsl_to_rgb(hue, sat, 0.5);

            r2 += (tr - 0.5) * shadow_weight * sat;
            g2 += (tg - 0.5) * shadow_weight * sat;
            b2 += (tb - 0.5) * shadow_weight * sat;
        }

        // Apply midtones tint
        if midtone_weight > 0.0 && cg.midtones_sat > 0 {
            let hue = (cg.midtones_hue as f32) / 360.0;
            let sat = (cg.midtones_sat as f32) / 100.0;
            let (tr, tg, tb) = hsl_to_rgb(hue, sat, 0.5);

            r2 += (tr - 0.5) * midtone_weight * sat;
            g2 += (tg - 0.5) * midtone_weight * sat;
            b2 += (tb - 0.5) * midtone_weight * sat;
        }

        // Apply highlights tint
        if highlight_weight > 0.0 && cg.highlights_sat > 0 {
            let hue = (cg.highlights_hue as f32) / 360.0;
            let sat = (cg.highlights_sat as f32) / 100.0;
            let (tr, tg, tb) = hsl_to_rgb(hue, sat, 0.5);

            r2 += (tr - 0.5) * highlight_weight * sat;
            g2 += (tg - 0.5) * highlight_weight * sat;
            b2 += (tb - 0.5) * highlight_weight * sat;
        }

        chunk[0] = (r2.clamp(0.0, 1.0) * 65535.0) as u16;
        chunk[1] = (g2.clamp(0.0, 1.0) * 65535.0) as u16;
        chunk[2] = (b2.clamp(0.0, 1.0) * 65535.0) as u16;
    }
}

/// Apply noise reduction using simplified Non-Local Means (NLM) for luminance + Gaussian for chroma
/// Luminance NR: Fast NLM - search window 7x7, patch 3x3, operates at reduced resolution for performance
/// Color NR: Box blur approximation on Cb/Cr channels
pub fn apply_noise_reduction(
    data: &mut [u16],
    width: u32,
    height: u32,
    settings: &NoiseReductionSettings,
) {
    if settings.luminance == 0 && settings.color == 0 {
        return; // Identity
    }

    let w = width as usize;
    let h = height as usize;

    // Apply luminance NR if needed
    if settings.luminance > 0 {
        apply_luminance_nr(data, w, h, settings.luminance);
    }

    // Apply color NR if needed
    if settings.color > 0 {
        apply_color_nr(data, w, h, settings.color);
    }
}

/// Apply luminance noise reduction using simplified NLM
fn apply_luminance_nr(data: &mut [u16], width: usize, height: usize, amount: u32) {
    // Work at reduced resolution for performance (1/2 scale)
    let scale = 2;
    let small_w = width / scale;
    let small_h = height / scale;

    // Convert to YCbCr - extract Y channel at reduced resolution
    let mut y_channel: Vec<f32> = Vec::with_capacity(small_w * small_h);
    for y in (0..height).step_by(scale) {
        for x in (0..width).step_by(scale) {
            let idx = (y * width + x) * 3;
            let r = data[idx] as f32 / 65535.0;
            let g = data[idx + 1] as f32 / 65535.0;
            let b = data[idx + 2] as f32 / 65535.0;

            // Rec. 601 luma
            let luma = 0.299 * r + 0.587 * g + 0.114 * b;
            y_channel.push(luma);
        }
    }

    // Apply simplified NLM on Y channel
    let h_param = (amount as f32) / 100.0 * 0.1; // h parameter controls filtering strength
    let h_squared = h_param * h_param;

    let mut denoised = vec![0.0_f32; small_w * small_h];

    let search_radius = 3; // 7x7 search window
    let patch_radius = 1;  // 3x3 patch

    for y in search_radius..(small_h - search_radius) {
        for x in search_radius..(small_w - search_radius) {
            let idx = y * small_w + x;
            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;

            // Search in 7x7 neighborhood
            for dy in -(search_radius as isize)..=(search_radius as isize) {
                for dx in -(search_radius as isize)..=(search_radius as isize) {
                    let nx = (x as isize + dx) as usize;
                    let ny = (y as isize + dy) as usize;
                    let nidx = ny * small_w + nx;

                    // Calculate patch distance (3x3)
                    let mut dist_sq = 0.0;
                    let mut patch_count = 0;

                    for py in -(patch_radius as isize)..=(patch_radius as isize) {
                        for px in -(patch_radius as isize)..=(patch_radius as isize) {
                            let p1y = y as isize + py;
                            let p1x = x as isize + px;
                            let p2y = ny as isize + py;
                            let p2x = nx as isize + px;

                            if p1y >= 0 && p1y < small_h as isize && p1x >= 0 && p1x < small_w as isize
                                && p2y >= 0 && p2y < small_h as isize && p2x >= 0 && p2x < small_w as isize {
                                let i1 = (p1y as usize) * small_w + (p1x as usize);
                                let i2 = (p2y as usize) * small_w + (p2x as usize);
                                let diff = y_channel[i1] - y_channel[i2];
                                dist_sq += diff * diff;
                                patch_count += 1;
                            }
                        }
                    }

                    if patch_count > 0 {
                        dist_sq /= patch_count as f32;
                    }

                    // Weight function: exp(-dist²/h²)
                    let weight = (-dist_sq / h_squared.max(0.0001)).exp();
                    weighted_sum += weight * y_channel[nidx];
                    weight_sum += weight;
                }
            }

            denoised[idx] = if weight_sum > 0.0 {
                weighted_sum / weight_sum
            } else {
                y_channel[idx]
            };
        }
    }

    // Copy boundary pixels
    for y in 0..small_h {
        for x in 0..small_w {
            if x < search_radius || x >= small_w - search_radius
               || y < search_radius || y >= small_h - search_radius {
                denoised[y * small_w + x] = y_channel[y * small_w + x];
            }
        }
    }

    // Apply denoised luma back to full-res image (bilinear upscale and blend)
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 3;

            // Sample from denoised
            let fx = (x as f32) / (scale as f32);
            let fy = (y as f32) / (scale as f32);
            let sx = fx.floor() as usize;
            let sy = fy.floor() as usize;

            if sx < small_w && sy < small_h {
                let orig_luma = {
                    let r = data[idx] as f32 / 65535.0;
                    let g = data[idx + 1] as f32 / 65535.0;
                    let b = data[idx + 2] as f32 / 65535.0;
                    0.299 * r + 0.587 * g + 0.114 * b
                };

                let denoised_luma = denoised[sy * small_w + sx];

                // Blend based on amount (not full strength to preserve detail)
                let blend = (amount as f32) / 100.0;
                let final_luma = orig_luma * (1.0 - blend) + denoised_luma * blend;

                // Apply luminance change to all channels proportionally
                let ratio = if orig_luma > 0.001 {
                    final_luma / orig_luma
                } else {
                    1.0
                };

                data[idx] = ((data[idx] as f32) * ratio).clamp(0.0, 65535.0) as u16;
                data[idx + 1] = ((data[idx + 1] as f32) * ratio).clamp(0.0, 65535.0) as u16;
                data[idx + 2] = ((data[idx + 2] as f32) * ratio).clamp(0.0, 65535.0) as u16;
            }
        }
    }
}

/// Apply color noise reduction using box blur on chroma channels
fn apply_color_nr(data: &mut [u16], width: usize, height: usize, color_amount: u32) {
    // Convert RGB to YCbCr, blur Cb/Cr, convert back
    let mut cb_channel = vec![0.0_f32; width * height];
    let mut cr_channel = vec![0.0_f32; width * height];

    // Extract Cb/Cr
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 3;
            let r = data[idx] as f32 / 65535.0;
            let g = data[idx + 1] as f32 / 65535.0;
            let b = data[idx + 2] as f32 / 65535.0;

            // Rec. 601 YCbCr
            let cb = -0.168736 * r - 0.331264 * g + 0.5 * b;
            let cr = 0.5 * r - 0.418688 * g - 0.081312 * b;

            let pidx = y * width + x;
            cb_channel[pidx] = cb;
            cr_channel[pidx] = cr;
        }
    }

    // Apply box blur (3 passes ≈ Gaussian)
    // Sigma maps from color_amount: 0->0, 100->3.0
    let sigma = (color_amount as f32) / 100.0 * 3.0;
    let radius = (sigma * 1.5).ceil() as usize;

    if radius > 0 {
        // 3 passes of box blur
        for _ in 0..3 {
            cb_channel = box_blur(&cb_channel, width, height, radius);
            cr_channel = box_blur(&cr_channel, width, height, radius);
        }
    }

    // Convert back to RGB
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 3;
            let pidx = y * width + x;

            let r = data[idx] as f32 / 65535.0;
            let g = data[idx + 1] as f32 / 65535.0;
            let b = data[idx + 2] as f32 / 65535.0;

            // Original Y
            let y_val = 0.299 * r + 0.587 * g + 0.114 * b;

            // Blurred Cb/Cr
            let cb = cb_channel[pidx];
            let cr = cr_channel[pidx];

            // YCbCr to RGB
            let r2 = y_val + 1.402 * cr;
            let g2 = y_val - 0.344136 * cb - 0.714136 * cr;
            let b2 = y_val + 1.772 * cb;

            data[idx] = (r2.clamp(0.0, 1.0) * 65535.0) as u16;
            data[idx + 1] = (g2.clamp(0.0, 1.0) * 65535.0) as u16;
            data[idx + 2] = (b2.clamp(0.0, 1.0) * 65535.0) as u16;
        }
    }
}

/// Simple box blur helper
fn box_blur(data: &[f32], width: usize, height: usize, radius: usize) -> Vec<f32> {
    let mut result = vec![0.0_f32; width * height];

    for y in 0..height {
        for x in 0..width {
            let mut sum = 0.0;
            let mut count = 0;

            for dy in -(radius as isize)..=(radius as isize) {
                for dx in -(radius as isize)..=(radius as isize) {
                    let nx = (x as isize + dx).clamp(0, width as isize - 1) as usize;
                    let ny = (y as isize + dy).clamp(0, height as isize - 1) as usize;
                    sum += data[ny * width + nx];
                    count += 1;
                }
            }

            result[y * width + x] = sum / count as f32;
        }
    }

    result
}

/// Apply healing/clone spots for blemish removal
pub fn apply_healing_spots(
    data: &mut [u16],
    width: u32,
    height: u32,
    spots: &[HealingSpot],
) {
    if spots.is_empty() {
        return;
    }

    let w = width as usize;

    for spot in spots {
        // Convert normalized coordinates to pixels
        let target_x = (spot.target_x * width as f32) as i32;
        let target_y = (spot.target_y * height as f32) as i32;
        let source_x = (spot.source_x * width as f32) as i32;
        let source_y = (spot.source_y * height as f32) as i32;

        // Radius in pixels (use average of width/height for normalization)
        let avg_dim = ((width + height) / 2) as f32;
        let radius_px = (spot.radius * avg_dim) as i32;

        if radius_px < 1 {
            continue;
        }

        // Process circular region around target
        for dy in -radius_px..=radius_px {
            for dx in -radius_px..=radius_px {
                let tx = target_x + dx;
                let ty = target_y + dy;

                // Check bounds
                if tx < 0 || tx >= width as i32 || ty < 0 || ty >= height as i32 {
                    continue;
                }

                // Calculate distance from center (for circular mask and feathering)
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist > radius_px as f32 {
                    continue; // Outside circle
                }

                // Calculate feather mask (radial gradient)
                let feather_start = radius_px as f32 * (1.0 - spot.feather);
                let mask = if dist < feather_start {
                    1.0 // Full strength inside
                } else {
                    // Fade out in feather region
                    1.0 - ((dist - feather_start) / (radius_px as f32 - feather_start))
                };

                let final_opacity = mask * spot.opacity;

                if final_opacity < 0.001 {
                    continue;
                }

                // Source pixel coordinates
                let sx = source_x + dx;
                let sy = source_y + dy;

                if sx < 0 || sx >= width as i32 || sy < 0 || sy >= height as i32 {
                    continue; // Source out of bounds
                }

                let target_idx = (ty as usize * w + tx as usize) * 3;
                let source_idx = (sy as usize * w + sx as usize) * 3;

                match spot.spot_type {
                    SpotType::Clone => {
                        // Simple clone: copy source to target with opacity blending
                        for c in 0..3 {
                            let target_val = data[target_idx + c] as f32;
                            let source_val = data[source_idx + c] as f32;
                            let blended = target_val * (1.0 - final_opacity) + source_val * final_opacity;
                            data[target_idx + c] = blended.clamp(0.0, 65535.0) as u16;
                        }
                    }
                    SpotType::Heal => {
                        // Heal: copy source texture but blend with target luminance
                        // This creates a more natural blend

                        // Calculate source and target luminance
                        let src_r = data[source_idx] as f32 / 65535.0;
                        let src_g = data[source_idx + 1] as f32 / 65535.0;
                        let src_b = data[source_idx + 2] as f32 / 65535.0;
                        let src_luma = 0.299 * src_r + 0.587 * src_g + 0.114 * src_b;

                        let tgt_r = data[target_idx] as f32 / 65535.0;
                        let tgt_g = data[target_idx + 1] as f32 / 65535.0;
                        let tgt_b = data[target_idx + 2] as f32 / 65535.0;
                        let tgt_luma = 0.299 * tgt_r + 0.587 * tgt_g + 0.114 * tgt_b;

                        // Blend luminance
                        let blended_luma = tgt_luma * (1.0 - final_opacity * 0.5) + src_luma * (final_opacity * 0.5);

                        // Apply luminance to source color (preserve source texture/color)
                        let luma_ratio = if src_luma > 0.001 {
                            blended_luma / src_luma
                        } else {
                            1.0
                        };

                        let healed_r = (src_r * luma_ratio).clamp(0.0, 1.0);
                        let healed_g = (src_g * luma_ratio).clamp(0.0, 1.0);
                        let healed_b = (src_b * luma_ratio).clamp(0.0, 1.0);

                        // Blend with original target
                        let final_r = tgt_r * (1.0 - final_opacity) + healed_r * final_opacity;
                        let final_g = tgt_g * (1.0 - final_opacity) + healed_g * final_opacity;
                        let final_b = tgt_b * (1.0 - final_opacity) + healed_b * final_opacity;

                        data[target_idx] = (final_r * 65535.0) as u16;
                        data[target_idx + 1] = (final_g * 65535.0) as u16;
                        data[target_idx + 2] = (final_b * 65535.0) as u16;
                    }
                }
            }
        }
    }
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

    #[test]
    fn test_tone_curve_identity() {
        let mut data = vec![0, 10000, 32768, 50000, 65535];
        let original = data.clone();
        let curve = ToneCurve::default(); // Linear curve
        apply_tone_curve(&mut data, &curve);

        // Should be unchanged (within rounding tolerance)
        for (a, b) in data.iter().zip(original.iter()) {
            assert!((*a as i32 - *b as i32).abs() < 2);
        }
    }

    #[test]
    fn test_tone_curve_lut_boundaries() {
        let curve = ToneCurve::default();
        let mut data = vec![0u16, 65535];
        apply_tone_curve(&mut data, &curve);

        // LUT boundaries should be correct
        assert_eq!(data[0], 0);
        assert_eq!(data[1], 65535);
    }

    #[test]
    fn test_hsl_zero_is_identity() {
        let mut data = vec![30000, 20000, 10000, 50000, 40000, 30000];
        let original = data.clone();
        let hsl = HslAdjustments::default();
        apply_hsl(&mut data, 2, 1, &hsl);

        // Should be very close (may have small rounding errors from HSL conversion)
        for (a, b) in data.iter().zip(original.iter()) {
            assert!((*a as i32 - *b as i32).abs() < 100);
        }
    }

    #[test]
    fn test_hsl_saturation_minus100() {
        let mut data = vec![50000, 30000, 10000]; // Colorful pixel
        let mut hsl = HslAdjustments::default();

        // Set all channels to -100 saturation
        for i in 0..8 {
            hsl.saturation[i] = -100;
        }

        apply_hsl(&mut data, 1, 1, &hsl);

        // Should converge toward gray (all channels similar)
        let range = data[0] as i32 - data[2] as i32;
        assert!(range.abs() < 5000, "Range too large: {}", range);
    }

    #[test]
    fn test_color_grading_zero_is_identity() {
        let mut data = vec![30000, 20000, 10000, 50000, 40000, 30000];
        let original = data.clone();
        let cg = ColorGrading::default();
        apply_color_grading(&mut data, 2, 1, &cg);

        // Should be unchanged since all saturations are 0
        assert_eq!(data, original);
    }

    #[test]
    fn test_nr_zero_is_identity() {
        let mut data = vec![10000, 20000, 30000, 40000, 50000, 60000];
        let original = data.clone();
        let settings = NoiseReductionSettings::default(); // luminance=0, color=0
        apply_noise_reduction(&mut data, 2, 1, &settings);

        // Should be unchanged
        assert_eq!(data, original);
    }

    #[test]
    fn test_nr_reduces_variance() {
        // Create noisy image
        let mut rng_state = 12345_u32; // Simple LCG for deterministic noise
        let width = 50;
        let height = 50;
        let mut data = Vec::with_capacity(width * height * 3);

        // Fill with noisy gray
        for _ in 0..(width * height) {
            let base = 30000_u16;
            for _ in 0..3 {
                // Simple pseudo-random noise
                rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
                let noise = ((rng_state >> 16) & 0x1FFF) as i32 - 4096; // ±4096
                let value = (base as i32 + noise).clamp(0, 65535) as u16;
                data.push(value);
            }
        }

        // Calculate variance before
        let mean_before: f32 = data.iter().map(|&v| v as f32).sum::<f32>() / data.len() as f32;
        let var_before: f32 = data.iter()
            .map(|&v| {
                let diff = v as f32 - mean_before;
                diff * diff
            })
            .sum::<f32>() / data.len() as f32;

        let mut data_denoised = data.clone();
        let settings = NoiseReductionSettings {
            luminance: 50,
            color: 0,
            ..Default::default()
        };
        apply_noise_reduction(&mut data_denoised, width as u32, height as u32, &settings);

        // Calculate variance after
        let mean_after: f32 = data_denoised.iter().map(|&v| v as f32).sum::<f32>() / data_denoised.len() as f32;
        let var_after: f32 = data_denoised.iter()
            .map(|&v| {
                let diff = v as f32 - mean_after;
                diff * diff
            })
            .sum::<f32>() / data_denoised.len() as f32;

        // Variance should be reduced (noise should be smoother)
        assert!(var_after < var_before, "Variance after ({}) should be less than before ({})", var_after, var_before);
    }

    #[test]
    fn test_color_nr_reduces_chroma_variance() {
        // Create image with chroma noise
        let width = 30;
        let height = 30;
        let mut data = Vec::with_capacity(width * height * 3);

        let mut rng_state = 54321_u32;

        // Gray image with chromatic noise
        for _ in 0..(width * height) {
            let base = 30000_u16;

            // R channel - with noise
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            let r_noise = ((rng_state >> 16) & 0xFFF) as i32 - 2048;
            data.push((base as i32 + r_noise).clamp(0, 65535) as u16);

            // G channel - base
            data.push(base);

            // B channel - with opposite noise
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            let b_noise = ((rng_state >> 16) & 0xFFF) as i32 - 2048;
            data.push((base as i32 + b_noise).clamp(0, 65535) as u16);
        }

        // Calculate chroma variance before (R-G and B-G differences)
        let mut chroma_var_before = 0.0_f32;
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 3;
                let r = data[idx] as i32;
                let g = data[idx + 1] as i32;
                let b = data[idx + 2] as i32;
                let diff_rg = (r - g) as f32;
                let diff_bg = (b - g) as f32;
                chroma_var_before += diff_rg * diff_rg + diff_bg * diff_bg;
            }
        }
        chroma_var_before /= (width * height) as f32;

        let mut data_denoised = data.clone();
        let settings = NoiseReductionSettings {
            luminance: 0,
            color: 50,
            ..Default::default()
        };
        apply_noise_reduction(&mut data_denoised, width as u32, height as u32, &settings);

        // Calculate chroma variance after
        let mut chroma_var_after = 0.0_f32;
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 3;
                let r = data_denoised[idx] as i32;
                let g = data_denoised[idx + 1] as i32;
                let b = data_denoised[idx + 2] as i32;
                let diff_rg = (r - g) as f32;
                let diff_bg = (b - g) as f32;
                chroma_var_after += diff_rg * diff_rg + diff_bg * diff_bg;
            }
        }
        chroma_var_after /= (width * height) as f32;

        // Chroma variance should be reduced
        assert!(chroma_var_after < chroma_var_before,
                "Chroma variance after ({}) should be less than before ({})",
                chroma_var_after, chroma_var_before);
    }

    #[test]
    fn test_healing_spots_empty_is_identity() {
        let mut data = vec![10000, 20000, 30000, 40000, 50000, 60000];
        let original = data.clone();
        let spots: Vec<HealingSpot> = vec![];
        apply_healing_spots(&mut data, 2, 1, &spots);

        // Should be unchanged
        assert_eq!(data, original);
    }

    #[test]
    fn test_clone_copies_pixels() {
        // Create a 20x20 image with a distinctive colored circle at source
        let width = 20;
        let height = 20;
        let mut data = vec![10000_u16; width * height * 3]; // Gray background

        // Create a red circle at (5, 5) with radius 2
        for dy in -2..=2 {
            for dx in -2..=2 {
                if (dx * dx + dy * dy) <= 4 { // radius 2
                    let x = 5 + dx;
                    let y = 5 + dy;
                    if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                        let idx = ((y as usize) * width + (x as usize)) * 3;
                        data[idx] = 50000; // Red
                        data[idx + 1] = 10000; // Low green
                        data[idx + 2] = 10000; // Low blue
                    }
                }
            }
        }

        // Clone from (5, 5) to (15, 15)
        let spot = HealingSpot {
            id: "test".to_string(),
            spot_type: SpotType::Clone,
            target_x: 15.0 / width as f32,
            target_y: 15.0 / height as f32,
            source_x: 5.0 / width as f32,
            source_y: 5.0 / height as f32,
            radius: 2.5 / ((width + height) / 2) as f32,
            feather: 0.0, // No feather for exact copy
            opacity: 1.0, // Full opacity
        };

        apply_healing_spots(&mut data, width as u32, height as u32, &[spot]);

        // Check that target now has similar colors to source
        let target_idx = (15 * width + 15) * 3;
        let source_idx = (5 * width + 5) * 3;

        // Should be similar (may not be exact due to feathering/blending)
        let r_diff = (data[target_idx] as i32 - data[source_idx] as i32).abs();
        let g_diff = (data[target_idx + 1] as i32 - data[source_idx + 1] as i32).abs();
        let b_diff = (data[target_idx + 2] as i32 - data[source_idx + 2] as i32).abs();

        assert!(r_diff < 5000, "R channel should be similar");
        assert!(g_diff < 5000, "G channel should be similar");
        assert!(b_diff < 5000, "B channel should be similar");
    }

    #[test]
    fn test_heal_blends_with_target() {
        // Create image with different luminance regions
        let width = 20;
        let height = 20;
        let mut data = vec![0_u16; width * height * 3];

        // Dark background
        for i in 0..data.len() {
            data[i] = 10000;
        }

        // Bright source region at (5, 5)
        for dy in -2..=2 {
            for dx in -2..=2 {
                if (dx * dx + dy * dy) <= 4 {
                    let x = 5 + dx;
                    let y = 5 + dy;
                    if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                        let idx = ((y as usize) * width + (x as usize)) * 3;
                        data[idx] = 40000;     // Bright red
                        data[idx + 1] = 30000; // Medium green
                        data[idx + 2] = 20000; // Medium blue
                    }
                }
            }
        }

        // Heal from bright (5, 5) to dark (15, 15)
        let spot = HealingSpot {
            id: "test".to_string(),
            spot_type: SpotType::Heal,
            target_x: 15.0 / width as f32,
            target_y: 15.0 / height as f32,
            source_x: 5.0 / width as f32,
            source_y: 5.0 / height as f32,
            radius: 2.0 / ((width + height) / 2) as f32,
            feather: 0.0,
            opacity: 1.0,
        };

        let original_target = data[(15 * width + 15) * 3];

        apply_healing_spots(&mut data, width as u32, height as u32, &[spot]);

        let healed_target = data[(15 * width + 15) * 3];

        // Healed region should be different from original (texture applied)
        assert_ne!(healed_target, original_target);

        // But should not be as bright as source (luminance blended)
        let source_val = data[(5 * width + 5) * 3];
        assert!(healed_target < source_val, "Healed should be darker than bright source");
        assert!(healed_target > original_target, "Healed should be brighter than dark target");
    }
}
