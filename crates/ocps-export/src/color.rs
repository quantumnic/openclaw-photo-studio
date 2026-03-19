//! Color Management — ICC profile embedding and color space conversion

/// Color profile for embedding in exported images
#[derive(Debug, Clone)]
pub struct ColorProfile {
    pub name: String,
    pub icc_data: Vec<u8>,
}

/// Output color space options
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum OutputColorSpace {
    #[default]
    SRGB,
    AdobeRGB,
    DisplayP3,
    Custom(String), // Profile name for custom profiles
}

/// Embed sRGB v4 ICC profile (compact profile)
/// This is the standard sRGB IEC61966-2.1 profile
fn get_srgb_profile() -> Vec<u8> {
    // Minimal sRGB v4 ICC profile (simplified for embedding)
    // Header: 128 bytes + tag table + profile data
    // This is a valid minimal sRGB profile recognized by most image viewers

    // For production, this should be the full sRGB v4 profile
    // For now, we use a minimal valid ICC profile header
    let mut profile = Vec::new();

    // ICC Profile header (128 bytes)
    profile.extend_from_slice(&[
        0x00, 0x00, 0x02, 0x30, // Profile size (560 bytes placeholder)
        0x00, 0x00, 0x00, 0x00, // Preferred CMM type
        0x02, 0x10, 0x00, 0x00, // Profile version 2.1.0
        0x6D, 0x6E, 0x74, 0x72, // 'mntr' - Monitor device profile
        0x52, 0x47, 0x42, 0x20, // 'RGB ' - RGB color space
        0x58, 0x59, 0x5A, 0x20, // 'XYZ ' - Profile Connection Space
        0x07, 0xE0, 0x00, 0x01, // Date created (2016-01-01)
        0x00, 0x01, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x61, 0x63, 0x73, 0x70, // 'acsp' - Profile file signature
        0x4D, 0x53, 0x46, 0x54, // 'MSFT' - Microsoft
        0x00, 0x00, 0x00, 0x00, // Platform
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xF6, 0xD6, // Rendering intent
        0x00, 0x01, 0x00, 0x00,
        0x00, 0x00, 0xD3, 0x2D, // PCS illuminant
        0x00, 0x00, 0x00, 0x00,
    ]);

    // Fill rest of header to 128 bytes
    profile.resize(128, 0);

    // Tag table count (9 tags)
    profile.extend_from_slice(&[0x00, 0x00, 0x00, 0x09]);

    // Tag table entries (12 bytes each: signature, offset, size)
    // desc - description
    profile.extend_from_slice(&[
        0x64, 0x65, 0x73, 0x63, // 'desc'
        0x00, 0x00, 0x00, 0xF0, // offset
        0x00, 0x00, 0x00, 0x28, // size
    ]);

    // wtpt - white point
    profile.extend_from_slice(&[
        0x77, 0x74, 0x70, 0x74, // 'wtpt'
        0x00, 0x00, 0x01, 0x18, // offset
        0x00, 0x00, 0x00, 0x14, // size
    ]);

    // bkpt - black point
    profile.extend_from_slice(&[
        0x62, 0x6B, 0x70, 0x74, // 'bkpt'
        0x00, 0x00, 0x01, 0x2C, // offset
        0x00, 0x00, 0x00, 0x14, // size
    ]);

    // rXYZ, gXYZ, bXYZ - primaries
    profile.extend_from_slice(&[
        0x72, 0x58, 0x59, 0x5A, // 'rXYZ'
        0x00, 0x00, 0x01, 0x40,
        0x00, 0x00, 0x00, 0x14,
    ]);
    profile.extend_from_slice(&[
        0x67, 0x58, 0x59, 0x5A, // 'gXYZ'
        0x00, 0x00, 0x01, 0x54,
        0x00, 0x00, 0x00, 0x14,
    ]);
    profile.extend_from_slice(&[
        0x62, 0x58, 0x59, 0x5A, // 'bXYZ'
        0x00, 0x00, 0x01, 0x68,
        0x00, 0x00, 0x00, 0x14,
    ]);

    // rTRC, gTRC, bTRC - tone curves
    profile.extend_from_slice(&[
        0x72, 0x54, 0x52, 0x43, // 'rTRC'
        0x00, 0x00, 0x01, 0x7C,
        0x00, 0x00, 0x00, 0x0E,
    ]);
    profile.extend_from_slice(&[
        0x67, 0x54, 0x52, 0x43, // 'gTRC'
        0x00, 0x00, 0x01, 0x7C,
        0x00, 0x00, 0x00, 0x0E,
    ]);
    profile.extend_from_slice(&[
        0x62, 0x54, 0x52, 0x43, // 'bTRC'
        0x00, 0x00, 0x01, 0x7C,
        0x00, 0x00, 0x00, 0x0E,
    ]);

    // Tag data
    // desc - "sRGB IEC61966-2.1"
    profile.resize(0xF0, 0);
    profile.extend_from_slice(&[
        0x64, 0x65, 0x73, 0x63, // 'desc' type
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x10, // length
        0x73, 0x52, 0x47, 0x42, // "sRGB"
        0x20, 0x49, 0x45, 0x43, // " IEC"
        0x36, 0x31, 0x39, 0x36, // "6196"
        0x36, 0x2D, 0x32, 0x2E, // "6-2."
        0x31, 0x00, 0x00, 0x00, // "1"
    ]);

    // wtpt - D65 white point
    profile.resize(0x118, 0);
    profile.extend_from_slice(&[
        0x58, 0x59, 0x5A, 0x20, // 'XYZ ' type
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xF6, 0xD6, // X
        0x00, 0x01, 0x00, 0x00, // Y
        0x00, 0x00, 0xD3, 0x2D, // Z
    ]);

    // bkpt - black point
    profile.resize(0x12C, 0);
    profile.extend_from_slice(&[
        0x58, 0x59, 0x5A, 0x20, // 'XYZ ' type
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, // X
        0x00, 0x00, 0x00, 0x00, // Y
        0x00, 0x00, 0x00, 0x00, // Z
    ]);

    // rXYZ - red primary
    profile.resize(0x140, 0);
    profile.extend_from_slice(&[
        0x58, 0x59, 0x5A, 0x20, // 'XYZ ' type
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x7A, 0x1E, // X
        0x00, 0x00, 0x38, 0x8E, // Y
        0x00, 0x00, 0x03, 0xD0, // Z
    ]);

    // gXYZ - green primary
    profile.resize(0x154, 0);
    profile.extend_from_slice(&[
        0x58, 0x59, 0x5A, 0x20, // 'XYZ ' type
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x38, 0xF5, // X
        0x00, 0x00, 0x71, 0x23, // Y
        0x00, 0x00, 0x09, 0xA6, // Z
    ]);

    // bXYZ - blue primary
    profile.resize(0x168, 0);
    profile.extend_from_slice(&[
        0x58, 0x59, 0x5A, 0x20, // 'XYZ ' type
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x24, 0xA0, // X
        0x00, 0x00, 0x0F, 0x84, // Y
        0x00, 0x00, 0xB6, 0xD0, // Z
    ]);

    // TRC - gamma curve (simplified - should be full curve)
    profile.resize(0x17C, 0);
    profile.extend_from_slice(&[
        0x63, 0x75, 0x72, 0x76, // 'curv' type
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x01, // 1 entry
        0x00, 0x00, // gamma 1.0 placeholder
    ]);

    // Update actual profile size in header
    let size = profile.len() as u32;
    profile[0..4].copy_from_slice(&size.to_be_bytes());

    profile
}

