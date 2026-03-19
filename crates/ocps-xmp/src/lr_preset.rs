//! Lightroom preset import (.lrtemplate and .xmp)
//!
//! Parses Lightroom .lrtemplate files (Lua-like format) and .xmp presets.

use crate::{XmpDevelopSettings, XmpError};
use std::path::Path;

/// Parse a Lightroom .lrtemplate file (Lua-like format)
///
/// Example format:
/// ```lua
/// s = {
///   id = "...",
///   internalName = "My Preset",
///   title = "My Preset",
///   settings = {
///     Exposure2012 = 0.5,
///     Contrast2012 = 20,
///     ...
///   }
/// }
/// ```
pub fn parse_lrtemplate(content: &str) -> Result<XmpDevelopSettings, XmpError> {
    let mut settings = XmpDevelopSettings::default();

    // Find the settings block
    let settings_start = content
        .find("settings")
        .ok_or_else(|| XmpError::Parse("No settings block found".to_string()))?;

    let settings_block_start = content[settings_start..]
        .find('{')
        .ok_or_else(|| XmpError::Parse("Settings block not opened".to_string()))?;

    let settings_content = &content[settings_start + settings_block_start..];

    // Parse key-value pairs
    for line in settings_content.lines() {
        let line = line.trim();

        // Parse lines like: Exposure2012 = 0.5,
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value_part = line[eq_pos + 1..].trim();
            let value = value_part.trim_end_matches(',').trim();

            match key {
                "Exposure2012" => {
                    if let Ok(v) = value.parse::<f32>() {
                        settings.exposure = Some(v);
                    }
                }
                "Contrast2012" => {
                    if let Ok(v) = value.parse::<i32>() {
                        settings.contrast = Some(v);
                    }
                }
                "Temperature" => {
                    if let Ok(v) = value.parse::<i32>() {
                        settings.temperature = Some(v);
                    }
                }
                "Tint" => {
                    if let Ok(v) = value.parse::<i32>() {
                        settings.tint = Some(v);
                    }
                }
                "Highlights2012" => {
                    if let Ok(v) = value.parse::<i32>() {
                        settings.highlights = Some(v);
                    }
                }
                "Shadows2012" => {
                    if let Ok(v) = value.parse::<i32>() {
                        settings.shadows = Some(v);
                    }
                }
                "Whites2012" => {
                    if let Ok(v) = value.parse::<i32>() {
                        settings.whites = Some(v);
                    }
                }
                "Blacks2012" => {
                    if let Ok(v) = value.parse::<i32>() {
                        settings.blacks = Some(v);
                    }
                }
                "Clarity2012" => {
                    if let Ok(v) = value.parse::<i32>() {
                        settings.clarity = Some(v);
                    }
                }
                "Dehaze" => {
                    if let Ok(v) = value.parse::<i32>() {
                        settings.dehaze = Some(v);
                    }
                }
                "Vibrance" => {
                    if let Ok(v) = value.parse::<i32>() {
                        settings.vibrance = Some(v);
                    }
                }
                "Saturation" => {
                    if let Ok(v) = value.parse::<i32>() {
                        settings.saturation = Some(v);
                    }
                }
                "ProcessVersion" => {
                    settings.process_version = Some(value.trim_matches('"').to_string());
                }
                _ => {}
            }
        }
    }

    Ok(settings)
}

/// Import a preset file (.lrtemplate or .xmp)
///
/// Returns (preset_name, settings)
pub fn import_preset_file(path: &Path) -> Result<(String, XmpDevelopSettings), XmpError> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| XmpError::Parse("No file extension".to_string()))?;

    let content = std::fs::read_to_string(path)?;

    match extension.to_lowercase().as_str() {
        "lrtemplate" => {
            // Parse Lua-like format
            let settings = parse_lrtemplate(&content)?;

            // Extract preset name from title or internalName
            let name = extract_preset_name(&content).unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled Preset")
                    .to_string()
            });

            Ok((name, settings))
        }
        "xmp" => {
            // Parse XMP format using existing reader
            let (settings, _iptc) = crate::reader::read_sidecar(path)?;

            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled Preset")
                .to_string();

            Ok((name, settings))
        }
        _ => Err(XmpError::Parse(format!(
            "Unsupported preset format: {}",
            extension
        ))),
    }
}

