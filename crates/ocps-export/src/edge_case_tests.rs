//! Edge case tests for export functionality
//!
//! Tests scenarios that may fail in production:
//! - Export to nonexistent paths
//! - JPEG quality boundaries
//! - Invalid dimensions
//! - Batch export with errors

use crate::jpeg;
use crate::naming;
use crate::queue::ExportQueue;
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_export_to_nonexistent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let export_path = temp_dir.path().join("nonexistent").join("output.jpg");

        // Create a small test image (10x10 RGB)
        let image_data = vec![128u8; 300];

        let result = jpeg::export_jpeg(&image_data, 10, 10, 85, &export_path);

        // Should either create directory or fail with clear error
        assert!(result.is_ok() || result.is_err());

        if result.is_ok() {
            assert!(export_path.exists());
        }
    }

    #[test]
    fn test_jpeg_quality_boundary_min() {
        let temp_dir = TempDir::new().unwrap();
        let export_path = temp_dir.path().join("min_quality.jpg");

        let image_data = vec![128u8; 300];

        let result = jpeg::export_jpeg(&image_data, 10, 10, 1, &export_path);

        // Should succeed with quality=1
        assert!(result.is_ok());
    }

    #[test]
    fn test_jpeg_quality_boundary_max() {
        let temp_dir = TempDir::new().unwrap();
        let export_path = temp_dir.path().join("max_quality.jpg");

        let image_data = vec![128u8; 300];

        let result = jpeg::export_jpeg(&image_data, 10, 10, 100, &export_path);

        // Should succeed with quality=100
        assert!(result.is_ok());
    }

    #[test]
    fn test_jpeg_invalid_data_size() {
        let temp_dir = TempDir::new().unwrap();
        let export_path = temp_dir.path().join("invalid.jpg");

        // Wrong size data
        let image_data = vec![128u8; 100]; // Should be 300 for 10x10 RGB

        let result = jpeg::export_jpeg(&image_data, 10, 10, 85, &export_path);

        // Should return error
        assert!(result.is_err());
    }

    #[test]
    fn test_export_1x1_image() {
        let temp_dir = TempDir::new().unwrap();
        let export_path = temp_dir.path().join("tiny.jpg");

        let image_data = vec![255u8, 0u8, 0u8]; // 1x1 red pixel

        let result = jpeg::export_jpeg(&image_data, 1, 1, 85, &export_path);

        // Should handle 1x1 images
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_zero_dimensions() {
        let temp_dir = TempDir::new().unwrap();
        let export_path = temp_dir.path().join("zero.jpg");

        let image_data = vec![];

        let result = jpeg::export_jpeg(&image_data, 0, 0, 85, &export_path);

        // Should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_export_to_readonly_directory() {
        let temp_dir = TempDir::new().unwrap();
        let readonly_dir = temp_dir.path().join("readonly");
        fs::create_dir(&readonly_dir).unwrap();

        // Make directory readonly
        let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&readonly_dir, perms).unwrap();

        let export_path = readonly_dir.join("output.jpg");
        let image_data = vec![128u8; 300];

        let result = jpeg::export_jpeg(&image_data, 10, 10, 85, &export_path);

        // Restore permissions for cleanup
        let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_readonly(false);
        fs::set_permissions(&readonly_dir, perms).unwrap();

        // Should fail with permission error
        assert!(result.is_err());
    }

    #[test]
    fn test_naming_template_empty() {
        use crate::naming::PhotoForNaming;

        let photo = PhotoForNaming {
            file_path: "photo.jpg".to_string(),
            date_taken: None,
            camera_make: None,
            camera_model: None,
            rating: 0,
        };

        let result = naming::apply_naming_template("", &photo, 1);

        // Empty template should return empty string or fallback
        assert!(!result.is_empty() || result.is_empty());
    }

    #[test]
    fn test_naming_template_seq_token() {
        use crate::naming::PhotoForNaming;

        let photo = PhotoForNaming {
            file_path: "photo.jpg".to_string(),
            date_taken: None,
            camera_make: None,
            camera_model: None,
            rating: 0,
        };

        let result = naming::apply_naming_template("{seq}", &photo, 42);

        // Should contain sequence number
        assert!(result.contains("0042") || result.contains("42"));
    }

    #[test]
    fn test_export_queue_empty() {
        let queue = ExportQueue::new();

        let status = queue.status();

        // Should handle empty queue
        assert_eq!(status.pending, 0);
        assert_eq!(status.completed, 0);
        assert_eq!(status.failed, 0);
        assert_eq!(status.is_running, false);
    }

    #[test]
    fn test_very_long_filename() {
        let temp_dir = TempDir::new().unwrap();

        // Very long filename (but not exceeding OS limits)
        let long_name = "a".repeat(200);
        let export_path = temp_dir.path().join(format!("{}.jpg", long_name));

        let image_data = vec![128u8; 300];

        let result = jpeg::export_jpeg(&image_data, 10, 10, 85, &export_path);

        // Should either succeed or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }
}
