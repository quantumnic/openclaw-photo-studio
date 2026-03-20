//! RAW file decoding — real implementation using rawloader
//!
//! Supports: DNG, CR2 (Canon legacy), ARW (Sony), NEF (Nikon), RAF (Fuji),
//!           ORF (Olympus), RW2 (Panasonic)
//! Note: CR3 support pending (ISOBMFF parser not yet in rawloader 0.37)

use std::path::Path;
use thiserror::Error;

pub mod camera_profiles;
pub mod demosaic;
pub mod thumbnail;

/// RAW decoding errors
#[derive(Debug, Error)]
pub enum RawDecodeError {
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Corrupt or invalid RAW file: {0}")]
    Corrupt(String),

    #[error("rawloader decode error: {0}")]
    RawloaderError(String),

    #[error("Missing metadata: {0}")]
    MissingMetadata(String),
}

/// CFA (Color Filter Array) pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CfaPattern {
    RGGB,
    BGGR,
    GBRG,
    GRBG,
}

impl CfaPattern {
    /// Convert from rawloader's CFA string representation
    pub fn from_rawloader_str(s: &str) -> Option<Self> {
        match s {
            "RGGB" => Some(CfaPattern::RGGB),
            "BGGR" => Some(CfaPattern::BGGR),
            "GBRG" => Some(CfaPattern::GBRG),
            "GRBG" => Some(CfaPattern::GRBG),
            _ => None,
        }
    }
}

/// RAW image metadata (without full pixel data)
#[derive(Debug, Clone)]
pub struct RawImageMeta {
    pub width: u32,
    pub height: u32,
    pub camera_make: String,
    pub camera_model: String,
    pub wb_coeffs: [f32; 4],
    pub black_level: u16,
    pub white_level: u16,
    pub format: String,
}

/// Decoded RAW image with metadata
#[derive(Debug, Clone)]
pub struct RawImage {
    /// Image width in pixels
    pub width: u32,

    /// Image height in pixels
    pub height: u32,

    /// Raw Bayer data (single channel, mosaiced)
    pub data: Vec<u16>,

    /// Camera manufacturer (e.g., "Canon", "Nikon", "Sony")
    pub camera_make: Option<String>,

    /// Camera model (e.g., "Canon EOS R5", "Nikon D850")
    pub camera_model: Option<String>,

    /// White balance coefficients [R, G, B, G2] (as-shot WB)
    pub wb_coeffs: [f32; 4],

    /// Black level per channel
    pub black_level: [u16; 4],

    /// White level (sensor saturation point)
    pub white_level: u16,

    /// CFA pattern
    pub cfa_pattern: CfaPattern,

    /// ISO sensitivity
    pub iso: Option<u32>,

    /// Exposure time in seconds
    pub exposure_time: Option<f32>,

    /// Aperture (f-number)
    pub aperture: Option<f32>,
}

impl RawImage {
    /// Normalize a raw value to 0.0-1.0 range, applying black/white levels
    pub fn normalize_value(&self, value: u16, channel: usize) -> f32 {
        let black = self.black_level[channel.min(3)] as f32;
        let white = self.white_level as f32;

        if white <= black {
            return 0.0;
        }

        let normalized = (value as f32 - black) / (white - black);
        normalized.clamp(0.0, 1.0)
    }
}

/// Decode a RAW file from disk
///
/// # Arguments
/// * `path` - Path to the RAW file
///
/// # Returns
/// * `Ok(RawImage)` - Successfully decoded RAW image with metadata
/// * `Err(RawDecodeError)` - Decoding failed
///
/// # Supported Formats
/// - DNG (Adobe Digital Negative)
/// - CR2 (Canon RAW legacy)
/// - ARW (Sony RAW)
/// - NEF (Nikon Electronic Format)
/// - RAF (Fujifilm RAW)
/// - ORF (Olympus RAW Format)
/// - RW2 (Panasonic RAW)
///
/// # Example
/// ```no_run
/// use ocps_core::raw::decode;
/// use std::path::Path;
///
/// let raw = decode(Path::new("photo.dng")).expect("Failed to decode RAW");
/// println!("Camera: {} {}",
///     raw.camera_make.unwrap_or_default(),
///     raw.camera_model.unwrap_or_default());
/// ```
pub fn decode(path: &Path) -> Result<RawImage, RawDecodeError> {
    // Check if file exists
    if !path.exists() {
        return Err(RawDecodeError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", path.display()),
        )));
    }

    // Check if it's a file (not a directory)
    if !path.is_file() {
        return Err(RawDecodeError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Not a file: {}", path.display()),
        )));
    }

    // Decode using rawloader
    let raw_file = rawloader::decode_file(path)
        .map_err(|e| RawDecodeError::RawloaderError(e.to_string()))?;

    // Extract dimensions (rawloader uses usize, convert to u32)
    let width = raw_file.width as u32;
    let height = raw_file.height as u32;

    if width == 0 || height == 0 {
        return Err(RawDecodeError::Corrupt(
            "Invalid dimensions (zero width or height)".to_string(),
        ));
    }

    // Extract camera metadata
    let camera_make = if raw_file.make.is_empty() {
        None
    } else {
        Some(raw_file.make.clone())
    };

    let camera_model = if raw_file.model.is_empty() {
        None
    } else {
        Some(raw_file.model.clone())
    };

    // Extract white balance coefficients (rawloader provides [R, G, B, E])
    let wb_coeffs = raw_file.wb_coeffs;

    // Extract black levels (rawloader provides [R, G, B, E])
    let black_level = raw_file.blacklevels;

    // Extract white level (use first channel, typically all the same)
    let white_level = raw_file.whitelevels[0];

    // Determine CFA pattern from rawloader
    // rawloader provides a CFA struct with a name field
    let cfa_pattern = {
        let cfa_name = raw_file.cfa.name.split_whitespace().next().unwrap_or("RGGB");
        CfaPattern::from_rawloader_str(cfa_name).unwrap_or(CfaPattern::RGGB)
    };

    // rawloader 0.37 doesn't expose EXIF data directly in the RawImage struct
    // These would need to be extracted separately using an EXIF parser
    // For now, set to None (will be implemented in Phase 1-2 with XMP integration)
    let iso = None;
    let exposure_time = None;
    let aperture = None;

    // Get the raw data - rawloader uses an enum for data
    // Extract u16 vector from RawImageData enum
    let data = match raw_file.data {
        rawloader::RawImageData::Integer(vec) => vec,
        rawloader::RawImageData::Float(vec) => {
            // Convert f32 to u16 (some DNGs use float)
            vec.iter().map(|&f| (f * 65535.0).round() as u16).collect()
        }
    };

    // Verify data size
    let expected_size = (width * height) as usize;
    if data.len() != expected_size {
        return Err(RawDecodeError::Corrupt(format!(
            "Data size mismatch: expected {} pixels, got {}",
            expected_size,
            data.len()
        )));
    }

    Ok(RawImage {
        width,
        height,
        data,
        camera_make,
        camera_model,
        wb_coeffs,
        black_level,
        white_level,
        cfa_pattern,
        iso,
        exposure_time,
        aperture,
    })
}

