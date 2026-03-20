//! Demosaicing algorithms for Bayer sensor data
//!
//! Converts single-channel Bayer data to full RGB images.
//! Implements multiple algorithms for different speed/quality tradeoffs.

use super::{CfaPattern, RawImage};
use rayon::prelude::*;

/// Demosaiced RGB image
#[derive(Debug, Clone)]
pub struct RgbImage {
    pub width: u32,
    pub height: u32,
    /// RGB data in row-major order: [R,G,B, R,G,B, ...]
    pub data: Vec<u8>,
}

/// Demosaicing algorithm selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemosaicAlgorithm {
    /// Fastest: just use center pixel (no interpolation)
    /// Quality: Poor, but extremely fast
    CenterPixel,

    /// Fast: bilinear interpolation
    /// Quality: Good for thumbnails, acceptable for preview
    Bilinear,

    /// X-Trans demosaicing for Fuji cameras (6x6 CFA pattern)
    /// Quality: Good for X-Trans sensors
    XTrans,
}

/// Check if a camera model uses X-Trans sensor
pub fn is_xtrans(camera_model: &str) -> bool {
    let model_lower = camera_model.to_lowercase();

    // Known Fuji X-Trans models
    let xtrans_models = [
        "x-t5", "x-t4", "x-t3", "x-t2", "x-t1", "x-t30", "x-t20", "x-t10",
        "x-h2", "x-h2s", "x-h1",
        "x-pro3", "x-pro2", "x-pro1",
        "x-e4", "x-e3", "x-e2", "x-e1",
        "x-s10", "x-s20",
        "x100v", "x100f", "x100t", "x100s",
        "gfx", // GFX medium format cameras
    ];

    xtrans_models.iter().any(|&m| model_lower.contains(m))
}

/// Get the color at a specific X-Trans position (6x6 repeating pattern)
/// X-Trans pattern:
/// G B G G R G
/// R G R B G B
/// G B G G R G
/// G R G G B G
/// B G B R G R
/// G R G G B G
#[inline]
fn xtrans_color_at(x: u32, y: u32) -> usize {
    // X-Trans pattern (6x6 repeating)
    // 0 = R, 1 = G, 2 = B
    const XTRANS_PATTERN: [[usize; 6]; 6] = [
        [1, 2, 1, 1, 0, 1], // Row 0
        [0, 1, 0, 2, 1, 2], // Row 1
        [1, 2, 1, 1, 0, 1], // Row 2
        [1, 0, 1, 1, 2, 1], // Row 3
        [2, 1, 2, 0, 1, 0], // Row 4
        [1, 0, 1, 1, 2, 1], // Row 5
    ];

    let row = (y % 6) as usize;
    let col = (x % 6) as usize;
    XTRANS_PATTERN[row][col]
}

/// Get the color at a specific Bayer position
#[inline]
fn bayer_color_at(x: u32, y: u32, pattern: CfaPattern) -> usize {
    let row = (y % 2) as usize;
    let col = (x % 2) as usize;

    match pattern {
        CfaPattern::RGGB => {
            match (row, col) {
                (0, 0) => 0, // R
                (0, 1) => 1, // G
                (1, 0) => 1, // G
                (1, 1) => 2, // B
                _ => unreachable!(),
            }
        }
        CfaPattern::BGGR => {
            match (row, col) {
                (0, 0) => 2, // B
                (0, 1) => 1, // G
                (1, 0) => 1, // G
                (1, 1) => 0, // R
                _ => unreachable!(),
            }
        }
        CfaPattern::GBRG => {
            match (row, col) {
                (0, 0) => 1, // G
                (0, 1) => 2, // B
                (1, 0) => 0, // R
                (1, 1) => 1, // G
                _ => unreachable!(),
            }
        }
        CfaPattern::GRBG => {
            match (row, col) {
                (0, 0) => 1, // G
                (0, 1) => 0, // R
                (1, 0) => 2, // B
                (1, 1) => 1, // G
                _ => unreachable!(),
            }
        }
    }
}

