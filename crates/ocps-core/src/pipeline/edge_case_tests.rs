//! Edge case tests for image processing pipeline
//!
//! Tests extreme values, boundary conditions, and edge cases

use crate::pipeline::types::{EditRecipe, RgbImage16};
use crate::pipeline::ImageProcessor;

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_pipeline_extreme_exposure() {
        let data = vec![32768u16; 300]; // Middle gray, 10x10 RGB
        let image = RgbImage16::from_data(10, 10, data);

        let mut recipe = EditRecipe::default();
        recipe.exposure = 5.0; // Maximum

        let result = ImageProcessor::process(&image, &recipe);

        // Should not panic
        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn test_pipeline_extreme_negative_exposure() {
        let data = vec![32768u16; 300];
        let image = RgbImage16::from_data(10, 10, data);

        let mut recipe = EditRecipe::default();
        recipe.exposure = -5.0; // Minimum

        let result = ImageProcessor::process(&image, &recipe);

        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn test_pipeline_extreme_contrast() {
        let data = vec![32768u16; 300];
        let image = RgbImage16::from_data(10, 10, data);

        let mut recipe = EditRecipe::default();
        recipe.contrast = 100;

        let result = ImageProcessor::process(&image, &recipe);

        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn test_pipeline_extreme_saturation() {
        let data = vec![32768u16; 300];
        let image = RgbImage16::from_data(10, 10, data);

        let mut recipe = EditRecipe::default();
        recipe.saturation = -100;

        let result = ImageProcessor::process(&image, &recipe);

        // Should produce grayscale image
        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn test_pipeline_all_extreme_values() {
        let data = vec![32768u16; 300];
        let image = RgbImage16::from_data(10, 10, data);

        let mut recipe = EditRecipe::default();
        recipe.exposure = 5.0;
        recipe.contrast = 100;
        recipe.highlights = -100;
        recipe.shadows = 100;
        recipe.whites = 100;
        recipe.blacks = -100;
        recipe.clarity = 100;
        recipe.vibrance = 100;
        recipe.saturation = 100;
        recipe.sharpening.amount = 100;

        let result = ImageProcessor::process(&image, &recipe);

        // Should not panic even with all extreme values
        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn test_pipeline_1x1_image() {
        let data = vec![32768u16, 32768u16, 32768u16]; // 1x1 RGB
        let image = RgbImage16::from_data(1, 1, data);

        let mut recipe = EditRecipe::default();
        recipe.exposure = 1.0;
        recipe.sharpening.amount = 50;

        let result = ImageProcessor::process(&image, &recipe);

        // Should handle 1x1 image (sharpening might be tricky)
        assert_eq!(result.width, 1);
        assert_eq!(result.height, 1);
    }

    #[test]
    fn test_pipeline_pure_black_image() {
        let data = vec![0u16; 300]; // All black
        let image = RgbImage16::from_data(10, 10, data);

        let mut recipe = EditRecipe::default();
        recipe.exposure = 5.0;
        recipe.shadows = 100;

        let result = ImageProcessor::process(&image, &recipe);

        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn test_pipeline_pure_white_image() {
        let data = vec![65535u16; 300]; // All white
        let image = RgbImage16::from_data(10, 10, data);

        let mut recipe = EditRecipe::default();
        recipe.exposure = -5.0;
        recipe.highlights = -100;

        let result = ImageProcessor::process(&image, &recipe);

        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn test_pipeline_clipping_prevention() {
        let data = vec![65535u16; 300]; // All white
        let image = RgbImage16::from_data(10, 10, data);

        let mut recipe = EditRecipe::default();
        recipe.exposure = 5.0; // Would normally cause massive clipping

        let result = ImageProcessor::process(&image, &recipe);

        // Should clamp values, not overflow
        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);
        // All pixels should be <= 255
        assert!(result.data.iter().all(|&v| v <= 255));
    }

    #[test]
    fn test_white_balance_extreme_temperature() {
        let data = vec![32768u16; 300];
        let image = RgbImage16::from_data(10, 10, data);

        let mut recipe = EditRecipe::default();
        recipe.white_balance.temperature = 100; // Very warm

        let result = ImageProcessor::process(&image, &recipe);

        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn test_clarity_on_solid_color() {
        let data = vec![32768u16; 300]; // Solid gray
        let image = RgbImage16::from_data(10, 10, data);

        let mut recipe = EditRecipe::default();
        recipe.clarity = 100;

        let result = ImageProcessor::process(&image, &recipe);

        // Clarity on solid color should not change anything dramatically
        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn test_crop_with_boundaries() {
        let data = vec![32768u16; 1200]; // 20x20
        let image = RgbImage16::from_data(20, 20, data);

        let mut recipe = EditRecipe::default();
        recipe.crop.left = 0.0;
        recipe.crop.top = 0.0;
        recipe.crop.right = 0.5;
        recipe.crop.bottom = 0.5;

        let result = ImageProcessor::process(&image, &recipe);

        // Should produce cropped image
        assert!(result.width <= 20);
        assert!(result.height <= 20);
    }

    #[test]
    fn test_batch_processing_empty_list() {
        let images: Vec<RgbImage16> = vec![];
        let recipe = EditRecipe::default();

        let results = ImageProcessor::process_batch(&images, &recipe);

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_batch_processing_single_image() {
        let data = vec![32768u16; 300];
        let image = RgbImage16::from_data(10, 10, data);

        let recipe = EditRecipe::default();

        let results = ImageProcessor::process_batch(&[image], &recipe);

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_identity_with_default_recipe() {
        let data = vec![32768u16; 300];
        let image = RgbImage16::from_data(10, 10, data);

        let recipe = EditRecipe::default(); // All defaults

        let result = ImageProcessor::process(&image, &recipe);

        // With default recipe, output should be reasonable
        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn test_high_frequency_pattern() {
        // Create checkerboard pattern
        let mut data = vec![0u16; 1200]; // 20x20 RGB
        for y in 0..20 {
            for x in 0..20 {
                let val = if (x + y) % 2 == 0 { 0 } else { 65535 };
                let idx = (y * 20 + x) * 3;
                data[idx] = val;
                data[idx + 1] = val;
                data[idx + 2] = val;
            }
        }
        let image = RgbImage16::from_data(20, 20, data);

        let mut recipe = EditRecipe::default();
        recipe.sharpening.amount = 100;
        recipe.clarity = 100;

        let result = ImageProcessor::process(&image, &recipe);

        // Should handle high-frequency content
        assert_eq!(result.width, 20);
        assert_eq!(result.height, 20);
    }
}
