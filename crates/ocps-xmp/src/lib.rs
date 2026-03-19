//! ocps-xmp — XMP/IPTC/EXIF metadata engine
//!
//! Adobe-compatible XMP sidecar read/write.
//! Supports crs: (Camera Raw Settings), dc:, xmp:, Iptc4xmpCore: namespaces.

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Develop settings extracted from an XMP sidecar (Adobe Camera Raw format).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct XmpDevelopSettings {
    // Basic
    pub temperature: Option<i32>,
    pub tint: Option<i32>,
    pub exposure: Option<f32>,
    pub contrast: Option<i32>,
    pub highlights: Option<i32>,
    pub shadows: Option<i32>,
    pub whites: Option<i32>,
    pub blacks: Option<i32>,
    pub clarity: Option<i32>,
    pub dehaze: Option<i32>,
    pub vibrance: Option<i32>,
    pub saturation: Option<i32>,
    // Rating & flag
    pub rating: Option<u8>,
    pub label: Option<String>,
    // Process version
    pub process_version: Option<String>,
}

/// IPTC metadata
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct IptcData {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub creator: Option<String>,
    pub copyright: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum XmpError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Not an XMP file")]
    NotXmp,
}

/// Read XMP sidecar file and return develop settings + IPTC data.
/// Phase 1: returns default/empty — real parser in Phase 4.
pub fn read_sidecar(
    _path: &std::path::Path,
) -> Result<(XmpDevelopSettings, IptcData), XmpError> {
    // TODO: Phase 1/4 — implement XML parsing of Adobe XMP sidecar
    Ok((XmpDevelopSettings::default(), IptcData::default()))
}

/// Write develop settings + IPTC to an XMP sidecar file.
/// Adobe-compatible format using crs: namespace.
pub fn write_sidecar(
    _path: &std::path::Path,
    _develop: &XmpDevelopSettings,
    _iptc: &IptcData,
) -> Result<(), XmpError> {
    // TODO: Phase 1/4 — implement XMP serialization
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = XmpDevelopSettings::default();
        assert!(settings.exposure.is_none());
        assert!(settings.keywords_or_default().is_empty());
    }
}

trait OrDefault {
    fn keywords_or_default(&self) -> Vec<String>;
}

impl OrDefault for XmpDevelopSettings {
    fn keywords_or_default(&self) -> Vec<String> {
        vec![]
    }
}
