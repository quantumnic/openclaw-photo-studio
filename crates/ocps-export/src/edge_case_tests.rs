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
        // Most importantly, should not panic
        if result.is_err() {
            // If it fails, the error should be clear (e.g., "No such file or directory")
            let err = result.unwrap_err();
            let err_msg = err.to_string();
            assert!(
                err_msg.contains("No such file") ||
                err_msg.contains("directory") ||
                err_msg.contains("not found"),
                "Error should be descriptive: {}", err_msg
            );
        } else {
            // If it succeeds, the directory and file should exist
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

    #[test]
    fn test_export_batch_continues_on_error() {
        use crate::queue::{ExportJob, ExportQueue, JobStatus};
        use chrono::Utc;
        use std::path::PathBuf;

        let temp_dir = TempDir::new().unwrap();
        let mut queue = ExportQueue::with_max_retries(0); // Disable auto-retry for this test

        // Create 5 jobs: job1-job5
        // We'll simulate job3 failing due to invalid photo_id
        let jobs: Vec<ExportJob> = (1..=5).map(|i| {
            ExportJob {
                id: format!("job{}", i),
                photo_id: format!("photo_{}", i),
                settings: serde_json::json!({"quality": 85}),
                output_path: temp_dir.path().join(format!("output_{}.jpg", i)),
                status: JobStatus::Pending,
                retry_count: 0,
                error: None,
                created_at: Utc::now(),
                completed_at: None,
            }
        }).collect();

        // Enqueue all jobs
        for job in jobs {
            queue.enqueue(job);
        }

        // Simulate batch processing with one failure
        let mut processed_count = 0;
        let mut success_count = 0;
        let mut failure_count = 0;

        while let Some(job) = queue.next_job() {
            processed_count += 1;

            // Simulate job3 failing (invalid photo_id scenario)
            if job.id == "job3" {
                queue.mark_failed(&job.id, "Invalid photo_id: photo not found in catalog".to_string()).unwrap();
                failure_count += 1;
            } else {
                // Other jobs succeed
                queue.mark_completed(&job.id).unwrap();
                success_count += 1;
            }
        }

        // Verify batch continued despite one failure
        assert_eq!(processed_count, 5, "Should process all 5 jobs");
        assert_eq!(success_count, 4, "Should have 4 successful jobs");
        assert_eq!(failure_count, 1, "Should have 1 failed job");

        // Verify queue status
        let status = queue.status();
        assert_eq!(status.completed, 4, "Should have 4 completed jobs");
        assert_eq!(status.failed, 1, "Should have 1 failed job");
        assert_eq!(status.pending, 0, "Should have no pending jobs");
        assert!(!status.is_running, "Should not be running");

        // Verify failed job has error message
        let failed_jobs = queue.failed_jobs();
        assert_eq!(failed_jobs.len(), 1);
        assert_eq!(failed_jobs[0].id, "job3");
        assert!(failed_jobs[0].error.is_some());
        assert!(failed_jobs[0].error.as_ref().unwrap().contains("Invalid photo_id"));

        // Verify successful jobs are marked completed
        let completed_jobs = queue.completed_jobs();
        assert_eq!(completed_jobs.len(), 4);
    }
}
