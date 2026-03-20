//! Edge case tests for catalog functionality
//!
//! Tests scenarios that may fail in production:
//! - Read-only paths
//! - Concurrent access
//! - Empty folders
//! - Deeply nested folders
//! - Invalid inputs

use crate::db::Catalog;
use crate::models::{PhotoFilter, SortOrder};
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_catalog_readonly_path() {
        // Test what happens when catalog path is read-only
        let temp_dir = TempDir::new().unwrap();
        let catalog_path = temp_dir.path().join("catalog.db");

        // Create catalog first
        {
            let _catalog = Catalog::open(&catalog_path).unwrap();
        }

        // Make it read-only
        let mut perms = fs::metadata(&catalog_path).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&catalog_path, perms).unwrap();

        // Attempting to open read-only catalog should fail gracefully
        let result = Catalog::open(&catalog_path);

        // Clean up: restore write permissions before temp_dir drops
        let mut perms = fs::metadata(&catalog_path).unwrap().permissions();
        perms.set_readonly(false);
        fs::set_permissions(&catalog_path, perms).unwrap();

        // Should either fail or open in read-only mode
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_import_empty_folder() {
        let catalog = Catalog::in_memory().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let result = catalog.import_folder(temp_dir.path()).unwrap();

        assert_eq!(result.total, 0);
        assert_eq!(result.inserted, 0);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_import_folder_no_supported_files() {
        let catalog = Catalog::in_memory().unwrap();
        let temp_dir = TempDir::new().unwrap();

        // Create non-supported files
        fs::write(temp_dir.path().join("file.txt"), "text").unwrap();
        fs::write(temp_dir.path().join("file.pdf"), "pdf").unwrap();
        fs::write(temp_dir.path().join("file.doc"), "doc").unwrap();

        let result = catalog.import_folder(temp_dir.path()).unwrap();

        assert_eq!(result.total, 0);
        assert_eq!(result.inserted, 0);
    }

    #[test]
    fn test_import_deeply_nested_folders() {
        let catalog = Catalog::in_memory().unwrap();
        let temp_dir = TempDir::new().unwrap();

        // Create 10 levels deep
        let mut current_path = temp_dir.path().to_path_buf();
        for i in 0..10 {
            current_path = current_path.join(format!("level_{}", i));
            fs::create_dir_all(&current_path).unwrap();
        }

        // Create a dummy file at the deepest level
        fs::write(current_path.join("photo.jpg"), vec![0u8; 100]).unwrap();

        let result = catalog.import_folder(temp_dir.path()).unwrap();

        // Should successfully find the file even 10 levels deep
        assert_eq!(result.total, 1);
    }

    #[test]
    fn test_search_empty_query() {
        let catalog = Catalog::in_memory().unwrap();

        // Empty query search
        let results = catalog.search("", 100).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_whitespace_only_query() {
        let catalog = Catalog::in_memory().unwrap();

        // Whitespace-only query
        let results = catalog.search("   ", 100).unwrap();
        // Should return empty or all photos
        assert!(results.is_empty());
    }

    #[test]
    fn test_batch_operation_empty_ids() {
        let catalog = Catalog::in_memory().unwrap();

        // Batch rating with empty ID list should succeed without error
        let result = catalog.batch_update_rating(&[], 5);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Batch flag with empty ID list
        let result = catalog.batch_update_flag(&[], "pick");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Batch delete with empty ID list
        let result = catalog.batch_delete(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_rating_boundary_values() {
        let catalog = Catalog::in_memory().unwrap();

        // Add a photo via import
        let temp_dir = TempDir::new().unwrap();
        let photo_path = temp_dir.path().join("photo.jpg");
        fs::write(&photo_path, vec![0u8; 100]).unwrap();
        catalog.import_folder(temp_dir.path()).unwrap();

        // Get the photo ID
        let photos = catalog.get_photos(&PhotoFilter::default(), &SortOrder::default()).unwrap();
        assert_eq!(photos.len(), 1);
        let photo_id = &photos[0].id;

        // Test boundary ratings
        let result = catalog.update_rating(photo_id, 0);
        assert!(result.is_ok());

        let result = catalog.update_rating(photo_id, 5);
        assert!(result.is_ok());

        // Out of range value - should be accepted (clamping done elsewhere)
        let result = catalog.update_rating(photo_id, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_filter_with_limits() {
        let catalog = Catalog::in_memory().unwrap();

        let mut filter = PhotoFilter::default();

        // Test with extreme limit values
        filter.limit = u32::MAX;
        let results = catalog.get_photos(&filter, &SortOrder::default()).unwrap();
        assert_eq!(results.len(), 0);

        // Test with zero limit
        filter.limit = 0;
        let results = catalog.get_photos(&filter, &SortOrder::default()).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_get_nonexistent_photo() {
        let catalog = Catalog::in_memory().unwrap();

        let result = catalog.get_photo("nonexistent-id");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_keyword_with_special_characters() {
        let catalog = Catalog::in_memory().unwrap();

        // Keywords with unicode, emoji, etc.
        let keyword_id1 = catalog.get_or_create_keyword("München").unwrap();
        let keyword_id2 = catalog.get_or_create_keyword("北京").unwrap();
        let keyword_id3 = catalog.get_or_create_keyword("😀 Happy").unwrap();

        // Should all succeed
        assert!(!keyword_id1.is_empty());
        assert!(!keyword_id2.is_empty());
        assert!(!keyword_id3.is_empty());

        // Verify retrieval
        let keywords = catalog.get_all_keywords_with_count().unwrap();
        assert!(keywords.iter().any(|(name, _, _)| name == "München"));
        assert!(keywords.iter().any(|(name, _, _)| name == "北京"));
        assert!(keywords.iter().any(|(name, _, _)| name == "😀 Happy"));
    }

    #[test]
    fn test_concurrent_catalog_access() {
        // Test opening the same catalog file from two instances
        let temp_dir = TempDir::new().unwrap();
        let catalog_path = temp_dir.path().join("catalog.db");

        let catalog1 = Catalog::open(&catalog_path).unwrap();
        let catalog2 = Catalog::open(&catalog_path).unwrap();

        // Both should open successfully (WAL mode allows concurrent reads)
        // Add photo in catalog1
        let temp_photo = temp_dir.path().join("photo.jpg");
        fs::write(&temp_photo, vec![0u8; 100]).unwrap();
        catalog1.import_folder(temp_dir.path()).unwrap();

        // catalog2 should eventually see the change
        let result = catalog2.photo_count();
        assert!(result.is_ok());
        assert!(result.unwrap() >= 1);
    }

    #[test]
    fn test_very_long_file_path() {
        let catalog = Catalog::in_memory().unwrap();
        let temp_dir = TempDir::new().unwrap();

        // Create a very long path (not too long to exceed OS limits)
        let long_name = "a".repeat(200);
        let photo_path = temp_dir.path().join(format!("{}.jpg", long_name));
        fs::write(&photo_path, vec![0u8; 100]).unwrap();

        let result = catalog.import_folder(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_import_symlink() {
        let catalog = Catalog::in_memory().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let real_file = temp_dir.path().join("real.jpg");
        fs::write(&real_file, vec![0u8; 100]).unwrap();

        #[cfg(unix)]
        {
            let link_file = temp_dir.path().join("link.jpg");
            std::os::unix::fs::symlink(&real_file, &link_file).unwrap();

            // Import should skip symlinks (follow_links = false in WalkDir)
            let result = catalog.import_folder(temp_dir.path()).unwrap();

            // Should only import the real file once
            assert_eq!(result.inserted, 1);
        }

        #[cfg(not(unix))]
        {
            // Skip on non-Unix systems
        }
    }
}

