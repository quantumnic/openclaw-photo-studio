//! Plugin Marketplace
//!
//! Simple plugin registry/marketplace system for discovering and installing plugins.

use std::path::{Path, PathBuf};
use wat;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MarketplaceError {
    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Invalid plugin: {0}")]
    InvalidPlugin(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: {0}")]
    Network(String),
}

/// A plugin available in the marketplace
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarketplacePlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub download_url: String,
    pub checksum_sha256: String,
    pub api_version: u32,
    pub plugin_type: String,
    pub downloads: u64,
    pub rating: f32,
}

/// Plugin marketplace client
pub struct Marketplace {
    pub base_url: String,
    pub plugins: Vec<MarketplacePlugin>,
}

impl Default for Marketplace {
    fn default() -> Self {
        Self::new()
    }
}

impl Marketplace {
    /// Create a new marketplace client
    pub fn new() -> Self {
        Self {
            base_url: "https://marketplace.openclaw.photo/api/v1".into(),
            plugins: vec![],
        }
    }

    /// Fetch available plugins from the marketplace
    ///
    /// In production: HTTP GET to base_url/plugins
    /// For now: returns hardcoded demo plugins
    pub fn fetch_plugins(&mut self) -> Result<usize, MarketplaceError> {
        // Demo plugins (simulate what marketplace would return)
        self.plugins = vec![
            MarketplacePlugin {
                id: "community.lut-loader".into(),
                name: "LUT Loader".into(),
                version: "1.0.0".into(),
                description: "Load and apply .cube LUT files".into(),
                author: "Community".into(),
                download_url: "https://marketplace.openclaw.photo/plugins/lut-loader-1.0.0.wasm"
                    .into(),
                checksum_sha256: "abc123def456".into(),
                api_version: 1,
                plugin_type: "image_filter".into(),
                downloads: 1234,
                rating: 4.7,
            },
            MarketplacePlugin {
                id: "community.flickr-upload".into(),
                name: "Flickr Upload".into(),
                version: "2.1.0".into(),
                description: "Export photos directly to Flickr".into(),
                author: "Community".into(),
                download_url: "https://marketplace.openclaw.photo/plugins/flickr-2.1.0.wasm"
                    .into(),
                checksum_sha256: "def456ghi789".into(),
                api_version: 1,
                plugin_type: "integration".into(),
                downloads: 5678,
                rating: 4.2,
            },
            MarketplacePlugin {
                id: "community.ai-denoise".into(),
                name: "AI Denoise".into(),
                version: "1.2.0".into(),
                description: "ML-based noise reduction (ONNX)".into(),
                author: "Community".into(),
                download_url: "https://marketplace.openclaw.photo/plugins/ai-denoise-1.2.0.wasm"
                    .into(),
                checksum_sha256: "ghi789jkl012".into(),
                api_version: 1,
                plugin_type: "ai_ml".into(),
                downloads: 9876,
                rating: 4.9,
            },
            MarketplacePlugin {
                id: "community.instagram-export".into(),
                name: "Instagram Export".into(),
                version: "1.5.0".into(),
                description: "Export with Instagram-optimized settings".into(),
                author: "Community".into(),
                download_url:
                    "https://marketplace.openclaw.photo/plugins/instagram-export-1.5.0.wasm".into(),
                checksum_sha256: "jkl012mno345".into(),
                api_version: 1,
                plugin_type: "import_export".into(),
                downloads: 3456,
                rating: 4.5,
            },
            MarketplacePlugin {
                id: "community.frequency-separation".into(),
                name: "Frequency Separation".into(),
                version: "2.0.0".into(),
                description: "Advanced portrait retouching technique".into(),
                author: "Community".into(),
                download_url:
                    "https://marketplace.openclaw.photo/plugins/freq-sep-2.0.0.wasm".into(),
                checksum_sha256: "mno345pqr678".into(),
                api_version: 1,
                plugin_type: "image_filter".into(),
                downloads: 7890,
                rating: 4.8,
            },
        ];

        Ok(self.plugins.len())
    }

