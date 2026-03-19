//! XMP sidecar writer

use crate::{IptcData, XmpDevelopSettings, XmpError};
use std::fs::File;
use std::io::Write;
use std::path::Path;

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
    write!(file, r#"    <rdf:Description"#)?;

    // Write namespaces
    writeln!(file)?;
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
        r#"      xmlns:Iptc4xmpCore="http://iptc.org/std/Iptc4xmpCore/1.0/xmlns/">"#
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
}
