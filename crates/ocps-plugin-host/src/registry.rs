//! Plugin registry for discovery and management

use crate::manifest::{load_manifest, PluginError};
use crate::PluginManifest;
use std::path::{Path, PathBuf};

pub struct PluginRegistry {
    plugins: Vec<(PluginManifest, PathBuf)>,
}

impl PluginRegistry {
    /// Create a new empty plugin registry
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Scan a directory for plugins and load their manifests
    ///
    /// Returns the number of plugins found.
    pub fn scan_directory(&mut self, dir: &Path) -> Result<usize, PluginError> {
        if !dir.exists() || !dir.is_dir() {
            return Ok(0);
        }

        let mut count = 0;

        // Walk through subdirectories
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Try to load plugin.toml from this directory
                match load_manifest(&path) {
                    Ok(manifest) => {
                        self.plugins.push((manifest, path));
                        count += 1;
                    }
                    Err(_) => {
                        // Not a valid plugin directory, skip silently
                        continue;
                    }
                }
            }
        }

        Ok(count)
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<&PluginManifest> {
        self.plugins.iter().map(|(manifest, _)| manifest).collect()
    }

    /// Find a plugin by ID
    pub fn find_by_id(&self, id: &str) -> Option<&PluginManifest> {
        self.plugins
            .iter()
            .find(|(manifest, _)| manifest.id == id)
            .map(|(manifest, _)| manifest)
    }

    /// Get the path to a plugin's directory
    pub fn get_plugin_path(&self, id: &str) -> Option<&Path> {
        self.plugins
            .iter()
            .find(|(manifest, _)| manifest.id == id)
            .map(|(_, path)| path.as_path())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use uuid::Uuid;

    #[test]
    fn test_registry_scan_empty_dir() {
        let temp_dir = std::env::temp_dir().join(format!("empty_plugins_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        let mut registry = PluginRegistry::new();
        let count = registry.scan_directory(&temp_dir).unwrap();

        assert_eq!(count, 0);
        assert_eq!(registry.list_plugins().len(), 0);

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_registry_scan_with_plugin() {
        let temp_dir = std::env::temp_dir().join(format!("plugins_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a plugin directory with plugin.toml
        let plugin_dir = temp_dir.join("test-plugin");
        fs::create_dir_all(&plugin_dir).unwrap();

        let manifest_content = r#"
[plugin]
name = "Test Plugin"
version = "1.0.0"
api_version = "1"
type = "image_filter"
entry_point = "plugin.wasm"
"#;

        fs::write(plugin_dir.join("plugin.toml"), manifest_content).unwrap();

        let mut registry = PluginRegistry::new();
        let count = registry.scan_directory(&temp_dir).unwrap();

        assert_eq!(count, 1);
        assert_eq!(registry.list_plugins().len(), 1);

        let plugins = registry.list_plugins();
        assert_eq!(plugins[0].name, "Test Plugin");

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_registry_find_by_id() {
        let temp_dir = std::env::temp_dir().join(format!("plugins_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        let plugin_dir = temp_dir.join("cool-filter");
        fs::create_dir_all(&plugin_dir).unwrap();

        let manifest_content = r#"
[plugin]
name = "Cool Filter"
version = "1.0.0"
api_version = "1"
type = "image_filter"
entry_point = "plugin.wasm"
"#;

        fs::write(plugin_dir.join("plugin.toml"), manifest_content).unwrap();

        let mut registry = PluginRegistry::new();
        registry.scan_directory(&temp_dir).unwrap();

        let found = registry.find_by_id("cool-filter");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Cool Filter");

        let not_found = registry.find_by_id("nonexistent");
        assert!(not_found.is_none());

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_registry_scan_nonexistent_dir() {
        let temp_dir = std::env::temp_dir().join(format!("nonexistent_{}", Uuid::new_v4()));

        let mut registry = PluginRegistry::new();
        let count = registry.scan_directory(&temp_dir).unwrap();

        assert_eq!(count, 0);
    }
}