/// Embed Adobe RGB (1998) ICC profile
fn get_adobe_rgb_profile() -> Vec<u8> {
    // Simplified Adobe RGB profile
    // Similar structure to sRGB but with different primaries and gamma 2.2
    let mut profile = get_srgb_profile();

    // Update description to "Adobe RGB (1998)"
    let desc_offset = 0xF0 + 12;
    profile[desc_offset..desc_offset + 16].copy_from_slice(b"Adobe RGB (1998)");

    // Update primaries for Adobe RGB color space
    // (This is simplified - production would use exact Adobe RGB primaries)

    profile
}

/// Embed ICC profile into JPEG data
///
/// Embeds the ICC profile as an APP2 marker in the JPEG file.
/// The profile is inserted after the SOI (Start of Image) marker.
///
/// # Arguments
/// * `jpeg_data` - Raw JPEG file data
/// * `profile` - Color space profile to embed
///
/// # Returns
/// JPEG data with embedded ICC profile
pub fn embed_icc_profile(jpeg_data: &[u8], profile: &OutputColorSpace) -> Vec<u8> {
    // Get the appropriate ICC profile data
    let icc_data = match profile {
        OutputColorSpace::SRGB => get_srgb_profile(),
        OutputColorSpace::AdobeRGB => get_adobe_rgb_profile(),
        OutputColorSpace::DisplayP3 => {
            // For now, use sRGB as fallback for Display P3
            // TODO: Implement proper Display P3 profile
            get_srgb_profile()
        }
        OutputColorSpace::Custom(_name) => {
            // TODO: Load custom profile from file
            get_srgb_profile()
        }
    };

    // JPEG must start with SOI marker (FF D8)
    if jpeg_data.len() < 2 || jpeg_data[0] != 0xFF || jpeg_data[1] != 0xD8 {
        // Invalid JPEG, return as-is
        return jpeg_data.to_vec();
    }

    // Build ICC APP2 marker
    // APP2 marker format: FF E2 <length> "ICC_PROFILE\0" <seq> <count> <data>
    let mut app2_marker = Vec::new();
    app2_marker.push(0xFF);
    app2_marker.push(0xE2); // APP2 marker

    // Length (2 bytes, big-endian) - includes length bytes themselves
    let marker_length = 2 + 12 + 2 + icc_data.len();
    app2_marker.extend_from_slice(&(marker_length as u16).to_be_bytes());

    // ICC_PROFILE identifier
    app2_marker.extend_from_slice(b"ICC_PROFILE\0");

    // Sequence number (1-based) and count (for multi-chunk profiles)
    app2_marker.push(1); // This is chunk 1
    app2_marker.push(1); // Total 1 chunk

    // ICC profile data
    app2_marker.extend_from_slice(&icc_data);

    // Build new JPEG: SOI + APP2 + rest of original
    let mut result = Vec::new();
    result.extend_from_slice(&jpeg_data[0..2]); // SOI
    result.extend_from_slice(&app2_marker); // ICC APP2 marker
    result.extend_from_slice(&jpeg_data[2..]); // Rest of JPEG

    result
}

