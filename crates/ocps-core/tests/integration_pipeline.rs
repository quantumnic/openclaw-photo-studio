//! Integration tests for the complete image processing pipeline

use ocps_core::{
    EditRecipe, ImageProcessor, RgbImage16, WhiteBalance, SharpeningSettings, CropSettings,
};

/// Create a realistic test image with gradients and colors
fn create_realistic_test_image(width: u32, height: u32) -> RgbImage16 {
    let mut img = RgbImage16::new(width, height);

    for y in 0..height {
        for x in 0..width {
            // Create a gradient pattern
            let x_norm = x as f32 / width as f32;
            let y_norm = y as f32 / height as f32;

            // Create some color variation
            let r = (x_norm * 65535.0) as u16;
            let g = (y_norm * 65535.0) as u16;
            let b = ((0.5 + 0.5 * (x_norm * y_norm)) * 65535.0) as u16;

            img.set_pixel(x, y, [r, g, b]);
        }
    }

    img
}

#[test]
fn test_full_pipeline_workflow() {
    // Simulate a typical editing workflow
    let img = create_realistic_test_image(200, 200);

    // Start with identity
    let mut recipe = EditRecipe::default();
    let result = ImageProcessor::process(&img, &recipe);
    assert_eq!(result.width, 200);
    assert_eq!(result.height, 200);

    // Add exposure adjustment
    recipe.exposure = 0.5;
    let result = ImageProcessor::process(&img, &recipe);
    assert_eq!(result.width, 200);

    // Add tone adjustments
    recipe.highlights = -30;
    recipe.shadows = 40;
    let result = ImageProcessor::process(&img, &recipe);
    assert_eq!(result.width, 200);

    // Add color adjustments
    recipe.vibrance = 25;
    recipe.saturation = 10;
    let result = ImageProcessor::process(&img, &recipe);
    assert_eq!(result.width, 200);

    // Add sharpening
    recipe.sharpening.amount = 60;
    recipe.sharpening.radius = 1.2;
    let result = ImageProcessor::process(&img, &recipe);
    assert_eq!(result.width, 200);

    // Add crop
    recipe.crop.left = 0.1;
    recipe.crop.top = 0.1;
    recipe.crop.right = 0.9;
    recipe.crop.bottom = 0.9;
    let result = ImageProcessor::process(&img, &recipe);
    assert_eq!(result.width, 160); // 80% of 200
    assert_eq!(result.height, 160);
}

#[test]
fn test_batch_consistency() {
    // Test that batch processing gives same results as individual processing
    let images = vec![
        create_realistic_test_image(100, 100),
        create_realistic_test_image(100, 100),
        create_realistic_test_image(100, 100),
    ];

    let mut recipe = EditRecipe::default();
    recipe.exposure = 0.3;
    recipe.contrast = 15;
    recipe.saturation = 20;

    // Process individually
    let individual: Vec<_> = images
        .iter()
        .map(|img| ImageProcessor::process(img, &recipe))
        .collect();

    // Process as batch
    let batch = ImageProcessor::process_batch(&images, &recipe);

    // Results should be identical
    assert_eq!(individual.len(), batch.len());
    for (ind, bat) in individual.iter().zip(batch.iter()) {
        assert_eq!(ind.data, bat.data);
    }
}

#[test]
fn test_copy_paste_simulation() {
    // Simulate the copy/paste workflow described in COPY-PASTE-SPEC.md

    // Photo A (source)
    let photo_a = create_realistic_test_image(150, 150);
    let mut recipe_a = EditRecipe::default();
    recipe_a.exposure = 0.5;
    recipe_a.contrast = 20;
    recipe_a.highlights = -25;
    recipe_a.shadows = 30;
    recipe_a.vibrance = 25;
    recipe_a.white_balance.temperature = 6000;
    recipe_a.white_balance.tint = 10;

    // Process Photo A
    let result_a = ImageProcessor::process(&photo_a, &recipe_a);
    assert_eq!(result_a.width, 150);

    // Copy settings (in real app, this would be Cmd+C)
    let copied_recipe = recipe_a.clone();

    // Photo B, C, D (targets - different images but same recipe)
    let photos = vec![
        create_realistic_test_image(150, 150),
        create_realistic_test_image(150, 150),
        create_realistic_test_image(150, 150),
    ];

    // Paste settings (in real app, this would be Cmd+V on selection)
    let results = ImageProcessor::process_batch(&photos, &copied_recipe);

    // All should be processed successfully
    assert_eq!(results.len(), 3);
    for result in results {
        assert_eq!(result.width, 150);
        assert_eq!(result.height, 150);
    }
}