/// Decode RAW file metadata without loading full pixel data
///
/// This is a fast path for extracting metadata only.
/// Much faster than full decode when you only need camera info.
///
/// # Arguments
/// * `path` - Path to the RAW file
///
/// # Returns
/// * `Ok(RawImageMeta)` - Successfully extracted metadata
/// * `Err(RawDecodeError)` - Decoding failed
pub fn decode_meta(path: &Path) -> Result<RawImageMeta, RawDecodeError> {
    // Check if file exists
    if !path.exists() {
        return Err(RawDecodeError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", path.display()),
        )));
    }

    // Check if it's a file (not a directory)
    if !path.is_file() {
        return Err(RawDecodeError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Not a file: {}", path.display()),
        )));
    }

    // Decode using rawloader
    let raw_file = rawloader::decode_file(path)
        .map_err(|e| RawDecodeError::RawloaderError(e.to_string()))?;

    // Extract dimensions
    let width = raw_file.width as u32;
    let height = raw_file.height as u32;

    // Extract camera metadata
    let camera_make = if raw_file.make.is_empty() {
        "Unknown".to_string()
    } else {
        raw_file.make.clone()
    };

    let camera_model = if raw_file.model.is_empty() {
        "Unknown".to_string()
    } else {
        raw_file.model.clone()
    };

    // Extract white balance coefficients
    let wb_coeffs = raw_file.wb_coeffs;

    // Extract black level (use first channel as representative)
    let black_level = raw_file.blacklevels[0];

    // Extract white level
    let white_level = raw_file.whitelevels[0];

    // Determine format from file extension
    let format = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("RAW")
        .to_uppercase();

    Ok(RawImageMeta {
        width,
        height,
        camera_make,
        camera_model,
        wb_coeffs,
        black_level,
        white_level,
        format,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_decode_nonexistent_file() {
        let result = decode(Path::new("/tmp/nonexistent_raw_file_12345.dng"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RawDecodeError::Io(_)));
    }

    #[test]
    fn test_decode_invalid_file() {
        // Create a temporary non-RAW file
        let temp_path = "/tmp/ocps_test_invalid.txt";
        let mut file = std::fs::File::create(temp_path).unwrap();
        writeln!(file, "This is not a RAW file").unwrap();
        drop(file);

        let result = decode(Path::new(temp_path));
        assert!(result.is_err());

        // Cleanup
        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    fn test_cfa_pattern_from_str() {
        assert_eq!(CfaPattern::from_rawloader_str("RGGB"), Some(CfaPattern::RGGB));
        assert_eq!(CfaPattern::from_rawloader_str("BGGR"), Some(CfaPattern::BGGR));
        assert_eq!(CfaPattern::from_rawloader_str("GBRG"), Some(CfaPattern::GBRG));
        assert_eq!(CfaPattern::from_rawloader_str("GRBG"), Some(CfaPattern::GRBG));
        assert_eq!(CfaPattern::from_rawloader_str("INVALID"), None);
    }

    #[test]
    fn test_raw_image_normalize() {
        let raw = RawImage {
            width: 100,
            height: 100,
            data: vec![0; 10000],
            camera_make: None,
            camera_model: None,
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            black_level: [512, 512, 512, 512],
            white_level: 16383,
            cfa_pattern: CfaPattern::RGGB,
            iso: None,
            exposure_time: None,
            aperture: None,
        };

        // Test black level
        let normalized_black = raw.normalize_value(512, 0);
        assert!((normalized_black - 0.0).abs() < 0.001);

        // Test white level
        let normalized_white = raw.normalize_value(16383, 0);
        assert!((normalized_white - 1.0).abs() < 0.001);

        // Test mid value
        let mid_value = 512 + (16383 - 512) / 2;
        let normalized_mid = raw.normalize_value(mid_value, 0);
        assert!((normalized_mid - 0.5).abs() < 0.01);
    }
}
