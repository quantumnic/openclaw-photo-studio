//! Histogram computation for RGB images.
//!
//! This module provides histogram analysis for RGB images, including per-channel
//! histograms and luminance-based statistics.

/// A histogram containing red, green, blue, and luminance distributions.
///
/// Each channel has 256 bins (0-255), where each bin contains the count
/// of pixels with that value.
#[derive(Debug, Clone)]
pub struct Histogram {
    /// Red channel histogram (256 bins)
    pub red: [u32; 256],
    /// Green channel histogram (256 bins)
    pub green: [u32; 256],
    /// Blue channel histogram (256 bins)
    pub blue: [u32; 256],
    /// Luminance histogram (256 bins), computed using Rec. 709 weights
    pub luma: [u32; 256],
}

impl Histogram {
    /// Compute histogram from 8-bit RGB image data.
    ///
    /// # Arguments
    ///
    /// * `data` - RGB pixel data in row-major order (R,G,B,R,G,B,...)
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    ///
    /// # Panics
    ///
    /// Panics if `data.len() != width * height * 3`.
    pub fn from_rgb8(data: &[u8], width: u32, height: u32) -> Self {
        let expected_len = (width * height * 3) as usize;
        assert_eq!(
            data.len(),
            expected_len,
            "Data length mismatch: expected {}, got {}",
            expected_len,
            data.len()
        );

        let mut histogram = Histogram {
            red: [0; 256],
            green: [0; 256],
            blue: [0; 256],
            luma: [0; 256],
        };

        // Process pixels in chunks of 3 (RGB)
        for pixel in data.chunks_exact(3) {
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];

            // Increment channel histograms
            histogram.red[r as usize] += 1;
            histogram.green[g as usize] += 1;
            histogram.blue[b as usize] += 1;

            // Compute luminance using Rec. 709 coefficients
            // Y = 0.2126*R + 0.7152*G + 0.0722*B
            let luma = (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32).round() as u8;
            histogram.luma[luma as usize] += 1;
        }