    /// Search plugins by name or description
    pub fn search(&self, query: &str) -> Vec<&MarketplacePlugin> {
        let query_lower = query.to_lowercase();
        self.plugins
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Download and install a plugin
    ///
    /// In production: Downloads from download_url, verifies sha256, saves to target_dir
    /// For now: Creates stub plugin files
    pub fn download_plugin(
        &self,
        plugin_id: &str,
        target_dir: &Path,
    ) -> Result<PathBuf, MarketplaceError> {
        let plugin = self
            .plugins
            .iter()
            .find(|p| p.id == plugin_id)
            .ok_or_else(|| MarketplaceError::NotFound(plugin_id.to_string()))?;

        // Create plugin directory
        let plugin_dir = target_dir.join(&plugin.id);
        std::fs::create_dir_all(&plugin_dir)?;

        // Write plugin.toml manifest
        let manifest = format!(
            r#"[plugin]
name = "{}"
version = "{}"
api_version = "1"
type = "{}"
description = "{}"
author = "{}"
entry_point = "plugin.wasm"

[permissions]
read_image = true
write_image = true
read_metadata = {}
write_catalog = false
network = {}
filesystem = false
"#,
            plugin.name,
            plugin.version,
            plugin.plugin_type,
            plugin.description,
            plugin.author,
            // Grant read_metadata to metadata and ai_ml plugins
            plugin.plugin_type == "metadata" || plugin.plugin_type == "ai_ml",
            // Grant network to integration plugins
            plugin.plugin_type == "integration"
        );

        std::fs::write(plugin_dir.join("plugin.toml"), manifest)?;

        // Write stub WASM (minimal valid plugin)
        let stub_wat = r#"(module
  (func (export "plugin_init") (result i32) (i32.const 0))
  (func (export "plugin_info") (param i32) (result i32) (i32.const 100))
  (memory (export "memory") 1)
)"#;

        let wasm_bytes = wat::parse_str(stub_wat)
            .map_err(|e| MarketplaceError::InvalidPlugin(e.to_string()))?;

        std::fs::write(plugin_dir.join("plugin.wasm"), wasm_bytes)?;

        Ok(plugin_dir)
    }

    /// Get a plugin by ID
    pub fn get_plugin(&self, plugin_id: &str) -> Option<&MarketplacePlugin> {
        self.plugins.iter().find(|p| p.id == plugin_id)
    }

    /// Get plugins by type
    pub fn get_by_type(&self, plugin_type: &str) -> Vec<&MarketplacePlugin> {
        self.plugins
            .iter()
            .filter(|p| p.plugin_type == plugin_type)
            .collect()
    }

    /// Get top rated plugins
    pub fn get_top_rated(&self, limit: usize) -> Vec<&MarketplacePlugin> {
        let mut sorted = self.plugins.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(limit).collect()
    }