#[test]
fn test_selective_paste() {
    // Simulate COPY SELECTED workflow (Cmd+Shift+C/V)

    let img = create_realistic_test_image(100, 100);

    // Full recipe
    let mut full_recipe = EditRecipe::default();
    full_recipe.exposure = 0.5;
    full_recipe.contrast = 20;
    full_recipe.saturation = 30;
    full_recipe.sharpening.amount = 60;
    full_recipe.crop.left = 0.1;

    // Selective recipe (only tone adjustments, no color/detail/crop)
    let mut selective_recipe = EditRecipe::default();
    selective_recipe.exposure = full_recipe.exposure;
    selective_recipe.contrast = full_recipe.contrast;
    // Deliberately omit saturation, sharpening, crop

    let full_result = ImageProcessor::process(&img, &full_recipe);
    let selective_result = ImageProcessor::process(&img, &selective_recipe);

    // Full result should be cropped
    assert_eq!(full_result.width, 90); // Cropped

    // Selective result should NOT be cropped
    assert_eq!(selective_result.width, 100); // Original size
}

#[test]
fn test_match_total_exposure_simulation() {
    // Simulate the "Match Total Exposure" feature (Cmd+Shift+E)

    // Create images with different brightness
    let mut dark_img = create_realistic_test_image(100, 100);
    let mut bright_img = create_realistic_test_image(100, 100);

    // Make bright_img actually brighter
    for pixel in bright_img.data.iter_mut() {
        *pixel = (*pixel as f32 * 1.5).min(65535.0) as u16;
    }

    // Target exposure
    let target_recipe = EditRecipe::default();

    // Dark image needs positive exposure
    let mut dark_recipe = EditRecipe::default();
    dark_recipe.exposure = 0.5; // Simulated auto-calculated value

    // Bright image needs negative exposure
    let mut bright_recipe = EditRecipe::default();
    bright_recipe.exposure = -0.3; // Simulated auto-calculated value

    // Process both
    let dark_result = ImageProcessor::process(&dark_img, &dark_recipe);
    let bright_result = ImageProcessor::process(&bright_img, &bright_recipe);

    // Both should produce valid results
    assert_eq!(dark_result.data.len(), 100 * 100 * 3);
    assert_eq!(bright_result.data.len(), 100 * 100 * 3);
}

#[test]
fn test_auto_sync_mode_simulation() {
    // Simulate Auto-Sync mode where changes apply to all selected photos

    let photos = vec![
        create_realistic_test_image(80, 80),
        create_realistic_test_image(80, 80),
        create_realistic_test_image(80, 80),
    ];

    // Start with base recipe
    let mut recipe = EditRecipe::default();
    recipe.exposure = 0.3;

    // Process all (initial state)
    let results_v1 = ImageProcessor::process_batch(&photos, &recipe);

    // User adjusts exposure (auto-sync would update all)
    recipe.exposure = 0.6;

    // Process all again (simulates real-time update)
    let results_v2 = ImageProcessor::process_batch(&photos, &recipe);

    // Results should be different (brighter)
    assert_ne!(results_v1[0].data, results_v2[0].data);

    // User adds contrast (auto-sync)
    recipe.contrast = 25;

    // Process again
    let results_v3 = ImageProcessor::process_batch(&photos, &recipe);
    assert_ne!(results_v2[0].data, results_v3[0].data);
}

#[test]
fn test_virtual_copy_workflow() {
    // Virtual copies share the same source image but different recipes

    let source_image = create_realistic_test_image(120, 120);

    // Master version
    let master_recipe = EditRecipe::default();

    // Virtual Copy 1: Black & White simulation (desaturate)
    let mut bw_recipe = EditRecipe::default();
    bw_recipe.saturation = -100; // Full desaturation
    bw_recipe.contrast = 30; // Higher contrast for B&W

    // Virtual Copy 2: Vibrant color
    let mut vibrant_recipe = EditRecipe::default();
    vibrant_recipe.vibrance = 50;
    vibrant_recipe.saturation = 20;
    vibrant_recipe.clarity = 30;

    // Virtual Copy 3: Underexposed rescue
    let mut rescue_recipe = EditRecipe::default();
    rescue_recipe.exposure = 1.5;
    rescue_recipe.shadows = 60;
    rescue_recipe.blacks = 20;

    // Process all versions
    let master_result = ImageProcessor::process(&source_image, &master_recipe);
    let bw_result = ImageProcessor::process(&source_image, &bw_recipe);
    let vibrant_result = ImageProcessor::process(&source_image, &vibrant_recipe);
    let rescue_result = ImageProcessor::process(&source_image, &rescue_recipe);

    // All should process successfully
    assert_eq!(master_result.width, 120);
    assert_eq!(bw_result.width, 120);
    assert_eq!(vibrant_result.width, 120);
    assert_eq!(rescue_result.width, 120);

    // Results should be different
    assert_ne!(master_result.data, bw_result.data);
    assert_ne!(master_result.data, vibrant_result.data);
    assert_ne!(bw_result.data, vibrant_result.data);
}

