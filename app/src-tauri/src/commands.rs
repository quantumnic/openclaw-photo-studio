/// Proof-of-concept IPC command
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!(
        "Hello, {}! OpenClaw Photo Studio v{} is running.",
        name,
        env!("CARGO_PKG_VERSION")
    )
}

/// Get version info from all crates
#[tauri::command]
pub fn get_version() -> serde_json::Value {
    serde_json::json!({
        "app": env!("CARGO_PKG_VERSION"),
        "core": ocps_core::version(),
        "catalog": ocps_catalog::version(),
        "xmp": ocps_xmp::version(),
        "export": ocps_export::version(),
    })
}

/// Get basic catalog info (placeholder)
#[tauri::command]
pub fn get_catalog_info() -> serde_json::Value {
    serde_json::json!({
        "status": "no_catalog_open",
        "photo_count": 0,
        "message": "Open a folder to start. Library coming in Phase 2."
    })
}
