//! Plugin manifest parser
//!
//! Parses plugin.toml files for plugin metadata and permissions.

use crate::PluginManifest;
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),
}

#[derive(Debug, Clone, Deserialize)]
struct PluginToml {
    plugin: PluginInfo,
    #[allow(dead_code)]
    permissions: Option<PluginPermissions>,
}

#[derive(Debug, Clone, Deserialize)]
struct PluginInfo {
    name: String,
    version: String,
    api_version: String,
    #[serde(rename = "type")]
    plugin_type: String,
    description: Option<String>,
    author: Option<String>,
    entry_point: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct PluginPermissions {
    read_image: Option<bool>,
    write_image: Option<bool>,
    read_catalog: Option<bool>,
    network: Option<bool>,
}

/// Load a plugin manifest from a plugin.toml file
pub fn load_manifest(plugin_dir: &Path) -> Result<PluginManifest, PluginError> {
    let manifest_path = plugin_dir.join("plugin.toml");

    if !manifest_path.exists() {
        return Err(PluginError::InvalidManifest(
            "plugin.toml not found".to_string(),
        ));
    }

    let content = std::fs::read_to_string(&manifest_path)?;
    let toml: PluginToml = toml::from_str(&content)?;

    // Generate plugin ID from name
    let id = toml.plugin.name.to_lowercase().replace(' ', "-");

    Ok(PluginManifest {
        id,
        name: toml.plugin.name,
        version: toml.plugin.version,
        api_version: toml.plugin.api_version,
        plugin_type: toml.plugin.plugin_type,
        author: toml.plugin.author.unwrap_or_else(|| "Unknown".to_string()),
        description: toml
            .plugin
            .description
            .unwrap_or_else(|| "No description".to_string()),
        entry_point: toml.plugin.entry_point,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use uuid::Uuid;

    #[test]
    fn test_load_manifest_valid() {
        let temp_dir = std::env::temp_dir();
        let plugin_dir = temp_dir.join(format!("test_plugin_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&plugin_dir).unwrap();

        let manifest_content = r#"
[plugin]
name = "Test Plugin"
version = "1.0.0"
api_version = "1"
type = "image_filter"
description = "A test plugin"
author = "Test Author"
entry_point = "plugin.wasm"

[permissions]
read_image = true
write_image = true
read_catalog = false
network = false
"#;

        let manifest_path = plugin_dir.join("plugin.toml");
        std::fs::write(&manifest_path, manifest_content).unwrap();

        let manifest = load_manifest(&plugin_dir).unwrap();
        assert_eq!(manifest.name, "Test Plugin");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.api_version, "1");
        assert_eq!(manifest.plugin_type, "image_filter");
        assert_eq!(manifest.description, "A test plugin");
        assert_eq!(manifest.author, "Test Author");
        assert_eq!(manifest.entry_point, "plugin.wasm");

        std::fs::remove_dir_all(plugin_dir).unwrap();
    }

    #[test]
    fn test_load_manifest_missing_file() {
        let temp_dir = std::env::temp_dir();
        let plugin_dir = temp_dir.join(format!("nonexistent_{}", Uuid::new_v4()));

        let result = load_manifest(&plugin_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_manifest_minimal() {
        let temp_dir = std::env::temp_dir();
        let plugin_dir = temp_dir.join(format!("test_plugin_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&plugin_dir).unwrap();

        let manifest_content = r#"
[plugin]
name = "Minimal Plugin"
version = "1.0.0"
api_version = "1"
type = "export"
entry_point = "plugin.wasm"
"#;

        let manifest_path = plugin_dir.join("plugin.toml");
        std::fs::write(&manifest_path, manifest_content).unwrap();

        let manifest = load_manifest(&plugin_dir).unwrap();
        assert_eq!(manifest.name, "Minimal Plugin");
        assert_eq!(manifest.description, "No description");
        assert_eq!(manifest.author, "Unknown");

        std::fs::remove_dir_all(plugin_dir).unwrap();
    }
}
