//! EXIF metadata extraction

use crate::XmpError;
use std::path::Path;

/// EXIF data extracted from image file
#[derive(Debug, Clone, Default)]
pub struct ExifData {
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens: Option<String>,
    pub focal_length: Option<f64>,
    pub aperture: Option<f64>,
    pub shutter_speed: Option<String>,
    pub iso: Option<u32>,
    pub date_taken: Option<String>,
    pub gps_lat: Option<f64>,
    pub gps_lon: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub orientation: Option<u32>,
}

/// Read EXIF data from an image file
pub fn read_exif(path: &Path) -> Result<ExifData, XmpError> {
    let file = std::fs::File::open(path)
        .map_err(|e| XmpError::Parse(format!("Failed to open file: {}", e)))?;
    let mut bufreader = std::io::BufReader::new(&file);

    let exif_reader = exif::Reader::new();
    let exif = exif_reader
        .read_from_container(&mut bufreader)
        .map_err(|e| XmpError::Parse(format!("Failed to read EXIF: {}", e)))?;

    let mut data = ExifData::default();

    // Extract camera make
    if let Some(field) = exif.get_field(exif::Tag::Make, exif::In::PRIMARY) {
        data.camera_make = field.display_value().to_string().into();
    }

    // Extract camera model
    if let Some(field) = exif.get_field(exif::Tag::Model, exif::In::PRIMARY) {
        data.camera_model = field.display_value().to_string().into();
    }

    // Extract lens
    if let Some(field) = exif.get_field(exif::Tag::LensModel, exif::In::PRIMARY) {
        data.lens = field.display_value().to_string().into();
    }

    // Extract focal length
    if let Some(field) = exif.get_field(exif::Tag::FocalLength, exif::In::PRIMARY) {
        if let exif::Value::Rational(ref values) = field.value {
            if let Some(val) = values.first() {
                data.focal_length = Some(val.num as f64 / val.denom as f64);
            }
        }
    }

    // Extract aperture (F-number)
    if let Some(field) = exif.get_field(exif::Tag::FNumber, exif::In::PRIMARY) {
        if let exif::Value::Rational(ref values) = field.value {
            if let Some(val) = values.first() {
                data.aperture = Some(val.num as f64 / val.denom as f64);
            }
        }
    }

    // Extract ISO
    if let Some(field) = exif.get_field(exif::Tag::PhotographicSensitivity, exif::In::PRIMARY) {
        if let exif::Value::Short(ref values) = field.value {
            if let Some(val) = values.first() {
                data.iso = Some(*val as u32);
            }
        }
    }

    // Extract shutter speed
    if let Some(field) = exif.get_field(exif::Tag::ExposureTime, exif::In::PRIMARY) {
        data.shutter_speed = Some(field.display_value().to_string());
    }

    // Extract date taken
    if let Some(field) = exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
        data.date_taken = Some(field.display_value().to_string());
    }

    // Extract image dimensions
    if let Some(field) = exif.get_field(exif::Tag::PixelXDimension, exif::In::PRIMARY) {
        if let exif::Value::Long(ref values) = field.value {
            if let Some(val) = values.first() {
                data.width = Some(*val);
            }
        }
    }

    if let Some(field) = exif.get_field(exif::Tag::PixelYDimension, exif::In::PRIMARY) {
        if let exif::Value::Long(ref values) = field.value {
            if let Some(val) = values.first() {
                data.height = Some(*val);
            }
        }
    }

    // Extract orientation
    if let Some(field) = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY) {
        if let exif::Value::Short(ref values) = field.value {
            if let Some(val) = values.first() {
                data.orientation = Some(*val as u32);
            }
        }
    }

    // Extract GPS coordinates
    if let Some(lat_field) = exif.get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY) {
        if let Some(lat_ref_field) = exif.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY) {
            if let exif::Value::Rational(ref values) = lat_field.value {
                if values.len() >= 3 {
                    let degrees = values[0].num as f64 / values[0].denom as f64;
                    let minutes = values[1].num as f64 / values[1].denom as f64;
                    let seconds = values[2].num as f64 / values[2].denom as f64;
                    let mut lat = degrees + minutes / 60.0 + seconds / 3600.0;

                    // Check latitude reference (N/S)
                    if let exif::Value::Ascii(ref ref_val) = lat_ref_field.value {
                        if let Some(first) = ref_val.first() {
                            if first == b"S" {
                                lat = -lat;
                            }
                        }
                    }
                    data.gps_lat = Some(lat);
                }
            }
        }
    }

    if let Some(lon_field) = exif.get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY) {
        if let Some(lon_ref_field) = exif.get_field(exif::Tag::GPSLongitudeRef, exif::In::PRIMARY)
        {
            if let exif::Value::Rational(ref values) = lon_field.value {
                if values.len() >= 3 {
                    let degrees = values[0].num as f64 / values[0].denom as f64;
                    let minutes = values[1].num as f64 / values[1].denom as f64;
                    let seconds = values[2].num as f64 / values[2].denom as f64;
                    let mut lon = degrees + minutes / 60.0 + seconds / 3600.0;

                    // Check longitude reference (E/W)
                    if let exif::Value::Ascii(ref ref_val) = lon_ref_field.value {
                        if let Some(first) = ref_val.first() {
                            if first == b"W" {
                                lon = -lon;
                            }
                        }
                    }
                    data.gps_lon = Some(lon);
                }
            }
        }
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_exif_missing_file() {
        let result = read_exif(Path::new("/nonexistent/file.jpg"));
        assert!(result.is_err());
    }
}
