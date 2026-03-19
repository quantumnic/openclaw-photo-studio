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

/// Demosaic a RAW image using the specified algorithm
pub fn demosaic(raw: &RawImage, algorithm: DemosaicAlgorithm) -> RgbImage {
    match algorithm {
        DemosaicAlgorithm::CenterPixel => demosaic_center_pixel(raw),
        DemosaicAlgorithm::Bilinear => demosaic_bilinear(raw),
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
}
