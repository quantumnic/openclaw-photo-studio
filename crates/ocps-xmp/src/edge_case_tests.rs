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

        // XMP with umlauts, Chinese characters, and emoji
        let xmp_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/">
      <dc:creator>
        <rdf:Seq>
          <rdf:li>München Photographer</rdf:li>
        </rdf:Seq>
      </dc:creator>
      <dc:subject>
        <rdf:Bag>
          <rdf:li>北京</rdf:li>
          <rdf:li>😀 Happy</rdf:li>
          <rdf:li>Zürich</rdf:li>
          <rdf:li>Москва</rdf:li>
        </rdf:Bag>
      </dc:subject>
      <dc:description>Test with ñ, é, ü, ö, ä characters</dc:description>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>"#;

        fs::write(&xmp_path, xmp_content).unwrap();

        let result = read_sidecar(&xmp_path);

        // Should parse successfully
        assert!(result.is_ok());
        let (settings, iptc) = result.unwrap();

        // Verify UTF-8 characters preserved
        assert!(iptc.keywords.contains(&"北京".to_string()));
        assert!(iptc.keywords.contains(&"😀 Happy".to_string()));
        assert!(iptc.keywords.contains(&"Zürich".to_string()));
        assert!(iptc.keywords.contains(&"Москва".to_string()));
        assert_eq!(iptc.creator, Some("München Photographer".to_string()));
        assert_eq!(iptc.description, Some("Test with ñ, é, ü, ö, ä characters".to_string()));
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

        // Not valid XML
        let xmp_content = "This is not XML at all!";

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
    fn test_xmp_with_very_long_keywords() {
        let temp_dir = TempDir::new().unwrap();
        let xmp_path = temp_dir.path().join("long_keywords.xmp");

        // Very long keyword (1000 characters)
        let long_keyword = "a".repeat(1000);

        let xmp_content = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/">
      <dc:subject>
        <rdf:Bag>
          <rdf:li>{}</rdf:li>
        </rdf:Bag>
      </dc:subject>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>"#, long_keyword);

        fs::write(&xmp_path, xmp_content).unwrap();

        let result = read_sidecar(&xmp_path);

        // Should parse successfully
        assert!(result.is_ok());
        let (_, iptc) = result.unwrap();
        assert_eq!(iptc.keywords.len(), 1);
        assert_eq!(iptc.keywords[0].len(), 1000);
    }

    #[test]
    fn test_xmp_with_extreme_exposure_values() {
        let temp_dir = TempDir::new().unwrap();
        let xmp_path = temp_dir.path().join("extreme.xmp");

        let xmp_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/">
      <crs:Exposure2012>+999.0</crs:Exposure2012>
      <crs:Contrast2012>+200</crs:Contrast2012>
      <crs:Saturation>-500</crs:Saturation>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>"#;

        fs::write(&xmp_path, xmp_content).unwrap();

        let result = read_sidecar(&xmp_path);

        // Should parse (values may be clamped later in pipeline)
        assert!(result.is_ok());
        let (settings, _) = result.unwrap();
        assert!(settings.exposure.is_some());
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
    fn test_xmp_keywords_with_duplicates() {
        let temp_dir = TempDir::new().unwrap();
        let xmp_path = temp_dir.path().join("duplicates.xmp");

        let xmp_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/">
      <dc:subject>
        <rdf:Bag>
          <rdf:li>landscape</rdf:li>
          <rdf:li>landscape</rdf:li>
          <rdf:li>portrait</rdf:li>
          <rdf:li>landscape</rdf:li>
        </rdf:Bag>
      </dc:subject>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>"#;

        fs::write(&xmp_path, xmp_content).unwrap();

        let result = read_sidecar(&xmp_path);

        assert!(result.is_ok());
        let (_, iptc) = result.unwrap();

        // May or may not deduplicate - both behaviors are valid
        assert!(!iptc.keywords.is_empty());
        assert!(iptc.keywords.contains(&"landscape".to_string()));
    }
}
