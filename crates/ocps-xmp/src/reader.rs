//! XMP sidecar reader with quick-xml parsing

use crate::{IptcData, XmpDevelopSettings, XmpError};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Read an XMP sidecar file and extract develop settings and IPTC data
pub fn read_sidecar(path: &Path) -> Result<(XmpDevelopSettings, IptcData), XmpError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut xml_reader = Reader::from_reader(reader);
    xml_reader.config_mut().trim_text(true);

    let mut develop = XmpDevelopSettings::default();
    let mut iptc = IptcData::default();
    let mut current_element = String::new();
    let mut text_buffer = String::new();
    let mut attributes = HashMap::new();

    let mut buf = Vec::new();
    loop {
        buf.clear();
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                current_element = name.clone();

                // Parse attributes
                attributes.clear();
                let decoder = xml_reader.decoder();
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = attr
                        .decode_and_unescape_value(decoder)
                        .unwrap_or_default()
                        .to_string();
                    attributes.insert(key, value);
                }

                // Parse crs: namespace attributes (Adobe Camera Raw Settings)
                parse_crs_attributes(&attributes, &mut develop);

                // Parse xmp: namespace attributes
                parse_xmp_attributes(&attributes, &mut develop);

                // Parse dc: namespace (Dublin Core)
                if name.contains("dc:") {
                    parse_dc_element(&name, &attributes, &mut iptc);
                }

                // Parse Iptc4xmpCore: namespace
                parse_iptc_attributes(&attributes, &mut iptc);
            }
            Ok(Event::Text(e)) => {
                text_buffer = e.unescape().map(|s| s.to_string()).unwrap_or_default();
            }
            Ok(Event::End(_)) => {
                // Process text content if any
                if !text_buffer.is_empty() {
                    if current_element.contains("dc:title") || current_element.contains("dc:description") {
                        iptc.title = Some(text_buffer.clone());
                    }
                    text_buffer.clear();
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(XmpError::Parse(format!("XML error: {}", e))),
            _ => {}
        }
    }

    Ok((develop, iptc))
}

fn parse_crs_attributes(attrs: &HashMap<String, String>, develop: &mut XmpDevelopSettings) {
    for (key, value) in attrs {
        match key.as_str() {
            "crs:Temperature" => develop.temperature = value.parse().ok(),
            "crs:Tint" => develop.tint = value.parse().ok(),
            "crs:Exposure2012" | "crs:Exposure" => develop.exposure = value.parse().ok(),
            "crs:Contrast2012" | "crs:Contrast" => develop.contrast = value.parse().ok(),
            "crs:Highlights2012" | "crs:Highlights" => develop.highlights = value.parse().ok(),
            "crs:Shadows2012" | "crs:Shadows" => develop.shadows = value.parse().ok(),
            "crs:Whites2012" | "crs:Whites" => develop.whites = value.parse().ok(),
            "crs:Blacks2012" | "crs:Blacks" => develop.blacks = value.parse().ok(),
            "crs:Clarity2012" | "crs:Clarity" => develop.clarity = value.parse().ok(),
            "crs:Dehaze" => develop.dehaze = value.parse().ok(),
            "crs:Vibrance" => develop.vibrance = value.parse().ok(),
            "crs:Saturation" => develop.saturation = value.parse().ok(),
            "crs:ProcessVersion" => develop.process_version = Some(value.clone()),
            _ => {}
        }
    }
}

fn parse_xmp_attributes(attrs: &HashMap<String, String>, develop: &mut XmpDevelopSettings) {
    for (key, value) in attrs {
        match key.as_str() {
            "xmp:Rating" => develop.rating = value.parse().ok(),
            "xmp:Label" => develop.label = Some(value.clone()),
            _ => {}
        }
    }
}

fn parse_dc_element(name: &str, _attrs: &HashMap<String, String>, _iptc: &mut IptcData) {
    // dc: namespace parsing
    // Title, description, keywords are typically in child elements
    // This is a simplified parser - full implementation would handle rdf:Seq etc.
    if name.contains("dc:title") {
        // Title parsing happens in text content
    } else if name.contains("dc:description") {
        // Description parsing
    }
}

fn parse_iptc_attributes(attrs: &HashMap<String, String>, iptc: &mut IptcData) {
    for (key, value) in attrs {
        match key.as_str() {
            "Iptc4xmpCore:CreatorContactInfo" | "photoshop:City" => {
                iptc.city = Some(value.clone());
            }
            "Iptc4xmpCore:CiAdrCtry" | "photoshop:Country" => {
                iptc.country = Some(value.clone());
            }
            "dc:creator" | "photoshop:Credit" => {
                iptc.creator = Some(value.clone());
            }
            "dc:rights" | "photoshop:Copyright" => {
                iptc.copyright = Some(value.clone());
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_basic_xmp() {
        let xmp_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:RDF>
    <rdf:Description
      crs:Temperature="5500"
      crs:Tint="10"
      crs:Exposure2012="1.5"
      crs:Contrast2012="20"
      xmp:Rating="5"
      xmp:Label="Red"
      />
  </rdf:RDF>
</x:xmpmeta>"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(xmp_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let (develop, _iptc) = read_sidecar(temp_file.path()).unwrap();

        assert_eq!(develop.temperature, Some(5500));
        assert_eq!(develop.tint, Some(10));
        assert_eq!(develop.exposure, Some(1.5));
        assert_eq!(develop.contrast, Some(20));
        assert_eq!(develop.rating, Some(5));
        assert_eq!(develop.label, Some("Red".to_string()));
    }
}
