//! Example: Process an image with the CPU pipeline
//!
//! This example demonstrates how to use the ImageProcessor to apply
//! various adjustments to a RAW image.

use ocps_core::{
    decode, demosaic, DemosaicAlgorithm, EditRecipe, ImageProcessor, RgbImage16,
};
use std::path::PathBuf;

fn main() {
    println!("OpenClaw Photo Studio - Image Processing Example\n");

    // Check for command line argument
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo run --example process_image <path-to-raw-file>");
        eprintln!("\nThis example will:");
        eprintln!("  1. Decode the RAW file");
        eprintln!("  2. Demosaic to RGB");
        eprintln!("  3. Apply various edit adjustments");
        eprintln!("  4. Process to 8-bit sRGB");
        eprintln!("\nNote: This example doesn't save the output, it just processes the image.");
        std::process::exit(1);
    }

    let raw_path = PathBuf::from(&args[1]);

    println!("Processing: {}", raw_path.display());

    // Step 1: Decode RAW file
    println!("\n[1/4] Decoding RAW file...");
    let raw_image = match decode(&raw_path) {
        Ok(img) => {
            println!("  ✓ Decoded: {}x{} pixels", img.width, img.height);
            println!("  Camera: {} {}",
                img.camera_make.as_deref().unwrap_or("Unknown"),
                img.camera_model.as_deref().unwrap_or("Unknown"));
            img
        }
        Err(e) => {
            eprintln!("  ✗ Failed to decode RAW file: {}", e);
            std::process::exit(1);
        }
    };

    // Step 2: Demosaic to RGB
    println!("\n[2/4] Demosaicing (Bayer → RGB)...");
    let rgb_image = demosaic(&raw_image, DemosaicAlgorithm::Bilinear);
    println!("  ✓ Demosaiced: {}x{} RGB image", rgb_image.width, rgb_image.height);

    // Convert to our pipeline format (u8 → u16)
    // The demosaic output is u8, but our pipeline works with u16 for better precision
    let data_u16: Vec<u16> = rgb_image
        .data
        .iter()
        .map(|&val| (val as u16) * 257) // Scale u8 to u16 (0-255 → 0-65535)
        .collect();

    let pipeline_image = RgbImage16::from_data(
        rgb_image.width,
        rgb_image.height,
        data_u16,
    );

    // Step 3: Create edit recipe with various adjustments
    println!("\n[3/4] Creating edit recipe...");
    let mut recipe = EditRecipe::default();

    // Apply some example edits
    recipe.exposure = 0.5; // +0.5 EV
    recipe.contrast = 15;
    recipe.highlights = -20;
    recipe.shadows = 30;
    recipe.whites = -10;
    recipe.blacks = 10;
    recipe.clarity = 20;
    recipe.vibrance = 25;
    recipe.saturation = 10;
    recipe.white_balance.temperature = 6000;
    recipe.white_balance.tint = 5;

    println!("  Recipe:");
    println!("    Exposure: {:+.1} EV", recipe.exposure);
    println!("    Contrast: {:+}", recipe.contrast);
    println!("    Highlights: {:+}", recipe.highlights);
    println!("    Shadows: {:+}", recipe.shadows);
    println!("    Whites: {:+}", recipe.whites);
    println!("    Blacks: {:+}", recipe.blacks);
    println!("    Clarity: {:+}", recipe.clarity);
    println!("    Vibrance: {:+}", recipe.vibrance);
    println!("    Saturation: {:+}", recipe.saturation);
    println!("    White Balance: {}K, tint {:+}",
        recipe.white_balance.temperature,
        recipe.white_balance.tint);

    // Step 4: Process with ImageProcessor
    println!("\n[4/4] Processing image through pipeline...");
    let processed = ImageProcessor::process(&pipeline_image, &recipe);

    println!("  ✓ Processed to 8-bit sRGB: {}x{} ({}KB)",
        processed.width,
        processed.height,
        processed.data.len() / 1024);

    // Calculate some statistics
    let mut min = 255u8;
    let mut max = 0u8;
    let mut sum = 0u64;

    for &val in &processed.data {
        min = min.min(val);
        max = max.max(val);
        sum += val as u64;
    }

    let avg = sum / processed.data.len() as u64;

    println!("\n  Statistics:");
    println!("    Value range: {} - {}", min, max);
    println!("    Average: {}", avg);
    println!("    Dynamic range: {}", max - min);

    println!("\n✓ Processing complete!");
    println!("\nTo save the output, you would typically:");
    println!("  - Convert RgbImage8 to PNG/JPEG using the image crate");
    println!("  - Or pass it to the GPU for display");
    println!("  - Or export it using ocps-export crate");
}