/// Extract preset name from .lrtemplate content
fn extract_preset_name(content: &str) -> Option<String> {
    // Look for title = "..."
    if let Some(title_start) = content.find("title") {
        let after_title = &content[title_start..];
        if let Some(eq_pos) = after_title.find('=') {
            let value_part = &after_title[eq_pos + 1..];
            if let Some(start_quote) = value_part.find('"') {
                let after_quote = &value_part[start_quote + 1..];
                if let Some(end_quote) = after_quote.find('"') {
                    return Some(after_quote[..end_quote].to_string());
                }
            }
        }
    }

    // Fallback to internalName
    if let Some(internal_start) = content.find("internalName") {
        let after_internal = &content[internal_start..];
        if let Some(eq_pos) = after_internal.find('=') {
            let value_part = &after_internal[eq_pos + 1..];
            if let Some(start_quote) = value_part.find('"') {
                let after_quote = &value_part[start_quote + 1..];
                if let Some(end_quote) = after_quote.find('"') {
                    return Some(after_quote[..end_quote].to_string());
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lrtemplate_basic() {
        let content = r#"
s = {
  id = "test",
  internalName = "Test Preset",
  title = "Test Preset",
  settings = {
    Exposure2012 = 0.5,
    Contrast2012 = 20,
  }
}
"#;

        let settings = parse_lrtemplate(content).unwrap();
        assert_eq!(settings.exposure, Some(0.5));
        assert_eq!(settings.contrast, Some(20));
    }

    #[test]
    fn test_parse_lrtemplate_all_fields() {
        let content = r#"
s = {
  settings = {
    Exposure2012 = 0.5,
    Contrast2012 = 20,
    Temperature = 5500,
    Tint = 10,
    Highlights2012 = -50,
    Shadows2012 = 30,
    Whites2012 = 15,
    Blacks2012 = -10,
    Clarity2012 = 25,
    Dehaze = 10,
    Vibrance = 15,
    Saturation = 5,
    ProcessVersion = "11.0",
  }
}
"#;

        let settings = parse_lrtemplate(content).unwrap();
        assert_eq!(settings.exposure, Some(0.5));
        assert_eq!(settings.contrast, Some(20));
        assert_eq!(settings.temperature, Some(5500));
        assert_eq!(settings.tint, Some(10));
        assert_eq!(settings.highlights, Some(-50));
        assert_eq!(settings.shadows, Some(30));
        assert_eq!(settings.whites, Some(15));
        assert_eq!(settings.blacks, Some(-10));
        assert_eq!(settings.clarity, Some(25));
        assert_eq!(settings.dehaze, Some(10));
        assert_eq!(settings.vibrance, Some(15));
        assert_eq!(settings.saturation, Some(5));
        assert_eq!(settings.process_version.as_deref(), Some("11.0"));
    }

    #[test]
    fn test_parse_lrtemplate_missing_settings() {
        let content = r#"
s = {
  id = "test",
}
"#;

        let result = parse_lrtemplate(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_preset_name_from_title() {
        let content = r#"
s = {
  title = "My Cool Preset",
  internalName = "internal",
}
"#;

        let name = extract_preset_name(content);
        assert_eq!(name, Some("My Cool Preset".to_string()));
    }

    #[test]
    fn test_extract_preset_name_from_internal() {
        let content = r#"
s = {
  internalName = "Internal Preset",
}
"#;

        let name = extract_preset_name(content);
        assert_eq!(name, Some("Internal Preset".to_string()));
    }

    #[test]
    fn test_import_preset_file_unsupported_extension() {
        let temp_dir = std::env::temp_dir();
        let preset_path = temp_dir.join("test.txt");
        std::fs::write(&preset_path, "test").unwrap();

        let result = import_preset_file(&preset_path);
        assert!(result.is_err());

        let _ = std::fs::remove_file(preset_path);
    }
}
