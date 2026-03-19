//! XMP sidecar writer

use crate::{IptcData, XmpDevelopSettings, XmpError};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Merge new settings into existing sidecar file
/// If the file doesn't exist, creates a new one
/// New values override existing ones
pub fn merge_sidecar(
    path: &Path,
    develop: &XmpDevelopSettings,
    iptc: &IptcData,
) -> Result<(), XmpError> {
    // Try to read existing sidecar
    let (mut existing_develop, mut existing_iptc) = if path.exists() {
        crate::reader::read_sidecar(path)?
    } else {
        (XmpDevelopSettings::default(), IptcData::default())
    };

    // Merge develop settings (new values override)
    if develop.temperature.is_some() {
        existing_develop.temperature = develop.temperature;
    }
    if develop.tint.is_some() {
        existing_develop.tint = develop.tint;
    }
    if develop.exposure.is_some() {
        existing_develop.exposure = develop.exposure;
    }
    if develop.contrast.is_some() {
        existing_develop.contrast = develop.contrast;
    }
    if develop.highlights.is_some() {
        existing_develop.highlights = develop.highlights;
    }
    if develop.shadows.is_some() {
        existing_develop.shadows = develop.shadows;
    }
    if develop.whites.is_some() {
        existing_develop.whites = develop.whites;
    }
    if develop.blacks.is_some() {
        existing_develop.blacks = develop.blacks;
    }
    if develop.clarity.is_some() {
        existing_develop.clarity = develop.clarity;
    }
    if develop.dehaze.is_some() {
        existing_develop.dehaze = develop.dehaze;
    }
    if develop.vibrance.is_some() {
        existing_develop.vibrance = develop.vibrance;
    }
    if develop.saturation.is_some() {
        existing_develop.saturation = develop.saturation;
    }
    if develop.rating.is_some() {
        existing_develop.rating = develop.rating;
    }
    if develop.label.is_some() {
        existing_develop.label = develop.label.clone();
    }
    if develop.process_version.is_some() {
        existing_develop.process_version = develop.process_version.clone();
    }

    // Merge IPTC data
    if iptc.title.is_some() {
        existing_iptc.title = iptc.title.clone();
    }
    if iptc.description.is_some() {
        existing_iptc.description = iptc.description.clone();
    }
    if !iptc.keywords.is_empty() {
        existing_iptc.keywords = iptc.keywords.clone();
    }
    if iptc.creator.is_some() {
        existing_iptc.creator = iptc.creator.clone();
    }
    if iptc.copyright.is_some() {
        existing_iptc.copyright = iptc.copyright.clone();
    }
    if iptc.city.is_some() {
        existing_iptc.city = iptc.city.clone();
    }
    if iptc.country.is_some() {
        existing_iptc.country = iptc.country.clone();
    }
    if iptc.country_code.is_some() {
        existing_iptc.country_code = iptc.country_code.clone();
    }

    // Write merged data
    write_sidecar(path, &existing_develop, &existing_iptc)
}

