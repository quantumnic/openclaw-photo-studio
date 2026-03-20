//! ocps-export — Export engine
//! JPEG, TIFF, PNG, WebP, AVIF, HEIF, DNG

pub mod jpeg;
pub mod png;
pub mod resize;
pub mod dng;
pub mod naming;
pub mod color;
pub mod watermark;
pub mod contact_sheet;
pub mod queue;

pub use dng::export_dng;
pub use naming::{apply_naming_template, PhotoForNaming};
pub use color::{OutputColorSpace, embed_icc_profile, convert_linear_to_output, apply_soft_proof};
pub use watermark::{TextWatermark, WatermarkPosition, apply_text_watermark};
pub use contact_sheet::{ContactSheetSettings, generate_contact_sheet};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ExportFormat { Jpeg, Png, Tiff8, Tiff16, WebP, Avif, Dng }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportSettings {
    pub format: ExportFormat,
    pub quality: u32,
    pub resize_long_edge: Option<u32>,
    pub include_metadata: bool,
    pub color_space: String,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            format: ExportFormat::Jpeg,
            quality: 90,
            resize_long_edge: None,
            include_metadata: true,
            color_space: "sRGB".to_string(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("Unsupported format")]
    UnsupportedFormat,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn export(
    _image_data: &[u8],
    _settings: &ExportSettings,
    _output_path: &std::path::Path,
) -> Result<(), ExportError> {
    // TODO: Phase 3 — implement export
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default_settings() {
        let s = ExportSettings::default();
        assert_eq!(s.quality, 90);
    }
}