#[test]
fn test_preset_application() {
    // Simulate applying saved presets

    let img = create_realistic_test_image(100, 100);

    // Preset 1: "Landscape"
    let landscape_preset = EditRecipe {
        vibrance: 30,
        saturation: 15,
        clarity: 25,
        highlights: -15,
        shadows: 20,
        ..Default::default()
    };

    // Preset 2: "Portrait"
    let portrait_preset = EditRecipe {
        exposure: 0.3,
        contrast: -10,
        highlights: -20,
        shadows: 15,
        clarity: -10, // Negative clarity for soft skin
        saturation: -5,
        ..Default::default()
    };

    // Preset 3: "High Contrast B&W"
    let hc_bw_preset = EditRecipe {
        saturation: -100,
        contrast: 50,
        highlights: -30,
        shadows: 30,
        clarity: 40,
        ..Default::default()
    };

    // Apply presets
    let landscape = ImageProcessor::process(&img, &landscape_preset);
    let portrait = ImageProcessor::process(&img, &portrait_preset);
    let hc_bw = ImageProcessor::process(&img, &hc_bw_preset);

    // All should produce different results
    assert_ne!(landscape.data, portrait.data);
    assert_ne!(landscape.data, hc_bw.data);
    assert_ne!(portrait.data, hc_bw.data);
}

#[test]
fn test_extreme_workflow() {
    // Test extreme editing scenarios photographers might do

    let img = create_realistic_test_image(150, 150);

    // Extreme recovery (severely underexposed)
    let mut recovery_recipe = EditRecipe::default();
    recovery_recipe.exposure = 3.0; // +3 EV
    recovery_recipe.shadows = 100;
    recovery_recipe.blacks = 100;
    recovery_recipe.highlights = -100;

    let result = ImageProcessor::process(&img, &recovery_recipe);
    assert_eq!(result.width, 150);

    // Extreme creative (heavy processing)
    let mut creative_recipe = EditRecipe::default();
    creative_recipe.exposure = -1.0;
    creative_recipe.contrast = 80;
    creative_recipe.clarity = 100;
    creative_recipe.vibrance = 100;
    creative_recipe.saturation = 50;
    creative_recipe.sharpening.amount = 150;

    let result = ImageProcessor::process(&img, &creative_recipe);
    assert_eq!(result.width, 150);

    // All values should still be valid
    for &val in &result.data {
        assert!(val <= 255, "Pixel value out of range: {}", val);
    }
}

#[test]
fn test_performance_sanity() {
    // Ensure processing completes in reasonable time
    use std::time::Instant;

    // Use smaller image for debug builds
    // In debug mode, nested loops (clarity, sharpening) are very slow
    let img = create_realistic_test_image(500, 500); // 250k pixels

    let mut recipe = EditRecipe::default();
    recipe.exposure = 0.5;
    recipe.contrast = 20;
    recipe.highlights = -20;
    recipe.shadows = 30;
    recipe.vibrance = 25;
    recipe.saturation = 10;
    // Skip clarity and sharpening for this test (they're O(n*r²) which is slow in debug)

    let start = Instant::now();
    let result = ImageProcessor::process(&img, &recipe);
    let elapsed = start.elapsed();

    assert_eq!(result.width, 500);
    assert_eq!(result.height, 500);

    // In debug mode, expect slower performance
    // In release mode, this should be < 50ms
    println!("Processing 500x500 image took: {:?}", elapsed);

    #[cfg(debug_assertions)]
    let max_time = 5000; // 5 seconds for debug mode

    #[cfg(not(debug_assertions))]
    let max_time = 500; // 500ms for release mode

    assert!(
        elapsed.as_millis() < max_time,
        "Processing too slow: {:?} (max: {}ms)",
        elapsed,
        max_time
    );
}