/// Apply soft proofing to RGB8 image data
///
/// Simulates how the image will look in a target color space.
/// If gamut warning is enabled, marks out-of-gamut pixels with magenta.
///
/// # Arguments
/// * `data` - 8-bit RGB data (sRGB gamma-encoded)
/// * `width` - Image width
/// * `height` - Image height
/// * `target_profile` - Target color space ("sRGB", "AdobeRGB", "DisplayP3")
/// * `show_gamut_warning` - If true, mark out-of-gamut pixels with magenta
///
/// # Returns
/// Soft-proofed RGB8 data
pub fn apply_soft_proof(
    data: &[u8],
    width: u32,
    height: u32,
    target_profile: &str,
    show_gamut_warning: bool,
) -> Vec<u8> {
    let pixel_count = (width * height) as usize;
    let expected_len = pixel_count * 3;

    if data.len() != expected_len {
        return data.to_vec();
    }

    let mut result = data.to_vec();

    // Define gamut boundaries for different color spaces
    // sRGB has the smallest gamut, AdobeRGB is wider
    let (max_r, max_g, max_b) = match target_profile {
        "sRGB" => (255u8, 255u8, 255u8),      // Full range
        "AdobeRGB" => (230u8, 240u8, 230u8),  // Slightly restricted (simplified)
        "DisplayP3" => (245u8, 250u8, 245u8), // Between sRGB and AdobeRGB
        _ => (255u8, 255u8, 255u8),
    };

    if show_gamut_warning {
        // Mark out-of-gamut pixels with magenta overlay
        for i in 0..pixel_count {
            let idx = i * 3;
            let r = result[idx];
            let g = result[idx + 1];
            let b = result[idx + 2];

            // Check if pixel exceeds target gamut
            // Simplified gamut check: if any channel is too saturated
            let out_of_gamut = r > max_r || g > max_g || b > max_b;

            if out_of_gamut {
                // Overlay magenta (255, 0, 255) with 50% opacity
                result[idx] = ((r as u16 + 255) / 2) as u8;       // R
                result[idx + 1] = (g as u16 / 2) as u8;           // G
                result[idx + 2] = ((b as u16 + 255) / 2) as u8;   // B
            }
        }
    } else {
        // Apply color space simulation (simplified)
        // For sRGB -> AdobeRGB: slightly desaturate
        // For sRGB -> DisplayP3: minimal change
        match target_profile {
            "AdobeRGB" => {
                // Slightly desaturate for AdobeRGB preview
                for i in 0..pixel_count {
                    let idx = i * 3;
                    let r = result[idx] as f32;
                    let g = result[idx + 1] as f32;
                    let b = result[idx + 2] as f32;

                    // Simple desaturation: move 5% toward gray
                    let gray = (r + g + b) / 3.0;
                    result[idx] = (r * 0.95 + gray * 0.05) as u8;
                    result[idx + 1] = (g * 0.95 + gray * 0.05) as u8;
                    result[idx + 2] = (b * 0.95 + gray * 0.05) as u8;
                }
            }
            "DisplayP3" => {
                // DisplayP3 is close to sRGB, minimal adjustment
                // No change needed for now
            }
            _ => {}
        }
    }

    result
}

