//! ocps-plugin-host — WASM Plugin Host
//! Sandboxed plugin execution via wasmtime

pub mod api;
pub mod host;
pub mod manifest;
pub mod marketplace;
pub mod registry;
pub mod sdk;
pub mod tether;

pub use api::{PluginErrorCode, PluginType, PLUGIN_API_VERSION};
pub use host::{PluginHost, PluginPermissions, PluginState};
pub use manifest::{load_manifest, PluginError};
pub use marketplace::{Marketplace, MarketplaceError, MarketplacePlugin};
pub use registry::PluginRegistry;
pub use sdk::{generate_rust_template, generate_wat_template};
pub use tether::{MockTetherProvider, TetherError, TetherProvider, TetherSession, TetheredCamera};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api_version: String,
    pub plugin_type: String,
    pub author: String,
    pub description: String,
    pub entry_point: String,
}

pub fn load_plugin(_path: &std::path::Path) -> anyhow::Result<PluginManifest> {
    // TODO: Phase 6 — implement WASM plugin loading via wasmtime
    anyhow::bail!("Plugin system not yet implemented — coming in Phase 6")
}