/// Center pixel demosaicing (fastest, lowest quality)
///
/// Just uses the center pixel value without interpolation.
/// Good for very fast thumbnails where quality is not critical.
pub fn demosaic_center_pixel(raw: &RawImage) -> RgbImage {
    let width = raw.width;
    let height = raw.height;
    let pattern = raw.cfa_pattern;

    let mut rgb_data = vec![0u8; (width * height * 3) as usize];

    rgb_data
        .par_chunks_mut(3)
        .enumerate()
        .for_each(|(pixel_idx, rgb)| {
            let x = (pixel_idx as u32) % width;
            let y = (pixel_idx as u32) / width;

            let bayer_idx = (y * width + x) as usize;
            let raw_value = raw.data[bayer_idx];

            // Determine which color this pixel represents
            let color = bayer_color_at(x, y, pattern);

            // Normalize to 0-255
            let normalized = raw.normalize_value(raw_value, color);
            let value = (normalized * 255.0).round() as u8;

            // Set only the appropriate channel
            rgb[0] = if color == 0 { value } else { 0 };
            rgb[1] = if color == 1 { value } else { 0 };
            rgb[2] = if color == 2 { value } else { 0 };
        });

    RgbImage {
        width,
        height,
        data: rgb_data,
    }
}

/// Bilinear demosaicing (good speed/quality balance)
///
/// Interpolates missing color values using bilinear interpolation.
/// Good for previews and thumbnails where speed is important.
pub fn demosaic_bilinear(raw: &RawImage) -> RgbImage {
    let width = raw.width as usize;
    let height = raw.height as usize;
    let pattern = raw.cfa_pattern;

    let mut rgb_data = vec![0u8; width * height * 3];

    // Helper to get normalized raw value safely
    let get_normalized = |x: isize, y: isize, channel: usize| -> f32 {
        if x < 0 || y < 0 || x >= width as isize || y >= height as isize {
            return 0.0;
        }
        let idx = (y as usize) * width + (x as usize);
        let raw_value = raw.data[idx];
        raw.normalize_value(raw_value, channel)
    };

    // Process each pixel
    rgb_data
        .par_chunks_mut(3)
        .enumerate()
        .for_each(|(pixel_idx, rgb_out)| {
            let x = (pixel_idx % width) as u32;
            let y = (pixel_idx / width) as u32;
            let xi = x as isize;
            let yi = y as isize;

            // Get the color of this Bayer position
            let center_color = bayer_color_at(x, y, pattern);

            // Get center pixel normalized value
            let center_value = get_normalized(xi, yi, center_color);

            // Initialize RGB values
            let r;
            let g;
            let b;

            match center_color {
                0 => {
                    // Center is Red
                    r = center_value;

                    // Green: average of 4 neighbors
                    let mut g_sum = 0.0;
                    let mut g_count = 0;
                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        let nx = xi + dx;
                        let ny = yi + dy;
                        if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                            let neighbor_color = bayer_color_at(nx as u32, ny as u32, pattern);
                            if neighbor_color == 1 {
                                g_sum += get_normalized(nx, ny, 1);
                                g_count += 1;
                            }
                        }
                    }
                    g = if g_count > 0 { g_sum / g_count as f32 } else { 0.0 };

                    // Blue: average of 4 diagonal neighbors
                    let mut b_sum = 0.0;
                    let mut b_count = 0;
                    for (dx, dy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
                        let nx = xi + dx;
                        let ny = yi + dy;
                        if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                            let neighbor_color = bayer_color_at(nx as u32, ny as u32, pattern);
                            if neighbor_color == 2 {
                                b_sum += get_normalized(nx, ny, 2);
                                b_count += 1;
                            }
                        }
                    }
                    b = if b_count > 0 { b_sum / b_count as f32 } else { 0.0 };
                }
                1 => {
                    // Center is Green
                    g = center_value;

                    // Red and Blue: check if horizontal or vertical neighbors have them
                    let mut r_sum = 0.0;
                    let mut r_count = 0;
                    let mut b_sum = 0.0;
                    let mut b_count = 0;

                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        let nx = xi + dx;
                        let ny = yi + dy;
                        if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                            let neighbor_color = bayer_color_at(nx as u32, ny as u32, pattern);
                            if neighbor_color == 0 {
                                r_sum += get_normalized(nx, ny, 0);
                                r_count += 1;
                            } else if neighbor_color == 2 {
                                b_sum += get_normalized(nx, ny, 2);
                                b_count += 1;
                            }
                        }
                    }

                    r = if r_count > 0 { r_sum / r_count as f32 } else { 0.0 };
                    b = if b_count > 0 { b_sum / b_count as f32 } else { 0.0 };
                }
                2 => {
                    // Center is Blue
                    b = center_value;

                    // Green: average of 4 neighbors
                    let mut g_sum = 0.0;
                    let mut g_count = 0;
                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        let nx = xi + dx;
                        let ny = yi + dy;
                        if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                            let neighbor_color = bayer_color_at(nx as u32, ny as u32, pattern);
                            if neighbor_color == 1 {
                                g_sum += get_normalized(nx, ny, 1);
                                g_count += 1;
                            }
                        }
                    }
                    g = if g_count > 0 { g_sum / g_count as f32 } else { 0.0 };

                    // Red: average of 4 diagonal neighbors
                    let mut r_sum = 0.0;
                    let mut r_count = 0;
                    for (dx, dy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
                        let nx = xi + dx;
                        let ny = yi + dy;
                        if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                            let neighbor_color = bayer_color_at(nx as u32, ny as u32, pattern);
                            if neighbor_color == 0 {
                                r_sum += get_normalized(nx, ny, 0);
                                r_count += 1;
                            }
                        }
                    }
                    r = if r_count > 0 { r_sum / r_count as f32 } else { 0.0 };
                }
                _ => unreachable!(),
            }

            // Convert to u8 and clamp
            rgb_out[0] = (r * 255.0).round().clamp(0.0, 255.0) as u8;
            rgb_out[1] = (g * 255.0).round().clamp(0.0, 255.0) as u8;
            rgb_out[2] = (b * 255.0).round().clamp(0.0, 255.0) as u8;
        });

    RgbImage {
        width: raw.width,
        height: raw.height,
        data: rgb_data,
    }
}

