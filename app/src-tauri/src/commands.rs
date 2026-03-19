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
    use std::path::Path;

    let start = std::time::Instant::now();

    // Decode the RAW file
    let raw = ocps_core::raw::decode(Path::new(&path))
        .map_err(|e| format!("Failed to decode RAW: {}", e))?;

    let decode_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Format exposure time as a string
    let exposure_str = match raw.exposure_time {
        Some(exp) if exp >= 1.0 => format!("{:.1}s", exp),
        Some(exp) => format!("1/{:.0}s", 1.0 / exp),
        None => "Unknown".to_string(),
    };

    Ok(serde_json::json!({
        "success": true,
        "camera_make": raw.camera_make,
        "camera_model": raw.camera_model,
        "width": raw.width,
        "height": raw.height,
        "megapixels": (raw.width * raw.height) as f64 / 1_000_000.0,
        "cfa_pattern": format!("{:?}", raw.cfa_pattern),
        "wb_coeffs": raw.wb_coeffs,
        "black_level": raw.black_level,
        "white_level": raw.white_level,
        "iso": raw.iso,
        "exposure_time": exposure_str,
        "aperture": raw.aperture.map(|a| format!("f/{:.1}", a)),
        "decode_time_ms": decode_time_ms,
    }))
}
