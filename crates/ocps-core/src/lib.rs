//! ocps-core — OpenClaw Photo Studio Core Engine
//!
//! RAW decode, GPU pipeline, image processing.
//! This is the heart of the application.

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub mod raw {
    //! RAW file decoding (CR3, ARW, NEF, RAF, DNG, ORF, RW2)
    //! Phase 1: Placeholder — real implementation in Phase 1-2

    #[derive(Debug, Clone)]
    pub struct RawImage {
        pub width: u32,
        pub height: u32,
        pub bayer_data: Vec<u16>,
        pub camera_make: Option<String>,
        pub camera_model: Option<String>,
    }

    #[derive(Debug, thiserror::Error)]
    pub enum RawDecodeError {
        #[error("Unsupported format: {0}")]
        UnsupportedFormat(String),
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),
        #[error("Corrupt file: {0}")]
        Corrupt(String),
    }

    /// Decode a RAW file from disk.
    /// Returns a RawImage with demosaiced pixel data.
    pub fn decode(_path: &std::path::Path) -> Result<RawImage, RawDecodeError> {
        // TODO: Phase 1 — implement rawloader integration
        Err(RawDecodeError::UnsupportedFormat(
            "RAW decode not yet implemented — coming in Phase 1".to_string(),
        ))
    }
}

pub mod pipeline {
    //! GPU-accelerated processing pipeline (wgpu)
    //! Phase 1: Placeholder

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct PipelineSettings {
        pub exposure: f32,
        pub contrast: i32,
        pub highlights: i32,
        pub shadows: i32,
        pub whites: i32,
        pub blacks: i32,
        pub clarity: i32,
        pub vibrance: i32,
        pub saturation: i32,
    }

    impl Default for PipelineSettings {
        fn default() -> Self {
            Self {
                exposure: 0.0,
                contrast: 0,
                highlights: 0,
                shadows: 0,
                whites: 0,
                blacks: 0,
                clarity: 0,
                vibrance: 0,
                saturation: 0,
            }
        }
    }
}

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

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[test]
    fn test_pipeline_defaults() {
        let settings = pipeline::PipelineSettings::default();
        assert_eq!(settings.exposure, 0.0);
        assert_eq!(settings.contrast, 0);
    }
}
