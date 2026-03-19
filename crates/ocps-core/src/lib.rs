//! ocps-core — OpenClaw Photo Studio Core Engine
//!
//! RAW decode, GPU pipeline, image processing.
//! This is the heart of the application.

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub mod raw;

pub use raw::{
    decode, decode_meta, CfaPattern, RawDecodeError, RawImage, RawImageMeta,
    demosaic::{demosaic, DemosaicAlgorithm, RgbImage},
    thumbnail::{extract_thumbnail, quick_thumbnail, ThumbnailError},
};

pub mod pipeline;

pub use pipeline::{
    ColorGradingSettings, CropSettings, EditRecipe, ImageProcessor, NoiseReductionSettings,
    RgbImage16, RgbImage8, SharpeningSettings, WhiteBalance,
};

pub mod cache {
    //! Preview and thumbnail cache management

    pub struct CacheConfig {
        pub ram_limit_mb: u64,
        pub disk_limit_mb: u64,
        pub preview_size: u32,
        pub thumbnail_size: u32,
    }

    impl Default for CacheConfig {
        fn default() -> Self {
            Self {
                ram_limit_mb: 2048,
                disk_limit_mb: 10240,
                preview_size: 2048,
                thumbnail_size: 256,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::Path;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[test]
    fn test_pipeline_defaults() {
        let recipe = pipeline::EditRecipe::default();
        assert_eq!(recipe.exposure, 0.0);
        assert_eq!(recipe.contrast, 0);
        assert!(recipe.is_identity());
    }

    #[test]
    fn test_decode_missing_file() {
        let result = raw::decode(Path::new("/tmp/nonexistent_ocps_test.dng"));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            raw::RawDecodeError::Io(_)
        ));
    }

    #[test]
    fn test_decode_invalid_file() {
        // Create a temp file with non-RAW content
        let temp_path = "/tmp/ocps_test_not_raw.txt";
        {
            let mut file = std::fs::File::create(temp_path).unwrap();
            writeln!(file, "This is not a RAW file, just plain text").unwrap();
        }

        let result = raw::decode(Path::new(temp_path));
        assert!(result.is_err());

        // Cleanup
        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    fn test_demosaic_synthetic_bayer() {
        // Create a small synthetic Bayer array
        let test_raw = raw::RawImage {
            width: 4,
            height: 4,
            data: vec![
                100, 200, 150, 250,
                300, 400, 350, 450,
                500, 600, 550, 650,
                700, 800, 750, 850,
            ],
            camera_make: Some("Test".to_string()),
            camera_model: Some("Synthetic".to_string()),
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            black_level: [0, 0, 0, 0],
            white_level: 1000,
            cfa_pattern: raw::CfaPattern::RGGB,
            iso: None,
            exposure_time: None,
            aperture: None,
        };

        // Test bilinear demosaic
        let rgb = raw::demosaic::demosaic(&test_raw, raw::demosaic::DemosaicAlgorithm::Bilinear);
        assert_eq!(rgb.width, 4);
        assert_eq!(rgb.height, 4);
        assert_eq!(rgb.data.len(), 4 * 4 * 3); // 4x4 pixels, 3 channels each

        // Verify all RGB values are within valid range
        for &value in &rgb.data {
            assert!(value <= 255);
        }
    }

    #[test]
    fn test_thumbnail_extraction_fallback() {
        // This test verifies that thumbnail extraction fails gracefully
        // on non-existent files
        let result = raw::thumbnail::extract_thumbnail(
            Path::new("/tmp/nonexistent_test_raw.dng"),
            256,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_cfa_patterns() {
        use raw::CfaPattern;

        // Test all CFA pattern variants exist
        let patterns = [
            CfaPattern::RGGB,
            CfaPattern::BGGR,
            CfaPattern::GBRG,
            CfaPattern::GRBG,
        ];

        for pattern in patterns {
            // Just verify they're valid enum values
            let _debug_str = format!("{:?}", pattern);
        }
    }
}
