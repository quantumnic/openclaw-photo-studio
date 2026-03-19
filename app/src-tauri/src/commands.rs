use std::sync::Mutex;
use tauri::State;

/// Application state holding the catalog
pub struct AppState {
    pub catalog: Mutex<Option<ocps_catalog::Catalog>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            catalog: Mutex::new(None),
        }
    }
}

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

/// Get basic catalog info
#[tauri::command]
pub fn get_catalog_info(state: State<AppState>) -> serde_json::Value {
    let catalog_lock = state.catalog.lock().unwrap();

    if let Some(ref catalog) = *catalog_lock {
        match catalog.photo_count() {
            Ok(count) => serde_json::json!({
                "status": "open",
                "photo_count": count,
            }),
            Err(e) => serde_json::json!({
                "status": "error",
                "message": format!("{}", e),
            }),
        }
    } else {
        serde_json::json!({
            "status": "no_catalog_open",
            "photo_count": 0,
            "message": "No catalog open. Use import_folder to start."
        })
    }
}

/// Import a folder of photos into the catalog
///
/// This opens a catalog in the same directory as the folder and imports all photos.
#[tauri::command]
pub fn import_folder(state: State<AppState>, path: String) -> Result<serde_json::Value, String> {
    let folder_path = std::path::Path::new(&path);

    if !folder_path.exists() {
        return Err("Folder does not exist".to_string());
    }

    if !folder_path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    // Create catalog path in the same directory
    let catalog_path = folder_path.join(".ocps-catalog.db");

    // Open or create catalog
    let catalog = ocps_catalog::Catalog::open(&catalog_path)
        .map_err(|e| format!("Failed to open catalog: {}", e))?;

    // Import folder
    let result = catalog
        .import_folder(folder_path)
        .map_err(|e| format!("Import failed: {}", e))?;

    // Store catalog in state
    let mut catalog_lock = state.catalog.lock().unwrap();
    *catalog_lock = Some(catalog);

    Ok(serde_json::json!({
        "total": result.total,
        "inserted": result.inserted,
        "skipped": result.skipped,
        "errors": result.errors,
    }))
}

/// Get photos from the catalog with filtering and pagination
#[tauri::command]
pub fn get_photos(
    state: State<AppState>,
    filter: serde_json::Value,
    limit: u32,
    offset: u32,
) -> Result<Vec<serde_json::Value>, String> {
    let catalog_lock = state.catalog.lock().unwrap();

    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    // Parse filter
    let photo_filter = ocps_catalog::PhotoFilter {
        rating_min: filter.get("rating_min").and_then(|v| v.as_u64()).map(|v| v as u8),
        flag: filter.get("flag").and_then(|v| v.as_str()).map(|s| s.to_string()),
        color_label: filter.get("color_label").and_then(|v| v.as_str()).map(|s| s.to_string()),
        search: filter.get("search").and_then(|v| v.as_str()).map(|s| s.to_string()),
        limit,
        offset,
    };

    let sort = ocps_catalog::SortOrder::DateTaken;

    let photos = catalog
        .get_photos(&photo_filter, &sort)
        .map_err(|e| format!("Failed to get photos: {}", e))?;

    // Convert to JSON
    let json_photos = photos
        .into_iter()
        .map(|photo| {
            serde_json::json!({
                "id": photo.id,
                "file_path": photo.file_path,
                "file_name": photo.file_name,
                "file_size": photo.file_size,
                "width": photo.width,
                "height": photo.height,
                "date_taken": photo.date_taken,
                "date_imported": photo.date_imported,
                "camera_make": photo.camera_make,
                "camera_model": photo.camera_model,
                "rating": photo.rating,
                "color_label": photo.color_label,
                "flag": photo.flag,
                "has_edits": photo.has_edits,
            })
        })
        .collect();

    Ok(json_photos)
}

/// Update photo rating
#[tauri::command]
pub fn update_rating(state: State<AppState>, photo_id: String, rating: u8) -> Result<(), String> {
    let catalog_lock = state.catalog.lock().unwrap();

    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    catalog
        .update_rating(&photo_id, rating)
        .map_err(|e| format!("Failed to update rating: {}", e))
}

/// Update photo flag
#[tauri::command]
pub fn update_flag(state: State<AppState>, photo_id: String, flag: String) -> Result<(), String> {
    let catalog_lock = state.catalog.lock().unwrap();

    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    catalog
        .update_flag(&photo_id, &flag)
        .map_err(|e| format!("Failed to update flag: {}", e))
}

/// Update photo color label
#[tauri::command]
pub fn update_color_label(
    state: State<AppState>,
    photo_id: String,
    label: String,
) -> Result<(), String> {
    let catalog_lock = state.catalog.lock().unwrap();

    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    catalog
        .update_color_label(&photo_id, &label)
        .map_err(|e| format!("Failed to update color label: {}", e))
}

/// Get catalog statistics
#[tauri::command]
pub fn get_catalog_stats(state: State<AppState>) -> Result<serde_json::Value, String> {
    let catalog_lock = state.catalog.lock().unwrap();

    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    let total = catalog
        .photo_count()
        .map_err(|e| format!("Failed to get count: {}", e))?;

    // Get counts for rated photos
    let rated_filter = ocps_catalog::PhotoFilter {
        rating_min: Some(1),
        flag: None,
        color_label: None,
        search: None,
        limit: 100000,
        offset: 0,
    };

    let rated_photos = catalog
        .get_photos(&rated_filter, &ocps_catalog::SortOrder::Rating)
        .map_err(|e| format!("Failed to get rated photos: {}", e))?;

    let rated = rated_photos.len();

    // Get picks
    let pick_filter = ocps_catalog::PhotoFilter {
        rating_min: None,
        flag: Some("pick".to_string()),
        color_label: None,
        search: None,
        limit: 100000,
        offset: 0,
    };

    let pick_photos = catalog
        .get_photos(&pick_filter, &ocps_catalog::SortOrder::DateTaken)
        .map_err(|e| format!("Failed to get picks: {}", e))?;

    let picks = pick_photos.len();

    // Get rejects
    let reject_filter = ocps_catalog::PhotoFilter {
        rating_min: None,
        flag: Some("reject".to_string()),
        color_label: None,
        search: None,
        limit: 100000,
        offset: 0,
    };

    let reject_photos = catalog
        .get_photos(&reject_filter, &ocps_catalog::SortOrder::DateTaken)
        .map_err(|e| format!("Failed to get rejects: {}", e))?;

    let rejects = reject_photos.len();

    Ok(serde_json::json!({
        "total": total,
        "rated": rated,
        "picks": picks,
        "rejects": rejects,
    }))
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
