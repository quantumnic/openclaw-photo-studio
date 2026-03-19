//! Preview cache with LRU RAM cache and disk persistence
//!
//! Stores generated previews in memory for fast access, with LRU eviction
//! and disk-based persistence for longer-term storage.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::time::SystemTime;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid photo ID: {0}")]
    InvalidId(String),
}

/// Cached preview data
#[derive(Debug, Clone)]
pub struct CachedPreview {
    pub data_base64: String,
    pub width: u32,
    pub height: u32,
    pub generated_at: SystemTime,
}

/// Preview cache with LRU RAM cache and disk persistence
pub struct PreviewCache {
    /// RAM cache: photo_id -> preview
    ram_cache: HashMap<String, CachedPreview>,

    /// LRU tracking: most recently accessed items at the back
    access_order: VecDeque<String>,

    /// Directory for disk cache
    cache_dir: PathBuf,

    /// Maximum number of entries in RAM cache
    max_ram_entries: usize,
}

impl PreviewCache {
    /// Create a new preview cache
    ///
    /// # Arguments
    /// - `cache_dir`: Directory to store disk cache files
    /// - `max_ram_entries`: Maximum number of previews to keep in RAM
    pub fn new(cache_dir: PathBuf, max_ram_entries: usize) -> Self {
        // Ensure cache directory exists
        let _ = std::fs::create_dir_all(&cache_dir);

        Self {
            ram_cache: HashMap::new(),
            access_order: VecDeque::new(),
            cache_dir,
            max_ram_entries,
        }
    }

    /// Get a preview from cache
    ///
    /// Checks RAM cache first, then disk cache. Updates LRU order.
    pub fn get(&mut self, photo_id: &str) -> Option<&CachedPreview> {
        // Check RAM cache first
        if self.ram_cache.contains_key(photo_id) {
            // Update LRU: move to back (most recent)
            self.access_order.retain(|id| id != photo_id);
            self.access_order.push_back(photo_id.to_string());

            return self.ram_cache.get(photo_id);
        }

        // Try disk cache
        if let Ok(preview) = self.load_from_disk(photo_id) {
            // Insert into RAM cache
            self.put(photo_id, preview);
            return self.ram_cache.get(photo_id);
        }

        None
    }

    /// Store a preview in cache
    ///
    /// Adds to RAM cache and disk cache. Evicts LRU entry if RAM is full.
    pub fn put(&mut self, photo_id: &str, preview: CachedPreview) {
        // Check if we need to evict
        if self.ram_cache.len() >= self.max_ram_entries && !self.ram_cache.contains_key(photo_id) {
            self.evict_lru();
        }

        // Insert into RAM cache
        self.ram_cache.insert(photo_id.to_string(), preview.clone());

        // Update LRU order
        self.access_order.retain(|id| id != photo_id);
        self.access_order.push_back(photo_id.to_string());

        // Save to disk
        let _ = self.save_to_disk(photo_id, &preview);
    }

    /// Invalidate a preview (remove from RAM and disk)
    pub fn invalidate(&mut self, photo_id: &str) {
        // Remove from RAM cache
        self.ram_cache.remove(photo_id);

        // Remove from LRU order
        self.access_order.retain(|id| id != photo_id);

        // Remove from disk
        let disk_path = self.disk_path(photo_id);
        let _ = std::fs::remove_file(disk_path);
    }

