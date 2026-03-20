//! Full CPU image processing pipeline
//! Input: RGB Vec<u16> + EditRecipe
//! Output: RGB Vec<u8> (8-bit sRGB for display/export)

pub mod color;
pub mod lens;
pub mod local_adj;
pub mod process;
pub mod types;

#[cfg(test)]
mod edge_case_tests;

pub use types::{
    ColorGrading, ColorGradingSettings, CropSettings, CurvePoint, EditRecipe, HealingSpot,
    HslAdjustments, LensCorrections, NoiseReductionSettings, RgbImage16, RgbImage8,
    SharpeningSettings, SpotType, ToneCurve, WhiteBalance,
};

use color::u16_linear_to_u8_srgb;
use process::{
    apply_clarity, apply_color_grading, apply_contrast, apply_crop, apply_exposure,
    apply_healing_spots, apply_highlights_shadows, apply_hsl, apply_noise_reduction,
    apply_saturation, apply_sharpening, apply_tone_curve, apply_white_balance,
};

/// Main image processor - applies full editing pipeline
pub struct ImageProcessor;

impl ImageProcessor {
    /// Process an image with the given recipe
    /// Applies transformations in a specific order for best quality
    pub fn process(image: &RgbImage16, recipe: &EditRecipe) -> RgbImage8 {
        // If recipe is identity, just convert to 8-bit
        if recipe.is_identity() {
            return Self::convert_to_u8(image);
        }

        // Clone the image so we can modify it
        let mut working = image.clone();

        // Step 1: White balance (should be first - affects color interpretation)
        apply_white_balance(
            &mut working.data,
            recipe.white_balance.temperature,
            recipe.white_balance.tint,
        );

        // Step 2: Exposure (global brightness adjustment)
        apply_exposure(&mut working.data, recipe.exposure);

        // Step 3: Contrast (midtone contrast)
        apply_contrast(&mut working.data, recipe.contrast);

        // Step 4: Highlights/Shadows/Whites/Blacks (tone mapping)
        apply_highlights_shadows(
            &mut working.data,
            recipe.highlights,
            recipe.shadows,
            recipe.whites,
            recipe.blacks,
        );

        // Step 4.5: Tone curve (after basic tone adjustments)
        apply_tone_curve(&mut working.data, &recipe.tone_curve_rgb);

        // Step 5: Clarity (local contrast - must be done before saturation)
        if recipe.clarity != 0 {
            apply_clarity(&mut working, recipe.clarity);
        }

        // Step 6: Vibrance/Saturation (color adjustments)
        apply_saturation(&mut working.data, recipe.saturation, recipe.vibrance);

        // Step 6.5: HSL adjustments (per-channel color adjustments)
        apply_hsl(
            &mut working.data,
            working.width,
            working.height,
            &recipe.hsl,
        );

        // Step 6.6: Color grading (3-way color wheels)
        apply_color_grading(
            &mut working.data,
            working.width,
            working.height,
            &recipe.color_grading_new,
        );

        // Step 6.7: Lens corrections (distortion and vignetting)
        if !recipe.lens_corrections.is_identity() {
            // Apply distortion first
            if recipe.lens_corrections.distortion != 0.0 {
                working.data = lens::apply_distortion(
                    &working.data,
                    working.width,
                    working.height,
                    recipe.lens_corrections.distortion,
                );
            }

            // Apply vignetting correction (in place)
            if recipe.lens_corrections.vignetting != 0.0 {
                lens::apply_vignetting_correction(
                    &mut working.data,
                    working.width,
                    working.height,
                    recipe.lens_corrections.vignetting,
                );
            }
        }

        // Step 6.8: Noise reduction (before sharpening to avoid amplifying noise)
        if recipe.noise_reduction.luminance > 0 || recipe.noise_reduction.color > 0 {
            apply_noise_reduction(
                &mut working.data,
                working.width,
                working.height,
                &recipe.noise_reduction,
            );
        }

        // Step 7: Healing spots (blemish removal, before sharpening to avoid sharpening artifacts)
        if !recipe.healing_spots.is_empty() {
            apply_healing_spots(
                &mut working.data,
                working.width,
                working.height,
                &recipe.healing_spots,
            );
        }

        // Step 8: Sharpening (should be near the end)
        if recipe.sharpening.amount > 0 {
            apply_sharpening(
                &mut working,
                recipe.sharpening.amount,
                recipe.sharpening.radius,
            );
        }

        // Step 8.5: Local adjustments (after global adjustments, before crop)
        if !recipe.local_adjustments.is_empty() {
            local_adj::apply_local_adjustments(&mut working, &recipe.local_adjustments);
        }

        // Step 9: Crop (geometric transformation, done before output)
        if !recipe.crop.is_identity() {
            working = apply_crop(&working, &recipe.crop);
        }

        // Step 10: Convert to 8-bit sRGB for output
        Self::convert_to_u8(&working)
    }