    /// Get most downloaded plugins
    pub fn get_most_downloaded(&self, limit: usize) -> Vec<&MarketplacePlugin> {
        let mut sorted = self.plugins.iter().collect::<Vec<_>>();
        sorted.sort_by_key(|p| std::cmp::Reverse(p.downloads));
        sorted.into_iter().take(limit).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_marketplace_fetch_demo_plugins() {
        let mut marketplace = Marketplace::new();
        let count = marketplace.fetch_plugins().expect("Should fetch plugins");

        assert!(
            count >= 3,
            "Should have at least 3 demo plugins, got {}",
            count
        );
        assert_eq!(count, marketplace.plugins.len());
    }

    #[test]
    fn test_marketplace_search() {
        let mut marketplace = Marketplace::new();
        marketplace.fetch_plugins().expect("Should fetch plugins");

        let results = marketplace.search("denoise");
        assert_eq!(
            results.len(),
            1,
            "Should find exactly one plugin matching 'denoise'"
        );
        assert_eq!(results[0].id, "community.ai-denoise");

        let results = marketplace.search("export");
        assert!(
            results.len() >= 2,
            "Should find at least 2 plugins matching 'export'"
        );
    }

    #[test]
    fn test_marketplace_search_case_insensitive() {
        let mut marketplace = Marketplace::new();
        marketplace.fetch_plugins().expect("Should fetch plugins");

        let results1 = marketplace.search("DENOISE");
        let results2 = marketplace.search("denoise");
        assert_eq!(results1.len(), results2.len(), "Search should be case-insensitive");
    }

    #[test]
    fn test_marketplace_download_creates_directory() {
        let mut marketplace = Marketplace::new();
        marketplace.fetch_plugins().expect("Should fetch plugins");

        let temp_dir = std::env::temp_dir().join("ocps-test-marketplace");
        let _ = fs::remove_dir_all(&temp_dir); // Clean up from previous test
        fs::create_dir_all(&temp_dir).expect("Should create temp dir");

        let plugin_dir = marketplace
            .download_plugin("community.lut-loader", &temp_dir)
            .expect("Should download plugin");

        // Verify directory created
        assert!(plugin_dir.exists(), "Plugin directory should exist");

        // Verify plugin.toml exists
        let manifest_path = plugin_dir.join("plugin.toml");
        assert!(manifest_path.exists(), "plugin.toml should exist");

        let manifest_content = fs::read_to_string(&manifest_path).expect("Should read manifest");
        assert!(manifest_content.contains("name = \"LUT Loader\""));
        assert!(manifest_content.contains("api_version = \"1\""));

        // Verify plugin.wasm exists
        let wasm_path = plugin_dir.join("plugin.wasm");
        assert!(wasm_path.exists(), "plugin.wasm should exist");

        let wasm_bytes = fs::read(&wasm_path).expect("Should read WASM");
        assert!(!wasm_bytes.is_empty(), "WASM file should not be empty");
        assert_eq!(&wasm_bytes[0..4], b"\0asm", "Should be valid WASM magic number");

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_marketplace_download_not_found() {
        let marketplace = Marketplace::new();
        let temp_dir = std::env::temp_dir().join("ocps-test-marketplace-notfound");

        let result = marketplace.download_plugin("nonexistent.plugin", &temp_dir);
        assert!(result.is_err(), "Should return error for non-existent plugin");

        match result {
            Err(MarketplaceError::NotFound(id)) => {
                assert_eq!(id, "nonexistent.plugin");
            }
            _ => panic!("Should return NotFound error"),
        }
    }

    #[test]
    fn test_marketplace_get_plugin() {
        let mut marketplace = Marketplace::new();
        marketplace.fetch_plugins().expect("Should fetch plugins");

        let plugin = marketplace.get_plugin("community.ai-denoise");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().name, "AI Denoise");

        let plugin = marketplace.get_plugin("nonexistent");
        assert!(plugin.is_none());
    }

    #[test]
    fn test_marketplace_get_by_type() {
        let mut marketplace = Marketplace::new();
        marketplace.fetch_plugins().expect("Should fetch plugins");

        let filters = marketplace.get_by_type("image_filter");
        assert!(filters.len() >= 2, "Should have at least 2 image filters");

        let integrations = marketplace.get_by_type("integration");
        assert!(integrations.len() >= 1, "Should have at least 1 integration");
    }

    #[test]
    fn test_marketplace_get_top_rated() {
        let mut marketplace = Marketplace::new();
        marketplace.fetch_plugins().expect("Should fetch plugins");

        let top = marketplace.get_top_rated(3);
        assert_eq!(top.len(), 3, "Should return 3 plugins");

        // Should be sorted by rating
        for i in 1..top.len() {
            assert!(
                top[i - 1].rating >= top[i].rating,
                "Should be sorted by rating descending"
            );
        }
    }

    #[test]
    fn test_marketplace_get_most_downloaded() {
        let mut marketplace = Marketplace::new();
        marketplace.fetch_plugins().expect("Should fetch plugins");

        let top = marketplace.get_most_downloaded(3);
        assert_eq!(top.len(), 3, "Should return 3 plugins");

        // Should be sorted by downloads
        for i in 1..top.len() {
            assert!(
                top[i - 1].downloads >= top[i].downloads,
                "Should be sorted by downloads descending"
            );
        }
    }
}