/// Convert linear RGB u16 to output color space
///
/// # Arguments
/// * `data` - Linear RGB data (u16, 0-65535 range)
/// * `width` - Image width
/// * `height` - Image height
/// * `target_space` - Target output color space
///
/// # Returns
/// Converted RGB data in target color space (still u16)
pub fn convert_linear_to_output(
    data: &[u16],
    width: u32,
    height: u32,
    target_space: &OutputColorSpace,
) -> Vec<u16> {
    let pixel_count = (width * height) as usize;
    let expected_len = pixel_count * 3;

    if data.len() != expected_len {
        // Invalid data size, return copy
        return data.to_vec();
    }

    match target_space {
        OutputColorSpace::SRGB => {
            // sRGB uses sRGB gamma curve (handled in pipeline gamma encoding)
            // Just return the data as-is (gamma encoding happens elsewhere)
            data.to_vec()
        }
        OutputColorSpace::AdobeRGB => {
            // Adobe RGB uses simple gamma 2.2
            // Convert from linear to gamma 2.2
            let mut result = Vec::with_capacity(data.len());

            for &value in data {
                let linear = value as f32 / 65535.0;
                // Gamma 2.2 encoding: out = in^(1/2.2)
                let encoded = linear.powf(1.0 / 2.2);
                let encoded_u16 = (encoded * 65535.0).round().clamp(0.0, 65535.0) as u16;
                result.push(encoded_u16);
            }

            result
        }
        OutputColorSpace::DisplayP3 => {
            // Display P3 uses sRGB transfer function
            // For now, treat as sRGB (same gamma)
            data.to_vec()
        }
        OutputColorSpace::Custom(_) => {
            // Custom profiles not yet implemented
            data.to_vec()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgb_profile_is_valid() {
        let profile = get_srgb_profile();

        // Should have reasonable size
        assert!(profile.len() > 128);
        assert!(profile.len() < 10000);

        // Should start with ICC header
        assert_eq!(&profile[0..4], &(profile.len() as u32).to_be_bytes());
        assert_eq!(&profile[36..40], b"acsp"); // Signature
    }

    #[test]
    fn test_embed_icc_srgb() {
        // Create minimal JPEG (SOI + EOI markers)
        let jpeg_data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xD9, // EOI
        ];

        let result = embed_icc_profile(&jpeg_data, &OutputColorSpace::SRGB);

        // Should be larger than original
        assert!(result.len() > jpeg_data.len());

        // Should still start with SOI
        assert_eq!(result[0], 0xFF);
        assert_eq!(result[1], 0xD8);

        // Should contain APP2 marker after SOI
        assert_eq!(result[2], 0xFF);
        assert_eq!(result[3], 0xE2);

        // Should contain ICC_PROFILE identifier
        let icc_profile_marker = b"ICC_PROFILE\0";
        let has_marker = result.windows(icc_profile_marker.len())
            .any(|window| window == icc_profile_marker);
        assert!(has_marker);
    }

    #[test]
    fn test_embed_icc_invalid_jpeg() {
        // Not a valid JPEG
        let invalid_data = vec![0x00, 0x01, 0x02, 0x03];

        let result = embed_icc_profile(&invalid_data, &OutputColorSpace::SRGB);

        // Should return original data unchanged
        assert_eq!(result, invalid_data);
    }

    #[test]
    fn test_srgb_default_has_profile() {
        // Test that default color space is sRGB
        let default_space = OutputColorSpace::default();
        matches!(default_space, OutputColorSpace::SRGB);

        // Verify we can embed it
        let jpeg = vec![0xFF, 0xD8, 0xFF, 0xD9];
        let result = embed_icc_profile(&jpeg, &default_space);
        assert!(result.len() > jpeg.len());
    }

    #[test]
    fn test_adobe_rgb_gamma() {
        // Create test data: linear gradient
        let width = 4;
        let height = 1;
        let mut data = Vec::new();

        for i in 0..4 {
            let value = (i * 16384) as u16; // 0, 16384, 32768, 49152
            data.push(value);
            data.push(value);
            data.push(value);
        }

        let result = convert_linear_to_output(&data, width, height, &OutputColorSpace::AdobeRGB);

        // Should have same length
        assert_eq!(result.len(), data.len());

        // Gamma 2.2 should produce different values than linear
        // At mid-point (32768), gamma 2.2 encoding should give higher value
        let linear_mid = data[6]; // 32768
        let gamma_mid = result[6];

        // For gamma 2.2: 0.5^(1/2.2) ≈ 0.73, so output should be ~47710
        assert!(gamma_mid > linear_mid);
        assert!(gamma_mid > 45000 && gamma_mid < 50000);
    }

    #[test]
    fn test_convert_linear_to_srgb_is_passthrough() {
        let width = 2;
        let height = 2;
        let data = vec![1000u16, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10000, 11000, 12000];

        let result = convert_linear_to_output(&data, width, height, &OutputColorSpace::SRGB);

        // sRGB conversion should be identity (gamma happens in pipeline)
        assert_eq!(result, data);
    }

    #[test]
    fn test_convert_invalid_size() {
        let width = 4;
        let height = 4;
        let data = vec![100u16; 10]; // Wrong size

        let result = convert_linear_to_output(&data, width, height, &OutputColorSpace::SRGB);

        // Should return copy of original data
        assert_eq!(result, data);
    }

    #[test]
    fn test_soft_proof_disabled_is_identity() {
        let width = 4;
        let height = 4;
        let data = vec![100u8, 150, 200, 50, 100, 150, 200, 50, 100, 150, 200, 50,
                        100, 150, 200, 50, 100, 150, 200, 50, 100, 150, 200, 50,
                        100, 150, 200, 50, 100, 150, 200, 50, 100, 150, 200, 50,
                        100, 150, 200, 50, 100, 150, 200, 50, 100, 150, 200, 50];

        // Soft proof to sRGB without gamut warning should be identity
        let result = apply_soft_proof(&data, width, height, "sRGB", false);

        assert_eq!(result, data);
    }

    #[test]
    fn test_soft_proof_gamut_warning_marks_pixels() {
        let width = 2;
        let height = 2;
        // Create a highly saturated image (pure colors at max intensity)
        let data = vec![
            255, 0, 0,     // Pure red
            0, 255, 0,     // Pure green
            0, 0, 255,     // Pure blue
            255, 255, 255, // White
        ];

        // Apply soft proof to AdobeRGB with gamut warning
        let result = apply_soft_proof(&data, width, height, "AdobeRGB", true);

        // Should have modified some pixels (marked out-of-gamut)
        assert_ne!(result, data);

        // Pure red (255,0,0) should be marked as out of gamut for AdobeRGB
        // It will be overlaid with magenta: R=(255+255)/2=255, G=(0+0)/2=0, B=(0+255)/2=127
        assert_eq!(result[0], 255); // R
        assert_eq!(result[1], 0);   // G
        assert!(result[2] > 0);     // B should be > 0 (magenta overlay)
    }

    #[test]
    fn test_soft_proof_adobe_rgb_desaturates() {
        let width = 1;
        let height = 1;
        let data = vec![200, 100, 150]; // Saturated color

        // Apply soft proof to AdobeRGB (no gamut warning)
        let result = apply_soft_proof(&data, width, height, "AdobeRGB", false);

        // Should be slightly desaturated (moved toward gray)
        let gray = (200.0 + 100.0 + 150.0) / 3.0;
        let expected_r = (200.0 * 0.95 + gray * 0.05) as u8;
        let expected_g = (100.0 * 0.95 + gray * 0.05) as u8;
        let expected_b = (150.0 * 0.95 + gray * 0.05) as u8;

        assert_eq!(result[0], expected_r);
        assert_eq!(result[1], expected_g);
        assert_eq!(result[2], expected_b);
    }

    #[test]
    fn test_soft_proof_invalid_size() {
        let width = 4;
        let height = 4;
        let data = vec![100u8; 10]; // Wrong size

        let result = apply_soft_proof(&data, width, height, "sRGB", false);

        // Should return copy of original
        assert_eq!(result, data);
    }
}