/// Write XMP sidecar file in Adobe-compatible format
pub fn write_sidecar(
    path: &Path,
    develop: &XmpDevelopSettings,
    iptc: &IptcData,
) -> Result<(), XmpError> {
    let mut file = File::create(path)?;

    // Write XMP header
    writeln!(file, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(
        file,
        r#"<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="OpenClaw Photo Studio">"#
    )?;
    writeln!(
        file,
        r#"  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">"#
    )?;
    writeln!(file, r#"    <rdf:Description"#)?;

    // Write namespaces (keep tag open)
    writeln!(
        file,
        r#"      xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/""#
    )?;
    writeln!(
        file,
        r#"      xmlns:xmp="http://ns.adobe.com/xap/1.0/""#
    )?;
    writeln!(
        file,
        r#"      xmlns:dc="http://purl.org/dc/elements/1.1/""#
    )?;
    writeln!(
        file,
        r#"      xmlns:photoshop="http://ns.adobe.com/photoshop/1.0/""#
    )?;
    writeln!(
        file,
        r#"      xmlns:Iptc4xmpCore="http://iptc.org/std/Iptc4xmpCore/1.0/xmlns/""#
    )?;

    // Write develop settings (crs: namespace)
    if let Some(temp) = develop.temperature {
        writeln!(file, r#"      crs:Temperature="{}""#, temp)?;
    }
    if let Some(tint) = develop.tint {
        writeln!(file, r#"      crs:Tint="{}""#, tint)?;
    }
    if let Some(exp) = develop.exposure {
        writeln!(file, r#"      crs:Exposure2012="{}""#, exp)?;
    }
    if let Some(cont) = develop.contrast {
        writeln!(file, r#"      crs:Contrast2012="{}""#, cont)?;
    }
    if let Some(hl) = develop.highlights {
        writeln!(file, r#"      crs:Highlights2012="{}""#, hl)?;
    }
    if let Some(sh) = develop.shadows {
        writeln!(file, r#"      crs:Shadows2012="{}""#, sh)?;
    }
    if let Some(w) = develop.whites {
        writeln!(file, r#"      crs:Whites2012="{}""#, w)?;
    }
    if let Some(b) = develop.blacks {
        writeln!(file, r#"      crs:Blacks2012="{}""#, b)?;
    }
    if let Some(cl) = develop.clarity {
        writeln!(file, r#"      crs:Clarity2012="{}""#, cl)?;
    }
    if let Some(dh) = develop.dehaze {
        writeln!(file, r#"      crs:Dehaze="{}""#, dh)?;
    }
    if let Some(vib) = develop.vibrance {
        writeln!(file, r#"      crs:Vibrance="{}""#, vib)?;
    }
    if let Some(sat) = develop.saturation {
        writeln!(file, r#"      crs:Saturation="{}""#, sat)?;
    }

    // Write process version
    if let Some(ref pv) = develop.process_version {
        writeln!(file, r#"      crs:ProcessVersion="{}""#, pv)?;
    } else {
        writeln!(file, r#"      crs:ProcessVersion="ocps-1.0""#)?;
    }

    // Write rating and label (xmp: namespace)
    if let Some(rating) = develop.rating {
        writeln!(file, r#"      xmp:Rating="{}""#, rating)?;
    }
    if let Some(ref label) = develop.label {
        writeln!(file, r#"      xmp:Label="{}""#, label)?;
    }

    // Write IPTC data
    if let Some(ref title) = iptc.title {
        writeln!(file, r#"      dc:title="{}""#, escape_xml(title))?;
    }
    if let Some(ref desc) = iptc.description {
        writeln!(file, r#"      dc:description="{}""#, escape_xml(desc))?;
    }
    if let Some(ref creator) = iptc.creator {
        writeln!(file, r#"      dc:creator="{}""#, escape_xml(creator))?;
    }
    if let Some(ref copyright) = iptc.copyright {
        writeln!(file, r#"      dc:rights="{}""#, escape_xml(copyright))?;
    }
    if let Some(ref city) = iptc.city {
        writeln!(file, r#"      photoshop:City="{}""#, escape_xml(city))?;
    }
    if let Some(ref country) = iptc.country {
        writeln!(file, r#"      photoshop:Country="{}""#, escape_xml(country))?;
    }

    // Close description and RDF
    writeln!(file, r#"    />"#)?;
    writeln!(file, r#"  </rdf:RDF>"#)?;
    writeln!(file, r#"</x:xmpmeta>"#)?;

    Ok(())
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_write_xmp() {
        let develop = XmpDevelopSettings {
            temperature: Some(5500),
            tint: Some(10),
            exposure: Some(1.5),
            contrast: Some(20),
            rating: Some(5),
            label: Some("Red".to_string()),
            process_version: Some("ocps-1.0".to_string()),
            ..Default::default()
        };

        let iptc = IptcData {
            title: Some("Test Photo".to_string()),
            creator: Some("Test Photographer".to_string()),
            ..Default::default()
        };

        let temp_file = NamedTempFile::new().unwrap();
        write_sidecar(temp_file.path(), &develop, &iptc).unwrap();

        // Verify file was created
        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("crs:Temperature=\"5500\""));
        assert!(content.contains("crs:Exposure2012=\"1.5\""));
        assert!(content.contains("xmp:Rating=\"5\""));
        assert!(content.contains("dc:title=\"Test Photo\""));
    }

    #[test]
    fn test_write_and_read_roundtrip() {
        // Create XMP with known values
        let develop = XmpDevelopSettings {
            temperature: Some(6500),
            tint: Some(-15),
            exposure: Some(0.75),
            contrast: Some(30),
            highlights: Some(-50),
            shadows: Some(40),
            whites: Some(-20),
            blacks: Some(10),
            clarity: Some(25),
            dehaze: Some(15),
            vibrance: Some(20),
            saturation: Some(-10),
            rating: Some(4),
            label: Some("Green".to_string()),
            process_version: Some("test-1.0".to_string()),
        };

        let iptc = IptcData {
            title: Some("Roundtrip Test".to_string()),
            description: Some("Test description with \"quotes\" & special <chars>".to_string()),
            keywords: vec!["landscape".to_string(), "nature".to_string()],
            creator: Some("Test User".to_string()),
            copyright: Some("© 2026 Test".to_string()),
            city: Some("Zurich".to_string()),
            country: Some("Switzerland".to_string()),
            country_code: Some("CH".to_string()),
        };

        let temp_file = NamedTempFile::new().unwrap();

        // Write sidecar
        write_sidecar(temp_file.path(), &develop, &iptc).unwrap();

        // Read it back
        let (read_develop, read_iptc) = crate::reader::read_sidecar(temp_file.path()).unwrap();

        // Verify develop settings
        assert_eq!(read_develop.temperature, Some(6500));
        assert_eq!(read_develop.tint, Some(-15));
        assert_eq!(read_develop.exposure, Some(0.75));
        assert_eq!(read_develop.contrast, Some(30));
        assert_eq!(read_develop.highlights, Some(-50));
        assert_eq!(read_develop.shadows, Some(40));
        assert_eq!(read_develop.whites, Some(-20));
        assert_eq!(read_develop.blacks, Some(10));
        assert_eq!(read_develop.clarity, Some(25));
        assert_eq!(read_develop.dehaze, Some(15));
        assert_eq!(read_develop.vibrance, Some(20));
        assert_eq!(read_develop.saturation, Some(-10));
        assert_eq!(read_develop.rating, Some(4));
        assert_eq!(read_develop.label, Some("Green".to_string()));

        // Verify IPTC
        assert_eq!(read_iptc.creator, Some("Test User".to_string()));
        assert_eq!(read_iptc.city, Some("Zurich".to_string()));
        assert_eq!(read_iptc.country, Some("Switzerland".to_string()));
    }

    #[test]
    fn test_write_only_nondefault() {
        // Only set exposure, leave everything else as None
        let develop = XmpDevelopSettings {
            exposure: Some(1.0),
            ..Default::default()
        };

        let iptc = IptcData::default();

        let temp_file = NamedTempFile::new().unwrap();
        write_sidecar(temp_file.path(), &develop, &iptc).unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();

        // Should contain exposure
        assert!(content.contains("crs:Exposure2012=\"1\""));

        // Should NOT contain temperature (it's None)
        assert!(!content.contains("crs:Temperature="));

        // Should NOT contain tint
        assert!(!content.contains("crs:Tint="));
    }

    #[test]
    fn test_merge_preserves_existing() {
        let temp_file = NamedTempFile::new().unwrap();

        // Write initial sidecar with exposure
        let develop1 = XmpDevelopSettings {
            exposure: Some(0.5),
            contrast: Some(20),
            ..Default::default()
        };
        let iptc1 = IptcData {
            creator: Some("Original Creator".to_string()),
            ..Default::default()
        };
        write_sidecar(temp_file.path(), &develop1, &iptc1).unwrap();

        // Merge with only tint set
        let develop2 = XmpDevelopSettings {
            tint: Some(10),
            ..Default::default()
        };
        let iptc2 = IptcData {
            copyright: Some("© 2026".to_string()),
            ..Default::default()
        };
        merge_sidecar(temp_file.path(), &develop2, &iptc2).unwrap();

        // Read back and verify both exposure and tint are present
        let (result_develop, result_iptc) = crate::reader::read_sidecar(temp_file.path()).unwrap();

        assert_eq!(result_develop.exposure, Some(0.5), "Original exposure should be preserved");
        assert_eq!(result_develop.contrast, Some(20), "Original contrast should be preserved");
        assert_eq!(result_develop.tint, Some(10), "New tint should be added");
        assert_eq!(result_iptc.creator, Some("Original Creator".to_string()), "Original creator preserved");
        assert_eq!(result_iptc.copyright, Some("© 2026".to_string()), "New copyright added");
    }

    #[test]
    fn test_roundtrip_iptc() {
        let develop = XmpDevelopSettings::default();
        let iptc = IptcData {
            title: Some("IPTC Test".to_string()),
            description: Some("Testing IPTC roundtrip".to_string()),
            keywords: vec!["test".to_string(), "iptc".to_string()],
            creator: Some("John Doe".to_string()),
            copyright: Some("© 2026 John Doe".to_string()),
            city: Some("Bern".to_string()),
            country: Some("Switzerland".to_string()),
            country_code: Some("CH".to_string()),
        };

        let temp_file = NamedTempFile::new().unwrap();
        write_sidecar(temp_file.path(), &develop, &iptc).unwrap();

        let (_read_develop, read_iptc) = crate::reader::read_sidecar(temp_file.path()).unwrap();

        assert_eq!(read_iptc.creator, Some("John Doe".to_string()));
        assert_eq!(read_iptc.copyright, Some("© 2026 John Doe".to_string()));
        assert_eq!(read_iptc.city, Some("Bern".to_string()));
        assert_eq!(read_iptc.country, Some("Switzerland".to_string()));
    }

    #[test]
    fn test_merge_creates_new_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_extension("xmp");

        // Ensure file doesn't exist
        let _ = std::fs::remove_file(&path);

        // Merge should create new file
        let develop = XmpDevelopSettings {
            exposure: Some(1.0),
            ..Default::default()
        };
        let iptc = IptcData::default();

        merge_sidecar(&path, &develop, &iptc).unwrap();

        // Verify file was created
        assert!(path.exists());

        // Verify contents
        let (read_develop, _) = crate::reader::read_sidecar(&path).unwrap();
        assert_eq!(read_develop.exposure, Some(1.0));

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }
}