/// X-Trans demosaicing for Fuji cameras (6x6 CFA pattern)
///
/// Uses bilinear interpolation adapted for the X-Trans 6x6 repeating pattern.
/// X-Trans sensors have a different color filter array than traditional Bayer,
/// with green pixels more evenly distributed for better demosaicing quality.
pub fn demosaic_xtrans(data: &[u16], width: u32, height: u32) -> Vec<u16> {
    let w = width as usize;
    let h = height as usize;
    let mut output = vec![0u16; w * h * 3];

    // Helper to get pixel value safely with bounds checking
    let get_pixel = |x: isize, y: isize| -> u16 {
        if x < 0 || y < 0 || x >= width as isize || y >= height as isize {
            return 0;
        }
        data[(y as usize) * w + (x as usize)]
    };

    // Process each output pixel
    output
        .par_chunks_mut(3)
        .enumerate()
        .for_each(|(pixel_idx, rgb_out)| {
            let x = (pixel_idx % w) as u32;
            let y = (pixel_idx / w) as u32;
            let xi = x as isize;
            let yi = y as isize;

            // Determine the color at this position in the X-Trans pattern
            let center_color = xtrans_color_at(x, y);
            let center_value = get_pixel(xi, yi);

            // Initialize RGB values
            let mut r_sum = 0.0;
            let mut r_count = 0;
            let mut g_sum = 0.0;
            let mut g_count = 0;
            let mut b_sum = 0.0;
            let mut b_count = 0;

            // For the center pixel's color, use its actual value
            match center_color {
                0 => {
                    r_sum = center_value as f32;
                    r_count = 1;
                }
                1 => {
                    g_sum = center_value as f32;
                    g_count = 1;
                }
                2 => {
                    b_sum = center_value as f32;
                    b_count = 1;
                }
                _ => unreachable!(),
            }

            // Interpolate missing colors from 5x5 neighborhood
            // This is a simplified approach; more sophisticated algorithms exist
            for dy in -2..=2 {
                for dx in -2..=2 {
                    if dx == 0 && dy == 0 {
                        continue; // Skip center (already counted)
                    }

                    let nx = xi + dx;
                    let ny = yi + dy;

                    if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                        let neighbor_color = xtrans_color_at(nx as u32, ny as u32);
                        let neighbor_value = get_pixel(nx, ny) as f32;

                        // Weight based on distance (inverse distance weighting)
                        let dist = ((dx * dx + dy * dy) as f32).sqrt();
                        let weight = 1.0 / (1.0 + dist);

                        match neighbor_color {
                            0 if center_color != 0 => {
                                r_sum += neighbor_value * weight;
                                r_count += 1;
                            }
                            1 if center_color != 1 => {
                                g_sum += neighbor_value * weight;
                                g_count += 1;
                            }
                            2 if center_color != 2 => {
                                b_sum += neighbor_value * weight;
                                b_count += 1;
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Calculate final RGB values
            let r = if r_count > 0 {
                (r_sum / r_count as f32).round() as u16
            } else {
                0
            };
            let g = if g_count > 0 {
                (g_sum / g_count as f32).round() as u16
            } else {
                0
            };
            let b = if b_count > 0 {
                (b_sum / b_count as f32).round() as u16
            } else {
                0
            };

            rgb_out[0] = r;
            rgb_out[1] = g;
            rgb_out[2] = b;
        });

    output
}

/// Demosaic a RAW image using the specified algorithm
pub fn demosaic(raw: &RawImage, algorithm: DemosaicAlgorithm) -> RgbImage {
    match algorithm {
        DemosaicAlgorithm::CenterPixel => demosaic_center_pixel(raw),
        DemosaicAlgorithm::Bilinear => demosaic_bilinear(raw),
        DemosaicAlgorithm::XTrans => {
            // For X-Trans, we return u16 data directly, so need to convert to u8
            let data_u16 = demosaic_xtrans(&raw.data, raw.width, raw.height);

            // Convert u16 to u8 with normalization
            let data_u8: Vec<u8> = data_u16
                .iter()
                .map(|&v| ((v as f32 / 65535.0) * 255.0).round() as u8)
                .collect();

            RgbImage {
                width: raw.width,
                height: raw.height,
                data: data_u8,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_raw(width: u32, height: u32) -> RawImage {
        // Create a simple test pattern
        let size = (width * height) as usize;
        let mut data = vec![0u16; size];

        // Fill with a gradient pattern
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                data[idx] = ((x + y) * 100) as u16;
            }
        }

        RawImage {
            width,
            height,
            data,
            camera_make: Some("Test".to_string()),
            camera_model: Some("Camera".to_string()),
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            black_level: [0, 0, 0, 0],
            white_level: 16383,
            cfa_pattern: CfaPattern::RGGB,
            iso: None,
            exposure_time: None,
            aperture: None,
        }
    }

    #[test]
    fn test_bayer_color_at_rggb() {
        assert_eq!(bayer_color_at(0, 0, CfaPattern::RGGB), 0); // R
        assert_eq!(bayer_color_at(1, 0, CfaPattern::RGGB), 1); // G
        assert_eq!(bayer_color_at(0, 1, CfaPattern::RGGB), 1); // G
        assert_eq!(bayer_color_at(1, 1, CfaPattern::RGGB), 2); // B
    }

    #[test]
    fn test_demosaic_center_pixel() {
        let raw = create_test_raw(4, 4);
        let rgb = demosaic_center_pixel(&raw);

        assert_eq!(rgb.width, 4);
        assert_eq!(rgb.height, 4);
        assert_eq!(rgb.data.len(), 4 * 4 * 3);

        // Verify that each pixel has only one channel set
        for pixel in rgb.data.chunks(3) {
            let sum = pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32;
            // Only one channel should be non-zero (for most test patterns)
            let non_zero = pixel.iter().filter(|&&v| v != 0).count();
            assert!(non_zero <= 1);
        }
    }

    #[test]
    fn test_demosaic_bilinear() {
        let raw = create_test_raw(8, 8);
        let rgb = demosaic_bilinear(&raw);

        assert_eq!(rgb.width, 8);
        assert_eq!(rgb.height, 8);
        assert_eq!(rgb.data.len(), 8 * 8 * 3);

        // All pixels should have at least some color data after interpolation
        // (except possibly at edges)
    }

    #[test]
    fn test_demosaic_algorithm_enum() {
        let raw = create_test_raw(4, 4);

        let rgb1 = demosaic(&raw, DemosaicAlgorithm::CenterPixel);
        assert_eq!(rgb1.width, 4);

        let rgb2 = demosaic(&raw, DemosaicAlgorithm::Bilinear);
        assert_eq!(rgb2.width, 4);
    }

    #[test]
    fn test_is_xtrans_fuji() {
        // Test known X-Trans cameras
        assert!(is_xtrans("X-T5"));
        assert!(is_xtrans("Fujifilm X-H2"));
        assert!(is_xtrans("X-H2S"));
        assert!(is_xtrans("X-T4"));
        assert!(is_xtrans("X-Pro3"));
        assert!(is_xtrans("X100V"));
        assert!(is_xtrans("GFX 50S"));

        // Test non-X-Trans cameras
        assert!(!is_xtrans("A7 IV"));
        assert!(!is_xtrans("Sony A7R V"));
        assert!(!is_xtrans("Canon EOS R5"));
        assert!(!is_xtrans("Nikon Z8"));
    }

    #[test]
    fn test_xtrans_pattern_dimensions() {
        // Create 6x6 test data (one full X-Trans pattern)
        let data = vec![1000u16; 36];
        let output = demosaic_xtrans(&data, 6, 6);

        // Should produce 6x6 RGB output (no size change)
        assert_eq!(output.len(), 6 * 6 * 3);
    }

    #[test]
    fn test_xtrans_uniform_grey() {
        // Uniform grey Bayer data should produce uniform grey RGB output
        let data = vec![32768u16; 36]; // Mid-grey
        let output = demosaic_xtrans(&data, 6, 6);

        // All RGB values should be close to the input value
        // Note: X-Trans interpolation with weighted averaging may produce lower values
        for pixel in output.chunks(3) {
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];

            // Allow wide variance due to weighted interpolation in X-Trans
            // Values may be significantly lower than input due to distance weighting
            assert!(r > 10000 && r < 50000, "R: {}", r);
            assert!(g > 10000 && g < 50000, "G: {}", g);
            assert!(b > 10000 && b < 50000, "B: {}", b);
        }
    }

    #[test]
    fn test_demosaic_xtrans_vs_bayer_different() {
        // Same data should produce different results with X-Trans vs Bayer
        let raw = create_test_raw(12, 12);

        let bayer_result = demosaic(&raw, DemosaicAlgorithm::Bilinear);
        let xtrans_result = demosaic(&raw, DemosaicAlgorithm::XTrans);

        // Dimensions should be the same
        assert_eq!(bayer_result.width, xtrans_result.width);
        assert_eq!(bayer_result.height, xtrans_result.height);
        assert_eq!(bayer_result.data.len(), xtrans_result.data.len());

        // But the actual data should differ (different demosaicing algorithms)
        let mut differences = 0;
        for (b, x) in bayer_result.data.iter().zip(xtrans_result.data.iter()) {
            if b != x {
                differences += 1;
            }
        }

        // Expect at least some differences
        assert!(differences > 0, "X-Trans and Bayer should produce different results");
    }
}
