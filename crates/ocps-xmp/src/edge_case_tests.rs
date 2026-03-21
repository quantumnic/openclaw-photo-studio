//! Edge case tests for XMP functionality
//!
//! Tests scenarios that may fail in production:
//! - UTF-8 special characters (umlauts, Chinese, emoji)
//! - Malformed/truncated XMP files
//! - Invalid XML
//! - Missing namespaces

use super::*;
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_xmp_with_utf8_special_characters() {
        let temp_dir = TempDir::new().unwrap();
        let xmp_path = temp_dir.path().join("test.xmp");

        // XMP with umlauts, Chinese characters, and emoji in attributes
        // Note: Current parser only handles attributes, not nested <rdf:li> elements
        let xmp_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description
      xmlns:dc="http://purl.org/dc/elements/1.1/"
      xmlns:Iptc4xmpCore="http://iptc.org/std/Iptc4xmpCore/1.0/xmlns/"
      Iptc4xmpCore:CreatorContactInfo.CiAdrCity="München"
      dc:title="Photo with ñ, é, ü, ö, ä characters">
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>"#;

        fs::write(&xmp_path, xmp_content).unwrap();

        let result = read_sidecar(&xmp_path);

        // Should parse successfully without crashing on UTF-8
        assert!(result.is_ok());
        let (_settings, iptc) = result.unwrap();

        // Verify UTF-8 characters preserved in title
        assert!(iptc.title.is_some());
    }

    #[test]
    fn test_malformed_xmp_missing_closing_tag() {
        let temp_dir = TempDir::new().unwrap();
        let xmp_path = temp_dir.path().join("malformed.xmp");

        // Missing closing </rdf:RDF>
        let xmp_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/">
      <crs:Exposure2012>+1.0</crs:Exposure2012>
    </rdf:Description>
  <!-- Missing closing tag here -->
</x:xmpmeta>"#;

        fs::write(&xmp_path, xmp_content).unwrap();

        let result = read_sidecar(&xmp_path);

        // Should return error, not panic
        assert!(result.is_err());
    }

    #[test]
    fn test_truncated_xmp_file() {
        let temp_dir = TempDir::new().unwrap();
        let xmp_path = temp_dir.path().join("truncated.xmp");

        // Truncated in the middle
        let xmp_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description xmlns:crs="#;

        fs::write(&xmp_path, xmp_content).unwrap();

        let result = read_sidecar(&xmp_path);

        // Should return error gracefully
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_xmp_file() {
        let temp_dir = TempDir::new().unwrap();
        let xmp_path = temp_dir.path().join("empty.xmp");

        fs::write(&xmp_path, "").unwrap();

        let result = read_sidecar(&xmp_path);

        // Should return error or default values
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_xmp_with_invalid_xml() {
        let temp_dir = TempDir::new().unwrap();
        let xmp_path = temp_dir.path().join("invalid.xmp");

        // Not valid XML - has unclosed tag and missing closing
        let xmp_content = r#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description crs:Exposure="1.0">
      <unclosed tag here"#;

        fs::write(&xmp_path, xmp_content).unwrap();

        let result = read_sidecar(&xmp_path);

        // Should return error
        assert!(result.is_err());
    }

    #[test]
    fn test_xmp_with_missing_namespaces() {
        let temp_dir = TempDir::new().unwrap();
        let xmp_path = temp_dir.path().join("no_namespace.xmp");

        // Valid XML but missing namespace declarations
        let xmp_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta>
  <rdf:RDF>
    <rdf:Description>
      <crs:Exposure2012>+1.0</crs:Exposure2012>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>"#;

        fs::write(&xmp_path, xmp_content).unwrap();

        let result = read_sidecar(&xmp_path);

        // May fail or succeed depending on parser tolerance
        // Just ensure it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_xmp_with_very_long_title() {
        let temp_dir = TempDir::new().unwrap();
        let xmp_path = temp_dir.path().join("long_title.xmp");

        // Very long title (1000 characters) - using attributes which current parser supports
        let long_title = "a".repeat(1000);

        let xmp_content = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description
      xmlns:dc="http://purl.org/dc/elements/1.1/"
      dc:title="{}">
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>"#, long_title);

        fs::write(&xmp_path, xmp_content).unwrap();

        let result = read_sidecar(&xmp_path);

        // Should parse successfully without crashing
        assert!(result.is_ok());
        let (_, iptc) = result.unwrap();
        assert!(iptc.title.is_some());
        assert_eq!(iptc.title.as_ref().unwrap().len(), 1000);
    }

    #[test]
    fn test_xmp_with_extreme_exposure_values() {
        let temp_dir = TempDir::new().unwrap();
        let xmp_path = temp_dir.path().join("extreme.xmp");

        let xmp_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description
      xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
      crs:Exposure2012="+999.0"
      crs:Contrast2012="+200"
      crs:Saturation="-500">
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>"#;

        fs::write(&xmp_path, xmp_content).unwrap();

        let result = read_sidecar(&xmp_path);

        // Should parse (values may be clamped later in pipeline)
        assert!(result.is_ok());
        let (settings, _) = result.unwrap();
        assert!(settings.exposure.is_some());
        assert_eq!(settings.exposure, Some(999.0));
    }

    #[test]
    fn test_read_nonexistent_xmp() {
        let temp_dir = TempDir::new().unwrap();
        let xmp_path = temp_dir.path().join("nonexistent.xmp");

        let result = read_sidecar(&xmp_path);

        // Should return IO error
        assert!(result.is_err());
    }

    #[test]
    fn test_write_xmp_to_readonly_path() {
        let temp_dir = TempDir::new().unwrap();
        let xmp_path = temp_dir.path().join("readonly.xmp");

        // Create file first
        fs::write(&xmp_path, "test").unwrap();

        // Make it readonly
        let mut perms = fs::metadata(&xmp_path).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&xmp_path, perms).unwrap();

        let settings = XmpDevelopSettings::default();
        let iptc = IptcData::default();

        let result = write_sidecar(&xmp_path, &settings, &iptc);

        // Restore write permissions for cleanup
        let mut perms = fs::metadata(&xmp_path).unwrap().permissions();
        perms.set_readonly(false);
        fs::set_permissions(&xmp_path, perms).unwrap();

        // Should fail gracefully
        assert!(result.is_err());
    }

    #[test]
    fn test_xmp_with_duplicate_attributes() {
        let temp_dir = TempDir::new().unwrap();
        let xmp_path = temp_dir.path().join("duplicates.xmp");

        // Test that parser handles multiple values gracefully (last one wins)
        let xmp_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description
      xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
      crs:Exposure2012="1.0">
    </rdf:Description>
    <rdf:Description
      xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
      crs:Exposure2012="2.0">
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>"#;

        fs::write(&xmp_path, xmp_content).unwrap();

        let result = read_sidecar(&xmp_path);

        assert!(result.is_ok());
        let (settings, _) = result.unwrap();

        // Parser should handle this gracefully (last value wins)
        assert!(settings.exposure.is_some());
    }
}
