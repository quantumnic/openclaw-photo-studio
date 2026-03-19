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

/// Decode a RAW file and return metadata
///
/// # Arguments
/// * `path` - Absolute path to the RAW file
///
/// # Returns
/// * `Ok(Value)` - JSON object with RAW metadata
/// * `Err(String)` - Error message if decode fails
#[tauri::command]
pub fn decode_raw_info(path: String) -> Result<serde_json::Value, String> {
    let p = std::path::Path::new(&path);
    match ocps_core::decode_meta(p) {
        Ok(meta) => Ok(serde_json::json!({
            "ok": true,
            "camera": format!("{} {}", meta.camera_make, meta.camera_model),
            "width": meta.width,
            "height": meta.height,
            "format": meta.format,
            "white_level": meta.white_level,
            "black_level": meta.black_level,
        })),
        Err(e) => Err(format!("{:?}", e)),
    }
}