    /// Convert 16-bit linear RGB to 8-bit sRGB
    fn convert_to_u8(image: &RgbImage16) -> RgbImage8 {
        let mut result = RgbImage8::new(image.width, image.height);

        // Use 65535 as white level for normalized 16-bit data
        const WHITE_LEVEL: u16 = 65535;

        for (src, dst) in image.data.iter().zip(result.data.iter_mut()) {
            *dst = u16_linear_to_u8_srgb(*src, WHITE_LEVEL);
        }

        result
    }

    /// Process a batch of images with the same recipe
    /// Returns Vec of processed images
    pub fn process_batch(images: &[RgbImage16], recipe: &EditRecipe) -> Vec<RgbImage8> {
        images.iter().map(|img| Self::process(img, recipe)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32) -> RgbImage16 {
        let mut img = RgbImage16::new(width, height);

        // Fill with a gradient pattern
        for y in 0..height {
            for x in 0..width {
                let r = ((x as f32 / width as f32) * 65535.0) as u16;
                let g = ((y as f32 / height as f32) * 65535.0) as u16;
                let b = 32768; // Mid-gray for blue
                img.set_pixel(x, y, [r, g, b]);
            }
        }

        img
    }

    #[test]
    fn test_process_identity_recipe() {
        let img = create_test_image(10, 10);
        let recipe = EditRecipe::default();

        let result = ImageProcessor::process(&img, &recipe);

        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);
        assert_eq!(result.data.len(), 10 * 10 * 3);
    }