        histogram
    }

    /// Compute mean luminance value (0.0 - 255.0).
    ///
    /// This is the weighted average of all luminance values.
    pub fn mean_luma(&self) -> f32 {
        let total_pixels: u32 = self.luma.iter().sum();
        if total_pixels == 0 {
            return 0.0;
        }

        let weighted_sum: u64 = self
            .luma
            .iter()
            .enumerate()
            .map(|(value, &count)| (value as u64) * (count as u64))
            .sum();

        weighted_sum as f32 / total_pixels as f32
    }

    /// Compute percentage of shadow clipping.
    ///
    /// Returns the percentage of pixels (0.0 - 100.0) with luminance at or below
    /// the specified threshold.
    ///
    /// # Arguments
    ///
    /// * `threshold` - Luminance value threshold (typically 0-10)
    pub fn clipped_shadows(&self, threshold: u32) -> f32 {
        let threshold = threshold.min(255) as usize;
        let total_pixels: u32 = self.luma.iter().sum();
        if total_pixels == 0 {
            return 0.0;
        }

        let clipped: u32 = self.luma[0..=threshold].iter().sum();
        (clipped as f32 / total_pixels as f32) * 100.0
    }

    /// Compute percentage of highlight clipping.
    ///
    /// Returns the percentage of pixels (0.0 - 100.0) with luminance at or above
    /// the specified threshold.
    ///
    /// # Arguments
    ///
    /// * `threshold` - Luminance value threshold (typically 245-255)
    pub fn clipped_highlights(&self, threshold: u32) -> f32 {
        let threshold = threshold.min(255) as usize;
        let total_pixels: u32 = self.luma.iter().sum();
        if total_pixels == 0 {
            return 0.0;
        }

        let clipped: u32 = self.luma[threshold..=255].iter().sum();
        (clipped as f32 / total_pixels as f32) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_all_black() {
        // Create 10x10 black image
        let data = vec![0u8; 10 * 10 * 3];
        let hist = Histogram::from_rgb8(&data, 10, 10);

        // All pixels should be in bin 0
        assert_eq!(hist.red[0], 100);
        assert_eq!(hist.green[0], 100);
        assert_eq!(hist.blue[0], 100);
        assert_eq!(hist.luma[0], 100);

        // All other bins should be 0
        for i in 1..256 {
            assert_eq!(hist.red[i], 0);
            assert_eq!(hist.green[i], 0);
            assert_eq!(hist.blue[i], 0);
            assert_eq!(hist.luma[i], 0);
        }
    }

    #[test]
    fn test_histogram_all_white() {
        // Create 10x10 white image
        let data = vec![255u8; 10 * 10 * 3];
        let hist = Histogram::from_rgb8(&data, 10, 10);

        // All pixels should be in bin 255
        assert_eq!(hist.red[255], 100);
        assert_eq!(hist.green[255], 100);
        assert_eq!(hist.blue[255], 100);
        assert_eq!(hist.luma[255], 100);

        // All other bins should be 0
        for i in 0..255 {
            assert_eq!(hist.red[i], 0);
            assert_eq!(hist.green[i], 0);
            assert_eq!(hist.blue[i], 0);
            assert_eq!(hist.luma[i], 0);
        }
    }

    #[test]
    fn test_histogram_midgray() {
        // Create 10x10 mid-gray image (128, 128, 128)
        let mut data = Vec::with_capacity(10 * 10 * 3);
        for _ in 0..100 {
            data.extend_from_slice(&[128u8, 128u8, 128u8]);
        }
        let hist = Histogram::from_rgb8(&data, 10, 10);

        // All pixels should be in bin 128
        assert_eq!(hist.red[128], 100);
        assert_eq!(hist.green[128], 100);
        assert_eq!(hist.blue[128], 100);
        assert_eq!(hist.luma[128], 100);

        // All other bins should be 0
        for i in 0..256 {
            if i != 128 {
                assert_eq!(hist.red[i], 0);
                assert_eq!(hist.green[i], 0);
                assert_eq!(hist.blue[i], 0);
                assert_eq!(hist.luma[i], 0);
            }
        }
    }

    #[test]
    fn test_histogram_luma_calculation() {
        // Create image with pure red pixels (255, 0, 0)
        // Expected luma: 0.2126 * 255 = 54.213 ≈ 54
        let mut data = Vec::with_capacity(10 * 10 * 3);
        for _ in 0..100 {
            data.extend_from_slice(&[255u8, 0u8, 0u8]);
        }
        let hist = Histogram::from_rgb8(&data, 10, 10);

        // Red channel should have all pixels at 255
        assert_eq!(hist.red[255], 100);

        // Luma should be around bin 54
        assert_eq!(hist.luma[54], 100);
    }

    #[test]
    fn test_histogram_clipping() {
        // Create image with black, mid-gray, and white pixels
        let mut data = Vec::with_capacity(30 * 3);
        // 10 black pixels
        for _ in 0..10 {
            data.extend_from_slice(&[0u8, 0u8, 0u8]);
        }
        // 10 mid-gray pixels
        for _ in 0..10 {
            data.extend_from_slice(&[128u8, 128u8, 128u8]);
        }
        // 10 white pixels
        for _ in 0..10 {
            data.extend_from_slice(&[255u8, 255u8, 255u8]);
        }

        let hist = Histogram::from_rgb8(&data, 30, 1);

        // Shadow clipping at threshold 0 should be ~33.33% (10/30)
        let shadow_clip = hist.clipped_shadows(0);
        assert!((shadow_clip - 33.33).abs() < 0.1);

        // Highlight clipping at threshold 255 should be ~33.33% (10/30)
        let highlight_clip = hist.clipped_highlights(255);
        assert!((highlight_clip - 33.33).abs() < 0.1);

        // Mean luma should be around 128 (balanced distribution)
        let mean = hist.mean_luma();
        assert!((mean - 128.0).abs() < 5.0);
    }

    #[test]
    fn test_mean_luma() {
        // Create gradient from black to white
        let mut data = Vec::with_capacity(256 * 3);
        for i in 0..256 {
            data.extend_from_slice(&[i as u8, i as u8, i as u8]);
        }
        let hist = Histogram::from_rgb8(&data, 256, 1);

        // Mean should be around 127.5 (middle of 0-255)
        let mean = hist.mean_luma();
        assert!((mean - 127.5).abs() < 1.0);
    }

    #[test]
    fn test_empty_histogram() {
        // Edge case: 0x0 image
        let data: Vec<u8> = vec![];
        let hist = Histogram::from_rgb8(&data, 0, 0);

        assert_eq!(hist.mean_luma(), 0.0);
        assert_eq!(hist.clipped_shadows(0), 0.0);
        assert_eq!(hist.clipped_highlights(255), 0.0);
    }

    #[test]
    #[should_panic(expected = "Data length mismatch")]
    fn test_invalid_data_length() {
        // Should panic if data length doesn't match width * height * 3
        let data = vec![0u8; 100];
        let _ = Histogram::from_rgb8(&data, 10, 10); // Expects 300 bytes
    }
}
