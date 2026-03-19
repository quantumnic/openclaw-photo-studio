//! CLI tool to test RAW decoding functionality
//!
//! Usage: ocps-decode-test <path-to-raw-file>
//!
//! Decodes a RAW file and prints metadata, extracts thumbnail,
//! and reports timing information.

use ocps_core::raw::{decode, thumbnail};
use std::env;
use std::path::Path;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <path-to-raw-file>", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!("  {} photo.dng", args[0]);
        eprintln!("  {} image.arw", args[0]);
        std::process::exit(1);
    }

    let raw_path = &args[1];
    let path = Path::new(raw_path);

    println!("OpenClaw Photo Studio — RAW Decode Test");
    println!("========================================");
    println!();
    println!("Input file: {}", path.display());
    println!();

    // Check if file exists
    if !path.exists() {
        eprintln!("ERROR: File not found: {}", path.display());
        std::process::exit(1);
    }

    // Decode RAW file
    println!("Decoding RAW file...");
    let start = Instant::now();
    let raw = match decode(path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("ERROR: Failed to decode RAW file: {}", e);
            std::process::exit(1);
        }
    };
    let decode_time = start.elapsed();
    println!("✓ Decode complete in {:.2}ms", decode_time.as_secs_f64() * 1000.0);
    println!();

    // Print metadata
    println!("RAW Metadata:");
    println!("  Camera Make:      {}", raw.camera_make.as_deref().unwrap_or("Unknown"));
    println!("  Camera Model:     {}", raw.camera_model.as_deref().unwrap_or("Unknown"));
    println!("  Dimensions:       {} x {} pixels", raw.width, raw.height);
    println!("  Megapixels:       {:.1} MP", (raw.width * raw.height) as f64 / 1_000_000.0);
    println!("  CFA Pattern:      {:?}", raw.cfa_pattern);
    println!("  White Balance:    R={:.3} G={:.3} B={:.3} G2={:.3}",
             raw.wb_coeffs[0], raw.wb_coeffs[1], raw.wb_coeffs[2], raw.wb_coeffs[3]);
    println!("  Black Levels:     [{}, {}, {}, {}]",
             raw.black_level[0], raw.black_level[1], raw.black_level[2], raw.black_level[3]);
    println!("  White Level:      {}", raw.white_level);

    if let Some(iso) = raw.iso {
        println!("  ISO:              {}", iso);
    }

    if let Some(exp_time) = raw.exposure_time {
        if exp_time >= 1.0 {
            println!("  Exposure Time:    {:.1}s", exp_time);
        } else {
            println!("  Exposure Time:    1/{:.0}s", 1.0 / exp_time);
        }
    }

    if let Some(aperture) = raw.aperture {
        println!("  Aperture:         f/{:.1}", aperture);
    }

    println!();

    // Extract thumbnail
    println!("Extracting thumbnail (256px)...");
    let start = Instant::now();
    let thumbnail_path = "/tmp/ocps-thumb.jpg";
    match thumbnail::extract_thumbnail(path, 256) {
        Ok(jpeg_bytes) => {
            let thumb_time = start.elapsed();
            println!("✓ Thumbnail extracted in {:.2}ms", thumb_time.as_secs_f64() * 1000.0);
            println!("  Size: {} bytes ({:.1} KB)", jpeg_bytes.len(), jpeg_bytes.len() as f64 / 1024.0);

            // Save to file
            match std::fs::write(thumbnail_path, &jpeg_bytes) {
                Ok(_) => {
                    println!("  Saved to: {}", thumbnail_path);
                }
                Err(e) => {
                    eprintln!("  Warning: Failed to save thumbnail: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("ERROR: Failed to extract thumbnail: {}", e);
        }
    }

    println!();
    println!("========================================");
    println!("Total time: {:.2}ms", (decode_time.as_secs_f64()) * 1000.0);
    println!();
    println!("Test complete! ✓");
}