    /// Calculate total disk cache size in bytes
    pub fn disk_cache_size_bytes(&self) -> u64 {
        let mut total = 0u64;

        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    total += metadata.len();
                }
            }
        }

        total
    }

    /// Get the number of entries in RAM cache
    pub fn ram_entry_count(&self) -> usize {
        self.ram_cache.len()
    }

    /// Evict the least recently used entry from RAM cache
    fn evict_lru(&mut self) {
        if let Some(lru_id) = self.access_order.pop_front() {
            self.ram_cache.remove(&lru_id);
            // Note: We don't delete from disk - disk cache is persistent
        }
    }

    /// Get disk path for a photo ID
    fn disk_path(&self, photo_id: &str) -> PathBuf {
        // Sanitize photo_id to be filesystem-safe
        let safe_id = photo_id.replace('/', "_").replace('\\', "_");
        self.cache_dir.join(format!("{}.jpg", safe_id))
    }

    /// Save preview to disk
    fn save_to_disk(&self, photo_id: &str, preview: &CachedPreview) -> Result<(), CacheError> {
        let path = self.disk_path(photo_id);

        // Decode base64 to bytes
        let jpeg_data = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &preview.data_base64,
        )
        .map_err(|e| CacheError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Base64 decode error: {}", e),
        )))?;

        std::fs::write(path, jpeg_data)?;
        Ok(())
    }

    /// Load preview from disk
    fn load_from_disk(&self, photo_id: &str) -> Result<CachedPreview, CacheError> {
        let path = self.disk_path(photo_id);

        if !path.exists() {
            return Err(CacheError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Cache file not found",
            )));
        }

        // Read JPEG data
        let jpeg_data = std::fs::read(&path)?;

        // Decode JPEG to get dimensions
        let img = image::load_from_memory(&jpeg_data)
            .map_err(|e| CacheError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid JPEG: {}", e),
            )))?;

        // Encode back to base64
        let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &jpeg_data);

        Ok(CachedPreview {
            data_base64: base64_data,
            width: img.width(),
            height: img.height(),
            generated_at: SystemTime::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_preview() -> CachedPreview {
        // Create a minimal base64-encoded JPEG (1x1 pixel)
        let jpeg_bytes = vec![
            0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
            0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43,
            0x00, 0x08, 0x06, 0x06, 0x07, 0x06, 0x05, 0x08, 0x07, 0x07, 0x07, 0x09,
            0x09, 0x08, 0x0A, 0x0C, 0x14, 0x0D, 0x0C, 0x0B, 0x0B, 0x0C, 0x19, 0x12,
            0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D, 0x1A, 0x1C, 0x1C, 0x20,
            0x24, 0x2E, 0x27, 0x20, 0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28, 0x37, 0x29,
            0x2C, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1F, 0x27, 0x39, 0x3D, 0x38, 0x32,
            0x3C, 0x2E, 0x33, 0x34, 0x32, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01,
            0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4, 0x00, 0x14, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x03, 0xFF, 0xC4, 0x00, 0x14, 0x10, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3F, 0x00,
            0xFE, 0x8A, 0x28, 0xFF, 0xD9,
        ];

        CachedPreview {
            data_base64: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &jpeg_bytes),
            width: 100,
            height: 100,
            generated_at: SystemTime::now(),
        }
    }

    #[test]
    fn test_cache_put_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = PreviewCache::new(temp_dir.path().to_path_buf(), 10);

        let preview = create_test_preview();
        cache.put("photo1", preview.clone());

        let retrieved = cache.get("photo1");
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.width, preview.width);
        assert_eq!(retrieved.height, preview.height);
    }

    #[test]
    fn test_cache_get_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = PreviewCache::new(temp_dir.path().to_path_buf(), 10);

        let result = cache.get("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_lru_eviction() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = PreviewCache::new(temp_dir.path().to_path_buf(), 3);

        // Insert 3 entries (at max capacity)
        cache.put("photo1", create_test_preview());
        cache.put("photo2", create_test_preview());
        cache.put("photo3", create_test_preview());

        assert_eq!(cache.ram_entry_count(), 3);

        // Insert 4th entry - should evict photo1 (least recently used)
        cache.put("photo4", create_test_preview());

        assert_eq!(cache.ram_entry_count(), 3);
        assert!(cache.ram_cache.get("photo1").is_none());
        assert!(cache.ram_cache.get("photo4").is_some());
    }

    #[test]
    fn test_cache_invalidate() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = PreviewCache::new(temp_dir.path().to_path_buf(), 10);

        let preview = create_test_preview();
        cache.put("photo1", preview);

        assert!(cache.get("photo1").is_some());

        cache.invalidate("photo1");

        assert!(cache.get("photo1").is_none());
        assert_eq!(cache.ram_entry_count(), 0);
    }

    #[test]
    fn test_cache_disk_persistence() {
        let temp_dir = TempDir::new().unwrap();

        // Create cache, add entry, drop it
        {
            let mut cache = PreviewCache::new(temp_dir.path().to_path_buf(), 1);
            cache.put("photo1", create_test_preview());
        }

        // Create new cache instance - should load from disk
        {
            let mut cache = PreviewCache::new(temp_dir.path().to_path_buf(), 1);
            let retrieved = cache.get("photo1");
            assert!(retrieved.is_some());
        }
    }

    #[test]
    fn test_cache_disk_size() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = PreviewCache::new(temp_dir.path().to_path_buf(), 10);

        cache.put("photo1", create_test_preview());
        cache.put("photo2", create_test_preview());

        let size = cache.disk_cache_size_bytes();
        assert!(size > 0);
    }

    #[test]
    fn test_cache_access_updates_lru() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = PreviewCache::new(temp_dir.path().to_path_buf(), 3);

        cache.put("photo1", create_test_preview());
        cache.put("photo2", create_test_preview());
        cache.put("photo3", create_test_preview());

        // Access photo1 - should move to back
        let _ = cache.get("photo1");

        // Add photo4 - should evict photo2 (now LRU), not photo1
        cache.put("photo4", create_test_preview());

        assert!(cache.ram_cache.get("photo1").is_some());
        assert!(cache.ram_cache.get("photo2").is_none());
        assert!(cache.ram_cache.get("photo4").is_some());
    }
}
