//! Export naming template engine
//!
//! Supported tokens:
//! - {original} - file_name without extension
//! - {date} - date_taken formatted as YYYY-MM-DD
//! - {year} - year from date_taken
//! - {month} - month as 2-digit
//! - {day} - day as 2-digit
//! - {camera} - camera_make + camera_model (no spaces)
//! - {rating} - rating digit
//! - {seq} - sequence number zero-padded to 4 digits (0001, 0002...)
//! - {seq:3} - sequence with custom width
//! - Literal text outside braces is preserved

use chrono::NaiveDateTime;
use std::path::Path;

/// Photo metadata for naming template
pub struct PhotoForNaming {
    pub file_path: String,
    pub date_taken: Option<String>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub rating: u8,
}

/// Apply a naming template to generate an output filename
///
/// # Arguments
/// * `template` - The template string with tokens like {original}, {date}, etc.
/// * `photo` - Photo metadata
/// * `sequence` - Sequence number (1-based)
///
/// # Returns
/// The generated filename (without extension)
pub fn apply_naming_template(template: &str, photo: &PhotoForNaming, sequence: u32) -> String {
    let mut result = template.to_string();

    // Extract original filename without extension
    let original = Path::new(&photo.file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("photo");

    result = result.replace("{original}", original);

    // Date tokens
    if let Some(ref date_str) = photo.date_taken {
        // Try to parse the date
        if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.fZ") {
            result = result.replace("{date}", &dt.format("%Y-%m-%d").to_string());
            result = result.replace("{year}", &dt.format("%Y").to_string());
            result = result.replace("{month}", &dt.format("%m").to_string());
            result = result.replace("{day}", &dt.format("%d").to_string());
        } else if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
            result = result.replace("{date}", &dt.format("%Y-%m-%d").to_string());
            result = result.replace("{year}", &dt.format("%Y").to_string());
            result = result.replace("{month}", &dt.format("%m").to_string());
            result = result.replace("{day}", &dt.format("%d").to_string());
        } else {
            // Fallback: just use placeholders
            result = result.replace("{date}", "unknown");
            result = result.replace("{year}", "unknown");
            result = result.replace("{month}", "unknown");
            result = result.replace("{day}", "unknown");
        }
    } else {
        result = result.replace("{date}", "unknown");
        result = result.replace("{year}", "unknown");
        result = result.replace("{month}", "unknown");
        result = result.replace("{day}", "unknown");
    }

    // Camera tokens
    let camera = match (&photo.camera_make, &photo.camera_model) {
        (Some(make), Some(model)) => {
            format!("{}{}", make, model).replace(' ', "")
        }
        (Some(make), None) => make.replace(' ', ""),
        (None, Some(model)) => model.replace(' ', ""),
        _ => "Unknown".to_string(),
    };
    result = result.replace("{camera}", &camera);

    // Rating
    result = result.replace("{rating}", &photo.rating.to_string());

    // Sequence with custom width - check for {seq:N} pattern
    if result.contains("{seq:") {
        // Find all {seq:N} patterns and replace them
        let mut new_result = String::new();
        let mut chars = result.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' {
                // Check if this is a seq token
                let mut ahead = String::new();
                let mut temp_chars: Vec<char> = Vec::new();

                // Look ahead to see if this is {seq:N}
                while let Some(&next_char) = chars.peek() {
                    temp_chars.push(next_char);
                    ahead.push(next_char);
                    chars.next();

                    if next_char == '}' {
                        break;
                    }

                    if ahead.len() > 10 {
                        // Not a valid seq token
                        break;
                    }
                }

                if ahead.starts_with("seq:") && ahead.ends_with('}') {
                    // Extract width
                    let width_str = ahead.trim_start_matches("seq:").trim_end_matches('}');
                    if let Ok(width) = width_str.parse::<usize>() {
                        let formatted = format!("{:0width$}", sequence, width = width);
                        new_result.push_str(&formatted);
                    } else {
                        // Invalid width, keep original
                        new_result.push('{');
                        new_result.push_str(&ahead);
                    }
                } else {
                    // Not a seq token, restore characters
                    new_result.push('{');
                    new_result.push_str(&ahead);
                }
            } else {
                new_result.push(c);
            }
        }

        result = new_result;
    }

    // Default {seq} with 4-digit padding
    result = result.replace("{seq}", &format!("{:04}", sequence));

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_photo() -> PhotoForNaming {
        PhotoForNaming {
            file_path: "/path/to/DSC_4523.ARW".to_string(),
            date_taken: Some("2026-03-15T14:30:00.000Z".to_string()),
            camera_make: Some("Sony".to_string()),
            camera_model: Some("A7 III".to_string()),
            rating: 4,
        }
    }

    #[test]
    fn test_template_original() {
        let photo = test_photo();
        let result = apply_naming_template("{original}", &photo, 1);
        assert_eq!(result, "DSC_4523");
    }

    #[test]
    fn test_template_date() {
        let photo = test_photo();
        let result = apply_naming_template("{date}", &photo, 1);
        assert_eq!(result, "2026-03-15");
    }

    #[test]
    fn test_template_date_components() {
        let photo = test_photo();
        let result = apply_naming_template("{year}-{month}-{day}", &photo, 1);
        assert_eq!(result, "2026-03-15");
    }

    #[test]
    fn test_template_sequence() {
        let photo = test_photo();
        let result = apply_naming_template("{seq}", &photo, 42);
        assert_eq!(result, "0042");
    }

    #[test]
    fn test_template_sequence_custom_width() {
        let photo = test_photo();
        let result = apply_naming_template("{seq:3}", &photo, 42);
        assert_eq!(result, "042");
    }

    #[test]
    fn test_template_sequence_custom_width_larger() {
        let photo = test_photo();
        let result = apply_naming_template("{seq:6}", &photo, 42);
        assert_eq!(result, "000042");
    }

    #[test]
    fn test_template_complex() {
        let photo = test_photo();
        let result = apply_naming_template("IMG_{date}_{seq:3}", &photo, 42);
        assert_eq!(result, "IMG_2026-03-15_042");
    }

    #[test]
    fn test_template_camera() {
        let photo = test_photo();
        let result = apply_naming_template("{camera}_{original}", &photo, 1);
        assert_eq!(result, "SonyA7III_DSC_4523");
    }

    #[test]
    fn test_template_rating() {
        let photo = test_photo();
        let result = apply_naming_template("{rating}star_{original}", &photo, 1);
        assert_eq!(result, "4star_DSC_4523");
    }

    #[test]
    fn test_template_unknown_token() {
        let photo = test_photo();
        let result = apply_naming_template("{original}_{unknown}", &photo, 1);
        assert_eq!(result, "DSC_4523_{unknown}");
    }

    #[test]
    fn test_template_no_date() {
        let mut photo = test_photo();
        photo.date_taken = None;
        let result = apply_naming_template("{date}_{original}", &photo, 1);
        assert_eq!(result, "unknown_DSC_4523");
    }

    #[test]
    fn test_template_no_camera() {
        let mut photo = test_photo();
        photo.camera_make = None;
        photo.camera_model = None;
        let result = apply_naming_template("{camera}_{original}", &photo, 1);
        assert_eq!(result, "Unknown_DSC_4523");
    }

    #[test]
    fn test_template_literal_text() {
        let photo = test_photo();
        let result = apply_naming_template("Wedding_{date}_{seq}", &photo, 5);
        assert_eq!(result, "Wedding_2026-03-15_0005");
    }
}