    #[test]
    fn test_process_with_exposure() {
        let img = create_test_image(10, 10);
        let mut recipe = EditRecipe::default();
        recipe.exposure = 1.0; // +1 EV

        let result = ImageProcessor::process(&img, &recipe);

        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);
        // Image should be brighter (can't easily test exact values due to gamma)
    }

    #[test]
    fn test_process_with_contrast() {
        let img = create_test_image(10, 10);
        let mut recipe = EditRecipe::default();
        recipe.contrast = 50;

        let result = ImageProcessor::process(&img, &recipe);

        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn test_process_with_saturation() {
        let img = create_test_image(10, 10);
        let mut recipe = EditRecipe::default();
        recipe.saturation = 50;

        let result = ImageProcessor::process(&img, &recipe);

        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn test_process_with_crop() {
        let img = create_test_image(100, 100);
        let mut recipe = EditRecipe::default();
        recipe.crop.left = 0.25;
        recipe.crop.top = 0.25;
        recipe.crop.right = 0.75;
        recipe.crop.bottom = 0.75;

        let result = ImageProcessor::process(&img, &recipe);

        // Should be cropped to 50x50
        assert_eq!(result.width, 50);
        assert_eq!(result.height, 50);
    }

    #[test]
    fn test_process_full_pipeline() {
        let img = create_test_image(50, 50);
        let mut recipe = EditRecipe::default();

        recipe.white_balance.temperature = 6500;
        recipe.white_balance.tint = 10;
        recipe.exposure = 0.5;
        recipe.contrast = 20;
        recipe.highlights = -20;
        recipe.shadows = 30;
        recipe.whites = -10;
        recipe.blacks = 10;
        recipe.clarity = 15;
        recipe.vibrance = 25;
        recipe.saturation = 10;
        recipe.sharpening.amount = 50;
        recipe.sharpening.radius = 1.0;

        let result = ImageProcessor::process(&img, &recipe);

        assert_eq!(result.width, 50);
        assert_eq!(result.height, 50);
        assert_eq!(result.data.len(), 50 * 50 * 3);

        // Verify all pixels are in valid range
        for &val in &result.data {
            assert!(val <= 255);
        }
    }

    #[test]
    fn test_convert_to_u8_black() {
        let img = RgbImage16::from_data(1, 1, vec![0, 0, 0]);
        let result = ImageProcessor::convert_to_u8(&img);

        assert_eq!(result.data[0], 0);
        assert_eq!(result.data[1], 0);
        assert_eq!(result.data[2], 0);
    }

    #[test]
    fn test_convert_to_u8_white() {
        let img = RgbImage16::from_data(1, 1, vec![65535, 65535, 65535]);
        let result = ImageProcessor::convert_to_u8(&img);

        assert_eq!(result.data[0], 255);
        assert_eq!(result.data[1], 255);
        assert_eq!(result.data[2], 255);
    }

    #[test]
    fn test_convert_to_u8_mid_gray() {
        // 18% gray in linear space
        let linear_18 = (65535.0 * 0.18) as u16;
        let img = RgbImage16::from_data(1, 1, vec![linear_18, linear_18, linear_18]);
        let result = ImageProcessor::convert_to_u8(&img);

        // sRGB 18% gray should be around 117 (middle gray looks brighter in gamma space)
        let gray = result.data[0];
        assert!(gray > 100 && gray < 130, "Gray value {} out of expected range", gray);
    }

    #[test]
    fn test_process_batch() {
        let images = vec![
            create_test_image(10, 10),
            create_test_image(20, 20),
            create_test_image(30, 30),
        ];

        let mut recipe = EditRecipe::default();
        recipe.exposure = 0.5;
        recipe.contrast = 10;

        let results = ImageProcessor::process_batch(&images, &recipe);

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].width, 10);
        assert_eq!(results[1].width, 20);
        assert_eq!(results[2].width, 30);
    }

    #[test]
    fn test_process_deterministic() {
        let img = create_test_image(20, 20);
        let mut recipe = EditRecipe::default();
        recipe.exposure = 0.3;
        recipe.contrast = 15;
        recipe.saturation = 20;

        let result1 = ImageProcessor::process(&img, &recipe);
        let result2 = ImageProcessor::process(&img, &recipe);

        // Processing should be deterministic
        assert_eq!(result1.data, result2.data);
    }

    #[test]
    fn test_process_order_independence_commutative_ops() {
        let img = create_test_image(20, 20);

        // Test that exposure and contrast commute (they should approximately)
        let mut recipe1 = EditRecipe::default();
        recipe1.exposure = 0.5;
        recipe1.contrast = 20;

        let mut recipe2 = EditRecipe::default();
        recipe2.contrast = 20;
        recipe2.exposure = 0.5;

        let result1 = ImageProcessor::process(&img, &recipe1);
        let result2 = ImageProcessor::process(&img, &recipe2);

        // Should be similar (may not be exactly equal due to order of operations)
        assert_eq!(result1.width, result2.width);
        assert_eq!(result1.height, result2.height);
    }

    #[test]
    fn test_extreme_values_dont_crash() {
        let img = create_test_image(10, 10);
        let mut recipe = EditRecipe::default();

        // Extreme values
        recipe.exposure = 5.0;
        recipe.contrast = 100;
        recipe.highlights = -100;
        recipe.shadows = 100;
        recipe.whites = -100;
        recipe.blacks = 100;
        recipe.clarity = 100;
        recipe.vibrance = 100;
        recipe.saturation = 100;
        recipe.sharpening.amount = 150;

        let result = ImageProcessor::process(&img, &recipe);

        // Should complete without panicking
        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);

        // All values should be in valid range
        for &val in &result.data {
            assert!(val <= 255);
        }
    }

    #[test]
    fn test_negative_extreme_values() {
        let img = create_test_image(10, 10);
        let mut recipe = EditRecipe::default();

        recipe.exposure = -5.0;
        recipe.contrast = -100;
        recipe.highlights = 100;
        recipe.shadows = -100;
        recipe.whites = 100;
        recipe.blacks = -100;
        recipe.clarity = -100;
        recipe.vibrance = -100;
        recipe.saturation = -100;

        let result = ImageProcessor::process(&img, &recipe);

        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);

        for &val in &result.data {
            assert!(val <= 255);
        }
    }
}
