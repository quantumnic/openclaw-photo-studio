//! ocps-plugin-host — WASM Plugin Host
//! Sandboxed plugin execution via wasmtime (Phase 6)

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
