use std::sync::Mutex;
use tauri::State;

/// Standard error type for all Tauri commands
#[derive(Debug, serde::Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

impl CommandError {
    pub fn catalog_not_open() -> Self {
        Self {
            code: "CATALOG_NOT_OPEN".into(),
            message: "No catalog is open. Import a folder first.".into(),
            details: None,
        }
    }

    pub fn file_not_found(path: &str) -> Self {
        Self {
            code: "FILE_NOT_FOUND".into(),
            message: format!("File not found: {}", path),
            details: None,
        }
    }

    #[allow(dead_code)]
    pub fn decode_failed(path: &str, reason: &str) -> Self {
        Self {
            code: "DECODE_FAILED".into(),
            message: format!("Failed to decode {}", path),
            details: Some(reason.to_string()),
        }
    }

    pub fn catalog_error(operation: &str, error: impl std::fmt::Display) -> Self {
        Self {
            code: "CATALOG_ERROR".into(),
            message: format!("{}: {}", operation, error),
            details: None,
        }
    }

    pub fn invalid_input(field: &str, reason: &str) -> Self {
        Self {
            code: "INVALID_INPUT".into(),
            message: format!("Invalid {}: {}", field, reason),
            details: None,
        }
    }

    pub fn internal_error(operation: &str, error: impl std::fmt::Display) -> Self {
        Self {
            code: "INTERNAL_ERROR".into(),
            message: format!("{} failed: {}", operation, error),
            details: None,
        }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for CommandError {}

/// Sidecar sync mode
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SidecarMode {
    Auto,
    Manual,
    ReadOnly,
    Disabled,
}

/// Application state holding the catalog and edit clipboard
pub struct AppState {
    pub catalog: Mutex<Option<ocps_catalog::Catalog>>,
    pub clipboard: Mutex<Option<ocps_core::EditClipboard>>,
    pub preset_library: Mutex<ocps_core::PresetLibrary>,
    pub sidecar_mode: Mutex<SidecarMode>,
    pub plugin_registry: Mutex<ocps_plugin_host::PluginRegistry>,
    pub preview_cache: Mutex<ocps_core::preview_cache::PreviewCache>,
    pub histories: Mutex<std::collections::HashMap<String, ocps_core::EditHistory>>,
    pub render_cache: Mutex<crate::render::RenderCache>,
}

impl AppState {
    pub fn new() -> Self {
        // Initialize preset library with user directory
        let user_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("openclaw-photo-studio")
            .join("presets");

        let mut preset_library = ocps_core::PresetLibrary::new(user_dir);
        let _ = preset_library.load_user_presets(); // Load user presets if they exist

        // Initialize plugin registry
        let mut plugin_registry = ocps_plugin_host::PluginRegistry::new();
        let plugin_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("openclaw-photo-studio")
            .join("plugins");
        let _ = plugin_registry.scan_directory(&plugin_dir);

        // Initialize preview cache
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("openclaw-photo-studio")
            .join("previews");

        let preview_cache = ocps_core::preview_cache::PreviewCache::new(cache_dir, 200);

        Self {
            catalog: Mutex::new(None),
            clipboard: Mutex::new(None),
            preset_library: Mutex::new(preset_library),
            sidecar_mode: Mutex::new(SidecarMode::Auto),
            plugin_registry: Mutex::new(plugin_registry),
            preview_cache: Mutex::new(preview_cache),
            histories: Mutex::new(std::collections::HashMap::new()),
            render_cache: Mutex::new(crate::render::RenderCache::new(10)),
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
            Ok(count) => {
                // Get catalog path from database path
                let db_path = catalog.database_path();
                let catalog_path = db_path.to_string_lossy().to_string();

                // Try to get creation/modification time
                let metadata = std::fs::metadata(db_path).ok();
                let created_at = metadata
                    .as_ref()
                    .and_then(|m| m.created().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs());
                let modified_at = metadata
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs());

                serde_json::json!({
                    "status": "open",
                    "photo_count": count,
                    "catalog_path": catalog_path,
                    "created_at": created_at,
                    "modified_at": modified_at,
                })
            }
            Err(e) => serde_json::json!({
                "status": "error",
                "message": format!("{}", e),
            }),
        }
    } else {
        serde_json::json!({
            "status": "no_catalog_open",
            "photo_count": 0,
            "catalog_path": null,
            "message": "No catalog open. Use import_folder or open_catalog to start."
        })
    }
}

/// Open an existing catalog from a specific .ocps file
#[tauri::command]
pub fn open_catalog(
    state: State<AppState>,
    catalog_path: String,
) -> Result<serde_json::Value, CommandError> {
    let path = std::path::Path::new(&catalog_path);

    if !path.exists() {
        return Err(CommandError::file_not_found(&catalog_path));
    }

    // Close existing catalog if any
    {
        let mut catalog_lock = state.catalog.lock().unwrap();
        *catalog_lock = None;
    }

    // Open the catalog
    let catalog = ocps_catalog::Catalog::open(path)
        .map_err(|e| CommandError::catalog_error("open catalog", e))?;

    let photo_count = catalog
        .photo_count()
        .map_err(|e| CommandError::catalog_error("get photo count", e))?;

    // Store in state
    {
        let mut catalog_lock = state.catalog.lock().unwrap();
        *catalog_lock = Some(catalog);
    }

    Ok(serde_json::json!({
        "photo_count": photo_count,
        "catalog_path": catalog_path,
    }))
}

/// Close the current catalog
#[tauri::command]
pub fn close_catalog(state: State<AppState>) -> Result<(), CommandError> {
    let mut catalog_lock = state.catalog.lock().unwrap();
    *catalog_lock = None;

    // Also clear histories
    let mut histories = state.histories.lock().unwrap();
    histories.clear();

    Ok(())
}

/// Create a new empty catalog at the specified path
#[tauri::command]
pub fn new_catalog(
    state: State<AppState>,
    catalog_path: String,
) -> Result<(), CommandError> {
    let path = std::path::Path::new(&catalog_path);

    // Check if parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            return Err(CommandError::file_not_found(parent.to_str().unwrap_or("")));
        }
    }

    // Close existing catalog
    {
        let mut catalog_lock = state.catalog.lock().unwrap();
        *catalog_lock = None;
    }

    // Create new catalog
    let catalog = ocps_catalog::Catalog::open(path)
        .map_err(|e| CommandError::catalog_error("create catalog", e))?;

    // Store in state
    {
        let mut catalog_lock = state.catalog.lock().unwrap();
        *catalog_lock = Some(catalog);
    }

    Ok(())
}

/// Import a folder of photos into the catalog
///
/// This opens a catalog in the same directory as the folder and imports all photos.
#[tauri::command]
pub fn import_folder(state: State<AppState>, path: String) -> Result<serde_json::Value, CommandError> {
    let folder_path = std::path::Path::new(&path);

    if !folder_path.exists() {
        return Err(CommandError::file_not_found(&path));
    }

    if !folder_path.is_dir() {
        return Err(CommandError::invalid_input("path", "not a directory"));
    }

    // Create catalog path in the same directory
    let catalog_path = folder_path.join(".ocps-catalog.db");

    // Open or create catalog
    let catalog = ocps_catalog::Catalog::open(&catalog_path)
        .map_err(|e| CommandError::catalog_error("open catalog", e))?;

    // Import folder
    let result = catalog
        .import_folder(folder_path)
        .map_err(|e| CommandError::catalog_error("import_folder", e))?;

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

    // Check if search query is provided
    let search_query = filter.get("search").and_then(|v| v.as_str()).map(|s| s.to_string());

    let photos = if let Some(query) = search_query {
        // Use FTS5 search if search query is provided
        if !query.trim().is_empty() {
            catalog
                .search(&query, limit)
                .map_err(|e| format!("Search failed: {}", e))?
        } else {
            // Empty search, fall through to regular filtering
            let photo_filter = ocps_catalog::PhotoFilter {
                rating_min: filter.get("rating_min").and_then(|v| v.as_u64()).map(|v| v as u8),
                flag: filter.get("flag").and_then(|v| v.as_str()).map(|s| s.to_string()),
                color_label: filter.get("color_label").and_then(|v| v.as_str()).map(|s| s.to_string()),
                search: None,
                limit,
                offset,
            };
            let sort = ocps_catalog::SortOrder::DateTaken;
            catalog
                .get_photos(&photo_filter, &sort)
                .map_err(|e| format!("Failed to get photos: {}", e))?
        }
    } else {
        // No search query, use regular filtering
        let photo_filter = ocps_catalog::PhotoFilter {
            rating_min: filter.get("rating_min").and_then(|v| v.as_u64()).map(|v| v as u8),
            flag: filter.get("flag").and_then(|v| v.as_str()).map(|s| s.to_string()),
            color_label: filter.get("color_label").and_then(|v| v.as_str()).map(|s| s.to_string()),
            search: None,
            limit,
            offset,
        };
        let sort = ocps_catalog::SortOrder::DateTaken;
        catalog
            .get_photos(&photo_filter, &sort)
            .map_err(|e| format!("Failed to get photos: {}", e))?
    };

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

/// Save edit recipe for a photo
#[tauri::command]
pub fn save_edit_recipe(
    state: State<AppState>,
    photo_id: String,
    recipe: serde_json::Value,
) -> Result<(), String> {
    let recipe_json = recipe.to_string();
    let catalog_lock = state.catalog.lock().unwrap();

    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    // Parse new recipe
    let new_recipe: ocps_core::EditRecipe = serde_json::from_value(recipe)
        .map_err(|e| format!("Failed to parse recipe: {}", e))?;

    // Update history
    let mut histories = state.histories.lock().unwrap();
    let description = if let Some(history) = histories.get(&photo_id) {
        // Generate description from changes
        ocps_core::EditHistory::auto_describe(history.current(), &new_recipe)
    } else {
        // First edit
        "Initial edit".to_string()
    };

    let history = histories
        .entry(photo_id.clone())
        .or_insert_with(|| ocps_core::EditHistory::new(ocps_core::EditRecipe::default()));

    history.push(new_recipe.clone(), description);
    drop(histories);

    // Save to catalog database
    catalog
        .save_edit(&photo_id, &recipe_json)
        .map_err(|e| format!("Failed to save edit: {}", e))?;

    // Check sidecar mode and write XMP if needed
    let sidecar_mode = state.sidecar_mode.lock().unwrap();
    if *sidecar_mode == SidecarMode::Auto {
        // Get photo file path
        if let Ok(Some(photo)) = catalog.get_photo(&photo_id) {
            let raw_path = std::path::Path::new(&photo.file_path);

            // Parse recipe to XmpDevelopSettings
            if let Ok(edit_recipe) = serde_json::from_str::<ocps_core::EditRecipe>(&recipe_json) {
                let xmp_settings = ocps_xmp::XmpDevelopSettings {
                    temperature: Some(edit_recipe.white_balance.temperature as i32),
                    tint: Some(edit_recipe.white_balance.tint),
                    exposure: Some(edit_recipe.exposure),
                    contrast: Some(edit_recipe.contrast),
                    highlights: Some(edit_recipe.highlights),
                    shadows: Some(edit_recipe.shadows),
                    whites: Some(edit_recipe.whites),
                    blacks: Some(edit_recipe.blacks),
                    clarity: Some(edit_recipe.clarity),
                    dehaze: Some(edit_recipe.dehaze),
                    vibrance: Some(edit_recipe.vibrance),
                    saturation: Some(edit_recipe.saturation),
                    rating: None,
                    label: None,
                    process_version: Some("1".to_string()),
                };

                let iptc = ocps_xmp::IptcData::default();

                // Write XMP sidecar
                let _ = ocps_xmp::write_sidecar(raw_path, &xmp_settings, &iptc);
            }
        }
    }

    Ok(())
}

/// Load edit recipe for a photo
#[tauri::command]
pub fn load_edit_recipe(
    state: State<AppState>,
    photo_id: String,
) -> Result<serde_json::Value, String> {
    let catalog_lock = state.catalog.lock().unwrap();

    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    match catalog.load_edit(&photo_id) {
        Ok(Some(json)) => {
            serde_json::from_str(&json).map_err(|e| format!("Failed to parse recipe: {}", e))
        }
        Ok(None) => {
            // Return default recipe
            let default = ocps_core::EditRecipe::default();
            serde_json::to_value(default).map_err(|e| format!("Failed to serialize: {}", e))
        }
        Err(e) => Err(format!("Failed to load edit: {}", e)),
    }
}

/// Copy edit settings from a photo
#[tauri::command]
pub fn copy_edit(
    state: State<AppState>,
    photo_id: String,
    modules: Vec<String>,
) -> Result<serde_json::Value, String> {
    // Load recipe from catalog
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    let recipe_json = catalog
        .load_edit(&photo_id)
        .map_err(|e| format!("Failed to load edit: {}", e))?;

    let recipe: ocps_core::EditRecipe = if let Some(json) = recipe_json {
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse recipe: {}", e))?
    } else {
        ocps_core::EditRecipe::default()
    };

    // Convert module names to EditModule enum
    let edit_modules: Vec<ocps_core::EditModule> = if modules.is_empty() {
        ocps_core::EditModule::safe_defaults()
    } else {
        modules
            .iter()
            .filter_map(|m| match m.as_str() {
                "white_balance" => Some(ocps_core::EditModule::WhiteBalance),
                "exposure" => Some(ocps_core::EditModule::Exposure),
                "contrast" => Some(ocps_core::EditModule::Contrast),
                "highlights" => Some(ocps_core::EditModule::Highlights),
                "shadows" => Some(ocps_core::EditModule::Shadows),
                "whites" => Some(ocps_core::EditModule::Whites),
                "blacks" => Some(ocps_core::EditModule::Blacks),
                "clarity" => Some(ocps_core::EditModule::Clarity),
                "dehaze" => Some(ocps_core::EditModule::Dehaze),
                "vibrance" => Some(ocps_core::EditModule::Vibrance),
                "saturation" => Some(ocps_core::EditModule::Saturation),
                "sharpening" => Some(ocps_core::EditModule::Sharpening),
                "noise_reduction" => Some(ocps_core::EditModule::NoiseReduction),
                "crop" => Some(ocps_core::EditModule::Crop),
                "color_grading" => Some(ocps_core::EditModule::ColorGrading),
                _ => None,
            })
            .collect()
    };

    // Create clipboard
    let clipboard = ocps_core::EditCopyPaste::copy_selected(&photo_id, &recipe, edit_modules);

    // Store in state
    let mut clipboard_lock = state.clipboard.lock().unwrap();
    *clipboard_lock = Some(clipboard.clone());

    // Return clipboard as JSON
    serde_json::to_value(clipboard).map_err(|e| format!("Failed to serialize clipboard: {}", e))
}

/// Paste edit settings to one or more photos
#[tauri::command]
pub fn paste_edit(
    state: State<AppState>,
    photo_ids: Vec<String>,
    modules: Vec<String>,
) -> Result<u32, String> {
    // Get clipboard
    let clipboard_lock = state.clipboard.lock().unwrap();
    let clipboard = clipboard_lock
        .as_ref()
        .ok_or("No edit settings copied".to_string())?
        .clone();
    drop(clipboard_lock);

    // Convert module names if provided
    let paste_modules: Vec<ocps_core::EditModule> = if modules.is_empty() {
        clipboard.modules.clone()
    } else {
        modules
            .iter()
            .filter_map(|m| match m.as_str() {
                "white_balance" => Some(ocps_core::EditModule::WhiteBalance),
                "exposure" => Some(ocps_core::EditModule::Exposure),
                "contrast" => Some(ocps_core::EditModule::Contrast),
                "highlights" => Some(ocps_core::EditModule::Highlights),
                "shadows" => Some(ocps_core::EditModule::Shadows),
                "whites" => Some(ocps_core::EditModule::Whites),
                "blacks" => Some(ocps_core::EditModule::Blacks),
                "clarity" => Some(ocps_core::EditModule::Clarity),
                "dehaze" => Some(ocps_core::EditModule::Dehaze),
                "vibrance" => Some(ocps_core::EditModule::Vibrance),
                "saturation" => Some(ocps_core::EditModule::Saturation),
                "sharpening" => Some(ocps_core::EditModule::Sharpening),
                "noise_reduction" => Some(ocps_core::EditModule::NoiseReduction),
                "crop" => Some(ocps_core::EditModule::Crop),
                "color_grading" => Some(ocps_core::EditModule::ColorGrading),
                _ => None,
            })
            .collect()
    };

    // Get catalog
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    let mut count = 0;

    // Apply to each photo
    for photo_id in &photo_ids {
        // Load current recipe
        let recipe_json = catalog
            .load_edit(photo_id)
            .map_err(|e| format!("Failed to load edit: {}", e))?;

        let mut recipe: ocps_core::EditRecipe = if let Some(json) = recipe_json {
            serde_json::from_str(&json)
                .map_err(|e| format!("Failed to parse recipe: {}", e))?
        } else {
            ocps_core::EditRecipe::default()
        };

        // Apply paste
        ocps_core::EditCopyPaste::paste_selected(&clipboard, &mut recipe, &paste_modules);

        // Save back
        let recipe_json =
            serde_json::to_string(&recipe).map_err(|e| format!("Failed to serialize: {}", e))?;
        catalog
            .save_edit(photo_id, &recipe_json)
            .map_err(|e| format!("Failed to save: {}", e))?;

        count += 1;
    }

    Ok(count)
}

/// Reset edit settings for a photo
#[tauri::command]
pub fn reset_edit(state: State<AppState>, photo_id: String) -> Result<(), String> {
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    let default = ocps_core::EditRecipe::default();
    let recipe_json =
        serde_json::to_string(&default).map_err(|e| format!("Failed to serialize: {}", e))?;

    catalog
        .save_edit(&photo_id, &recipe_json)
        .map_err(|e| format!("Failed to save: {}", e))
}

/// Export a photo as JPEG
#[tauri::command]
pub fn export_photo_jpeg(
    state: State<AppState>,
    photo_id: String,
    output_path: String,
    quality: u32,
    resize_long_edge: Option<u32>,
) -> Result<serde_json::Value, String> {
    let start = std::time::Instant::now();

    // Get catalog
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    // Load photo path
    let photo = catalog
        .get_photo(&photo_id)
        .map_err(|e| format!("Failed to get photo: {}", e))?
        .ok_or("Photo not found".to_string())?;

    let path = std::path::Path::new(&photo.file_path);

    // Detect file type and load appropriately
    let is_raw = is_raw_file(path);

    let image = if is_raw {
        // RAW workflow
        let raw = ocps_core::decode(path)
            .map_err(|e| format!("Failed to decode RAW: {:?}", e))?;

        let rgb = ocps_core::demosaic(&raw, ocps_core::DemosaicAlgorithm::Bilinear);

        // Convert u8 → u16 for pipeline
        let data_u16: Vec<u16> = rgb.data.iter().map(|&v| (v as u16) * 257).collect();

        ocps_core::RgbImage16::from_data(rgb.width, rgb.height, data_u16)
    } else {
        // JPEG/TIFF workflow
        let img = image::open(path)
            .map_err(|e| format!("Failed to open image: {}", e))?;

        let rgb8 = img.to_rgb8();
        let width = rgb8.width();
        let height = rgb8.height();

        // Convert u8 → u16 for pipeline
        let data_u16: Vec<u16> = rgb8.as_raw().iter().map(|&v| (v as u16) * 257).collect();

        ocps_core::RgbImage16::from_data(width, height, data_u16)
    };

    // Load edit recipe
    let recipe_json = catalog
        .load_edit(&photo_id)
        .map_err(|e| format!("Failed to load edit: {}", e))?;

    let recipe: ocps_core::EditRecipe = if let Some(json) = recipe_json {
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse recipe: {}", e))?
    } else {
        ocps_core::EditRecipe::default()
    };

    // Apply pipeline
    let output = ocps_core::ImageProcessor::process(&image, &recipe);

    // Resize if requested
    let (final_data, final_width, final_height) = if let Some(long_edge) = resize_long_edge {
        ocps_export::resize::resize_long_edge(&output.data, output.width, output.height, long_edge)
    } else {
        (output.data, output.width, output.height)
    };

    // Export JPEG
    let output_p = std::path::Path::new(&output_path);
    ocps_export::jpeg::export_jpeg(&final_data, final_width, final_height, quality, output_p)
        .map_err(|e| format!("Failed to export JPEG: {:?}", e))?;

    // Get file size
    let file_size = std::fs::metadata(output_p).map(|m| m.len()).unwrap_or(0);

    let duration_ms = start.elapsed().as_millis();

    Ok(serde_json::json!({
        "output_path": output_path,
        "width": final_width,
        "height": final_height,
        "file_size": file_size,
        "duration_ms": duration_ms,
    }))
}

/// Check if a file is a RAW format
fn is_raw_file(path: &std::path::Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(
            ext_str.as_str(),
            "arw" | "nef" | "raf" | "dng" | "cr2" | "cr3" | "orf" | "rw2"
        )
    } else {
        false
    }
}

/// Batch export multiple photos
#[tauri::command]
pub fn export_photos_batch(
    state: State<AppState>,
    photo_ids: Vec<String>,
    output_folder: String,
    format: String,
    quality: u32,
    resize_long_edge: Option<u32>,
    naming_template: String,
) -> Result<serde_json::Value, String> {
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    let output_dir = std::path::Path::new(&output_folder);
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    let mut succeeded = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    // Determine file extension from format
    let extension = match format.as_str() {
        "jpeg" => "jpg",
        "png" => "png",
        "tiff" => "tiff",
        _ => "jpg",
    };

    for (index, photo_id) in photo_ids.iter().enumerate() {
        // Get photo
        let photo = match catalog.get_photo(photo_id) {
            Ok(Some(p)) => p,
            Ok(None) => {
                errors.push(format!("{}: Photo not found", photo_id));
                failed += 1;
                continue;
            }
            Err(e) => {
                errors.push(format!("{}: {}", photo_id, e));
                failed += 1;
                continue;
            }
        };

        // Construct output filename based on naming template
        let photo_for_naming = ocps_export::PhotoForNaming {
            file_path: photo.file_path.clone(),
            date_taken: photo.date_taken.clone(),
            camera_make: photo.camera_make.clone(),
            camera_model: photo.camera_model.clone(),
            rating: photo.rating,
        };

        let output_name = ocps_export::apply_naming_template(
            &naming_template,
            &photo_for_naming,
            (index + 1) as u32,
        );

        let output_path = output_dir.join(format!("{}.{}", output_name, extension));

        // Try to export
        match export_single_photo(
            catalog,
            &photo,
            &output_path,
            quality,
            resize_long_edge,
        ) {
            Ok(_) => succeeded += 1,
            Err(e) => {
                errors.push(format!("{}: {}", photo_id, e));
                failed += 1;
            }
        }
    }

    Ok(serde_json::json!({
        "total": photo_ids.len(),
        "succeeded": succeeded,
        "failed": failed,
        "errors": errors,
    }))
}

/// Helper function to export a single photo (internal use)
fn export_single_photo(
    catalog: &ocps_catalog::Catalog,
    photo: &ocps_catalog::PhotoRecord,
    output_path: &std::path::Path,
    quality: u32,
    resize_long_edge: Option<u32>,
) -> Result<(), String> {
    let path = std::path::Path::new(&photo.file_path);

    // Detect file type and load appropriately
    let is_raw = is_raw_file(path);

    let image = if is_raw {
        // RAW workflow
        let raw = ocps_core::decode(path)
            .map_err(|e| format!("Failed to decode RAW: {:?}", e))?;

        let rgb = ocps_core::demosaic(&raw, ocps_core::DemosaicAlgorithm::Bilinear);

        // Convert u8 → u16 for pipeline
        let data_u16: Vec<u16> = rgb.data.iter().map(|&v| (v as u16) * 257).collect();

        ocps_core::RgbImage16::from_data(rgb.width, rgb.height, data_u16)
    } else {
        // JPEG/TIFF workflow
        let img = image::open(path)
            .map_err(|e| format!("Failed to open image: {}", e))?;

        let rgb8 = img.to_rgb8();
        let width = rgb8.width();
        let height = rgb8.height();

        // Convert u8 → u16 for pipeline
        let data_u16: Vec<u16> = rgb8.as_raw().iter().map(|&v| (v as u16) * 257).collect();

        ocps_core::RgbImage16::from_data(width, height, data_u16)
    };

    // Load edit recipe
    let recipe_json = catalog
        .load_edit(&photo.id)
        .map_err(|e| format!("Failed to load edit: {}", e))?;

    let recipe: ocps_core::EditRecipe = if let Some(json) = recipe_json {
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse recipe: {}", e))?
    } else {
        ocps_core::EditRecipe::default()
    };

    // Apply pipeline
    let output = ocps_core::ImageProcessor::process(&image, &recipe);

    // Resize if requested
    let (final_data, final_width, final_height) = if let Some(long_edge) = resize_long_edge {
        ocps_export::resize::resize_long_edge(&output.data, output.width, output.height, long_edge)
    } else {
        (output.data, output.width, output.height)
    };

    // Export JPEG
    ocps_export::jpeg::export_jpeg(&final_data, final_width, final_height, quality, output_path)
        .map_err(|e| format!("Failed to export JPEG: {:?}", e))?;

    Ok(())
}

/// Compute histogram for a photo
///
/// # Arguments
/// * `photo_id` - Photo ID to compute histogram for
///
/// # Returns
/// * `Ok(Value)` - JSON object with histogram data (red, green, blue, luma arrays of 256 values)
/// * `Err(String)` - Error message if computation fails
#[tauri::command]
pub fn compute_histogram(_photo_id: String) -> Result<serde_json::Value, String> {
    // For now, return synthetic histogram data
    // Real render pipeline integration will happen in Phase 3

    // Generate a synthetic histogram with a bell curve distribution
    let mut red = [0u32; 256];
    let mut green = [0u32; 256];
    let mut blue = [0u32; 256];
    let mut luma = [0u32; 256];

    // Create a bell curve centered around 128
    for i in 0..256 {
        let distance = (i as f32 - 128.0).abs();
        let value = (1000.0 * (-distance * distance / 2000.0).exp()) as u32;

        red[i] = value;
        green[i] = (value as f32 * 1.1) as u32; // Slightly brighter green
        blue[i] = (value as f32 * 0.9) as u32;  // Slightly darker blue
        luma[i] = value;
    }

    Ok(serde_json::json!({
        "red": red.to_vec(),
        "green": green.to_vec(),
        "blue": blue.to_vec(),
        "luma": luma.to_vec(),
    }))
}

/// Get all available presets (builtin + user)
///
/// # Returns
/// * `Vec<Value>` - Array of preset objects
#[tauri::command]
pub fn get_presets(state: State<AppState>) -> Vec<serde_json::Value> {
    let library = state.preset_library.lock().unwrap();
    let presets = library.all();

    presets
        .iter()
        .map(|preset| {
            serde_json::json!({
                "id": preset.id,
                "name": preset.name,
                "group": preset.group,
                "description": preset.description,
                "is_builtin": preset.is_builtin,
            })
        })
        .collect()
}

/// Apply a preset to a photo
///
/// # Arguments
/// * `photo_id` - Photo ID to apply preset to
/// * `preset_id` - Preset ID to apply
///
/// # Returns
/// * `Ok(Value)` - The updated recipe as JSON
/// * `Err(String)` - Error message if application fails
#[tauri::command]
pub fn apply_preset(
    photo_id: String,
    preset_id: String,
    state: State<AppState>,
) -> Result<serde_json::Value, String> {
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    // Load current recipe
    let current_recipe_json = catalog
        .load_edit(&photo_id)
        .map_err(|e| format!("Failed to load current recipe: {}", e))?
        .unwrap_or_else(|| serde_json::to_string(&ocps_core::EditRecipe::default()).unwrap());

    let current_recipe: ocps_core::EditRecipe = serde_json::from_str(&current_recipe_json)
        .map_err(|e| format!("Failed to parse recipe: {}", e))?;

    // Get preset
    let library = state.preset_library.lock().unwrap();
    let all_presets = library.all();
    let preset = all_presets
        .iter()
        .find(|p| p.id == preset_id)
        .ok_or_else(|| format!("Preset not found: {}", preset_id))?;

    // Apply preset
    let new_recipe = ocps_core::PresetLibrary::apply(preset, &current_recipe);

    // Save the new recipe
    let new_recipe_json = serde_json::to_string(&new_recipe)
        .map_err(|e| format!("Failed to serialize recipe: {}", e))?;

    catalog
        .save_edit(&photo_id, &new_recipe_json)
        .map_err(|e| format!("Failed to save recipe: {}", e))?;

    // Return the new recipe
    serde_json::to_value(&new_recipe)
        .map_err(|e| format!("Failed to convert recipe to JSON: {}", e))
}

/// Get all keywords with usage count
#[tauri::command]
pub fn get_keywords(state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    let keywords = catalog
        .get_all_keywords_with_count()
        .map_err(|e| format!("Failed to get keywords: {}", e))?;

    let json_keywords = keywords
        .into_iter()
        .map(|(id, name, count)| {
            serde_json::json!({
                "id": id,
                "name": name,
                "count": count,
            })
        })
        .collect();

    Ok(json_keywords)
}

/// Add keyword to photos
#[tauri::command]
pub fn add_keyword_to_photos(
    state: State<AppState>,
    photo_ids: Vec<String>,
    keyword: String,
) -> Result<u32, String> {
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    // Get or create keyword
    let keyword_id = catalog
        .get_or_create_keyword(&keyword)
        .map_err(|e| format!("Failed to create keyword: {}", e))?;

    // Add to photos
    let count = catalog
        .batch_add_keywords(&photo_ids, &[keyword_id])
        .map_err(|e| format!("Failed to add keywords: {}", e))?;

    Ok(count)
}

/// Batch update rating for multiple photos
#[tauri::command]
pub fn batch_update_rating(
    state: State<AppState>,
    photo_ids: Vec<String>,
    rating: u8,
) -> Result<u32, String> {
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    catalog
        .batch_update_rating(&photo_ids, rating)
        .map_err(|e| format!("Failed to batch update rating: {}", e))
}

/// Batch update flag for multiple photos
#[tauri::command]
pub fn batch_update_flag(
    state: State<AppState>,
    photo_ids: Vec<String>,
    flag: String,
) -> Result<u32, String> {
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    catalog
        .batch_update_flag(&photo_ids, &flag)
        .map_err(|e| format!("Failed to batch update flag: {}", e))
}

/// Batch update color label for multiple photos
#[tauri::command]
pub fn batch_update_color_label(
    state: State<AppState>,
    photo_ids: Vec<String>,
    label: String,
) -> Result<u32, String> {
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    catalog
        .batch_update_color_label(&photo_ids, &label)
        .map_err(|e| format!("Failed to batch update color label: {}", e))
}

/// Get all photos with GPS coordinates
#[tauri::command]
pub fn get_geo_photos(state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    let photos = catalog
        .get_photos_with_gps()
        .map_err(|e| format!("Failed to get geo photos: {}", e))?;

    let json_photos = photos
        .into_iter()
        .map(|photo| {
            serde_json::json!({
                "id": photo.id,
                "file_name": photo.file_name,
                "lat": photo.lat,
                "lon": photo.lon,
                "rating": photo.rating,
            })
        })
        .collect();

    Ok(json_photos)
}

// ========== PART G: New Commands ==========

/// Import a Lightroom Classic catalog
#[tauri::command]
pub fn import_lightroom_catalog(
    state: State<AppState>,
    lrcat_path: String,
) -> Result<serde_json::Value, String> {
    let mut catalog_lock = state.catalog.lock().unwrap();

    let catalog = catalog_lock
        .as_mut()
        .ok_or("No catalog open. Import a folder first to create a catalog.".to_string())?;

    let lr_path = std::path::Path::new(&lrcat_path);
    let result = ocps_catalog::import_lightroom_catalog(lr_path, catalog, None)
        .map_err(|e| format!("Lightroom import failed: {}", e))?;

    Ok(serde_json::json!({
        "photos_imported": result.photos_imported,
        "photos_skipped": result.photos_skipped,
        "keywords_imported": result.keywords_imported,
        "collections_imported": result.collections_imported,
        "errors": result.errors,
        "warnings": result.warnings,
    }))
}

/// Import a Lightroom preset file (.lrtemplate or .xmp)
#[tauri::command]
pub fn import_preset_file(
    state: State<AppState>,
    path: String,
) -> Result<serde_json::Value, String> {
    let preset_path = std::path::Path::new(&path);

    let (name, settings) = ocps_xmp::import_preset_file(preset_path)
        .map_err(|e| format!("Failed to import preset: {}", e))?;

    // Convert XmpDevelopSettings to EditRecipe
    let mut recipe = ocps_core::EditRecipe::default();
    if let Some(temp) = settings.temperature {
        recipe.white_balance.temperature = temp as u32;
    }
    if let Some(tint) = settings.tint {
        recipe.white_balance.tint = tint;
    }
    if let Some(exp) = settings.exposure {
        recipe.exposure = exp;
    }
    if let Some(con) = settings.contrast {
        recipe.contrast = con;
    }
    if let Some(hl) = settings.highlights {
        recipe.highlights = hl;
    }
    if let Some(sh) = settings.shadows {
        recipe.shadows = sh;
    }
    if let Some(wh) = settings.whites {
        recipe.whites = wh;
    }
    if let Some(bl) = settings.blacks {
        recipe.blacks = bl;
    }
    if let Some(cl) = settings.clarity {
        recipe.clarity = cl;
    }
    if let Some(dh) = settings.dehaze {
        recipe.dehaze = dh;
    }
    if let Some(vib) = settings.vibrance {
        recipe.vibrance = vib;
    }
    if let Some(sat) = settings.saturation {
        recipe.saturation = sat;
    }

    // Create preset and save to library
    let preset = ocps_core::Preset {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.clone(),
        group: "Imported".to_string(),
        description: Some(format!("Imported from {}", preset_path.display())),
        recipe,
        applied_modules: vec!["all".to_string()],
        is_builtin: false,
    };

    let library = state.preset_library.lock().unwrap();
    library.save_preset(&preset)
        .map_err(|e| format!("Failed to save preset: {}", e))?;

    Ok(serde_json::json!({
        "id": preset.id,
        "name": preset.name,
        "group": preset.group,
        "description": preset.description,
    }))
}

/// Get all installed plugins
#[tauri::command]
pub fn get_plugins(state: State<AppState>) -> Vec<serde_json::Value> {
    let registry = state.plugin_registry.lock().unwrap();
    let plugins = registry.list_plugins();

    plugins
        .iter()
        .map(|manifest| {
            serde_json::json!({
                "id": manifest.id,
                "name": manifest.name,
                "version": manifest.version,
                "api_version": manifest.api_version,
                "plugin_type": manifest.plugin_type,
                "author": manifest.author,
                "description": manifest.description,
            })
        })
        .collect()
}

/// Rescan plugin directory
#[tauri::command]
pub fn scan_plugins(state: State<AppState>) -> Result<u32, String> {
    let plugin_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("openclaw-photo-studio")
        .join("plugins");

    let mut registry = state.plugin_registry.lock().unwrap();
    let count = registry
        .scan_directory(&plugin_dir)
        .map_err(|e| format!("Failed to scan plugins: {}", e))?;

    Ok(count as u32)
}

/// Get supported RAW formats
#[tauri::command]
pub fn get_supported_formats() -> Vec<String> {
    vec![
        "arw".to_string(),
        "nef".to_string(),
        "raf".to_string(),
        "dng".to_string(),
        "cr2".to_string(),
        "cr3".to_string(),
        "orf".to_string(),
        "rw2".to_string(),
    ]
}

/// Set XMP sidecar mode
#[tauri::command]
pub fn set_sidecar_mode(state: State<AppState>, mode: String) -> Result<(), String> {
    let sidecar_mode = match mode.as_str() {
        "auto" => SidecarMode::Auto,
        "manual" => SidecarMode::Manual,
        "readonly" => SidecarMode::ReadOnly,
        "disabled" => SidecarMode::Disabled,
        _ => return Err(format!("Invalid sidecar mode: {}", mode)),
    };

    let mut mode_lock = state.sidecar_mode.lock().unwrap();
    *mode_lock = sidecar_mode;

    Ok(())
}

/// Get XMP sidecar mode
#[tauri::command]
pub fn get_sidecar_mode(state: State<AppState>) -> String {
    let mode_lock = state.sidecar_mode.lock().unwrap();
    match *mode_lock {
        SidecarMode::Auto => "auto".to_string(),
        SidecarMode::Manual => "manual".to_string(),
        SidecarMode::ReadOnly => "readonly".to_string(),
        SidecarMode::Disabled => "disabled".to_string(),
    }
}

// ========== PART C: Preview/Thumbnail Commands ==========

/// Get thumbnail for a photo
///
/// # Arguments
/// * `photo_id` - Photo ID from catalog
/// * `max_size` - Maximum dimension (width or height) in pixels
///
/// # Returns
/// * `Ok(Value)` - JSON with { data: "base64...", width: N, height: N, from_cache: bool }
/// * `Err(String)` - Error message
#[tauri::command]
pub fn get_thumbnail(
    state: State<AppState>,
    photo_id: String,
    max_size: u32,
) -> Result<serde_json::Value, String> {
    // Check preview cache first
    let mut cache_lock = state.preview_cache.lock().unwrap();

    if let Some(cached) = cache_lock.get(&photo_id) {
        // Check if cached size matches (approximate check)
        if cached.width <= max_size && cached.height <= max_size {
            return Ok(serde_json::json!({
                "data": cached.data_base64,
                "width": cached.width,
                "height": cached.height,
                "from_cache": true,
            }));
        }
    }

    drop(cache_lock);

    // Not in cache - generate thumbnail
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    // Get photo path
    let photo = catalog
        .get_photo(&photo_id)
        .map_err(|e| format!("Failed to get photo: {}", e))?
        .ok_or("Photo not found".to_string())?;

    drop(catalog_lock);

    // Generate thumbnail
    let req = ocps_core::thumbnail_service::ThumbnailRequest {
        photo_path: photo.file_path.clone(),
        max_size,
        quality: 85,
    };

    let result = ocps_core::thumbnail_service::generate_thumbnail(&req)
        .map_err(|e| format!("Failed to generate thumbnail: {}", e))?;

    // Store in cache
    let mut cache_lock = state.preview_cache.lock().unwrap();
    cache_lock.put(
        &photo_id,
        ocps_core::preview_cache::CachedPreview {
            data_base64: result.data_base64.clone(),
            width: result.width,
            height: result.height,
            generated_at: std::time::SystemTime::now(),
        },
    );

    Ok(serde_json::json!({
        "data": result.data_base64,
        "width": result.width,
        "height": result.height,
        "from_cache": false,
    }))
}

/// Get full-size preview for a photo (for loupe/develop view)
///
/// # Arguments
/// * `photo_id` - Photo ID from catalog
///
/// # Returns
/// * `Ok(Value)` - JSON with { data: "base64...", width: N, height: N, from_cache: bool }
/// * `Err(String)` - Error message
#[tauri::command]
pub fn get_preview(
    state: State<AppState>,
    photo_id: String,
) -> Result<serde_json::Value, String> {
    // Use get_thumbnail with 2048px max size
    get_thumbnail(state, photo_id, 2048)
}

/// Invalidate preview cache for a photo
///
/// Called when edit recipe changes significantly
#[tauri::command]
pub fn invalidate_preview(
    state: State<AppState>,
    photo_id: String,
) -> Result<(), String> {
    let mut cache_lock = state.preview_cache.lock().unwrap();
    cache_lock.invalidate(&photo_id);
    Ok(())
}

/// Get cache statistics
///
/// # Returns
/// * `Ok(Value)` - JSON with { ram_entries: N, disk_size_bytes: N }
#[tauri::command]
pub fn get_cache_stats(
    state: State<AppState>,
) -> Result<serde_json::Value, String> {
    let cache_lock = state.preview_cache.lock().unwrap();

    Ok(serde_json::json!({
        "ram_entries": cache_lock.ram_entry_count(),
        "disk_size_bytes": cache_lock.disk_cache_size_bytes(),
    }))
}

/// Render preview with custom recipe (for live updates during editing)
///
/// This is called during slider drag and should be fast.
/// Results are NOT cached.
///
/// # Arguments
/// * `photo_id` - Photo ID
/// * `recipe` - Edit recipe JSON
/// * `max_size` - Maximum dimension
///
/// # Returns
/// * `Ok(Value)` - JSON with { data: "base64...", width: N, height: N }
#[tauri::command]
pub fn render_preview_with_recipe(
    state: State<AppState>,
    photo_id: String,
    recipe: serde_json::Value,
    max_size: u32,
) -> Result<serde_json::Value, String> {
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    // Get photo path
    let photo = catalog
        .get_photo(&photo_id)
        .map_err(|e| format!("Failed to get photo: {}", e))?
        .ok_or("Photo not found".to_string())?;

    let path = std::path::Path::new(&photo.file_path);
    drop(catalog_lock);

    // Parse recipe
    let edit_recipe: ocps_core::EditRecipe = serde_json::from_value(recipe)
        .map_err(|e| format!("Failed to parse recipe: {}", e))?;

    // Load and process image
    let is_raw = is_raw_file(path);

    let image = if is_raw {
        let raw = ocps_core::decode(path)
            .map_err(|e| format!("Failed to decode RAW: {:?}", e))?;

        let rgb = ocps_core::demosaic(&raw, ocps_core::DemosaicAlgorithm::Bilinear);

        // Convert u8 → u16 for pipeline
        let data_u16: Vec<u16> = rgb.data.iter().map(|&v| (v as u16) * 257).collect();

        ocps_core::RgbImage16::from_data(rgb.width, rgb.height, data_u16)
    } else {
        let img = image::open(path)
            .map_err(|e| format!("Failed to load image: {}", e))?;

        let rgb = img.to_rgb16();
        ocps_core::RgbImage16 {
            width: rgb.width(),
            height: rgb.height(),
            data: rgb.into_raw(),
        }
    };

    // Apply pipeline
    let output = ocps_core::ImageProcessor::process(&image, &edit_recipe);

    // Resize if needed
    let (final_width, final_height, final_data) = if output.width > max_size || output.height > max_size {
        let ratio = (max_size as f32) / output.width.max(output.height) as f32;
        let new_width = (output.width as f32 * ratio) as u32;
        let new_height = (output.height as f32 * ratio) as u32;

        let img = image::RgbImage::from_raw(output.width, output.height, output.data.clone())
            .ok_or("Failed to create image")?;

        let resized = image::imageops::resize(
            &img,
            new_width,
            new_height,
            image::imageops::FilterType::Triangle,
        );

        (new_width, new_height, resized.into_raw())
    } else {
        (output.width, output.height, output.data)
    };

    // Encode as JPEG
    let img = image::RgbImage::from_raw(final_width, final_height, final_data.clone())
        .ok_or("Failed to create final image")?;

    let mut buffer = std::io::Cursor::new(Vec::new());
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, 85);
    img.write_with_encoder(encoder)
        .map_err(|e| format!("JPEG encode failed: {}", e))?;

    let jpeg_data = buffer.into_inner();
    let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &jpeg_data);

    Ok(serde_json::json!({
        "data": base64_data,
        "width": final_width,
        "height": final_height,
    }))
}


/// Check for tethered camera (stub for Phase 7)
#[tauri::command]
pub fn check_tethered_camera() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "connected": false,
        "message": "Tethering coming in Phase 7"
    }))
}

// ========== PART A: Render Commands ==========

/// Render a preview image with applied edits
///
/// # Arguments
/// * `photo_id` - Photo ID from catalog
/// * `recipe` - Optional edit recipe (None = use saved recipe from catalog)
/// * `max_width` - Maximum width for preview
/// * `max_height` - Maximum height for preview
///
/// # Returns
/// * `Ok(Value)` - JSON with { data_uri, width, height, duration_ms }
/// * `Err(String)` - Error message
#[tauri::command]
pub async fn render_preview(
    state: tauri::State<'_, AppState>,
    photo_id: String,
    recipe: Option<serde_json::Value>,
    max_width: u32,
    max_height: u32,
) -> Result<serde_json::Value, String> {
    // Load photo path and recipe from catalog (drop lock before await)
    let (photo_path, edit_recipe) = {
        let catalog_lock = state.catalog.lock().unwrap();
        let catalog = catalog_lock
            .as_ref()
            .ok_or("No catalog open".to_string())?;

        let photo = catalog
            .get_photo(&photo_id)
            .map_err(|e| format!("Failed to get photo: {}", e))?
            .ok_or("Photo not found".to_string())?;

        let photo_path = photo.file_path.clone();

        // Load or use provided recipe
        let edit_recipe: ocps_core::EditRecipe = if let Some(recipe_value) = recipe {
            serde_json::from_value(recipe_value)
                .map_err(|e| format!("Failed to parse recipe: {}", e))?
        } else {
            // Load from catalog
            let recipe_json = catalog
                .load_edit(&photo_id)
                .map_err(|e| format!("Failed to load edit: {}", e))?;

            if let Some(json) = recipe_json {
                serde_json::from_str(&json).map_err(|e| format!("Failed to parse recipe: {}", e))?
            } else {
                ocps_core::EditRecipe::default()
            }
        };

        (photo_path, edit_recipe)
    }; // Lock dropped here

    // Render in background thread
    let path = std::path::PathBuf::from(photo_path);
    let result = tauri::async_runtime::spawn_blocking(move || {
        crate::render::render_photo(&path, &edit_recipe, max_width, max_height)
    })
    .await
    .map_err(|e| format!("Render task failed: {}", e))??;

    Ok(serde_json::json!({
        "data_uri": result.data_uri,
        "width": result.width,
        "height": result.height,
        "duration_ms": result.duration_ms,
    }))
}

/// Render thumbnail for a single photo (256x256 max)
///
/// # Arguments
/// * `photo_id` - Photo ID from catalog
///
/// # Returns
/// * `Ok(String)` - Data URI string
/// * `Err(String)` - Error message
#[tauri::command]
pub async fn render_thumbnail(
    state: tauri::State<'_, AppState>,
    photo_id: String,
) -> Result<String, String> {
    // Check preview cache first (drop lock immediately)
    {
        let mut cache_lock = state.preview_cache.lock().unwrap();
        if let Some(cached) = cache_lock.get(&photo_id) {
            return Ok(format!("data:image/jpeg;base64,{}", cached.data_base64));
        }
    }

    // Not in cache - render it
    let result_json = render_preview(state.clone(), photo_id.clone(), None, 256, 256).await?;

    let data_uri = result_json
        .get("data_uri")
        .and_then(|v| v.as_str())
        .ok_or("Missing data_uri in render result")?
        .to_string();

    // Cache it
    {
        let mut cache_lock = state.preview_cache.lock().unwrap();
        let base64_part = data_uri
            .strip_prefix("data:image/jpeg;base64,")
            .unwrap_or(&data_uri);
        cache_lock.put(
            &photo_id,
            ocps_core::preview_cache::CachedPreview {
                data_base64: base64_part.to_string(),
                width: result_json.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                height: result_json.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                generated_at: std::time::SystemTime::now(),
            },
        );
    }

    Ok(data_uri)
}

/// Render thumbnails for multiple photos in parallel
///
/// # Arguments
/// * `photo_ids` - Vector of photo IDs
///
/// # Returns
/// * `Ok(Vec<Value>)` - Array of { photo_id, data_uri } or { photo_id, error }
/// * `Err(String)` - Error message
#[tauri::command]
pub async fn render_thumbnails_batch(
    state: tauri::State<'_, AppState>,
    photo_ids: Vec<String>,
) -> Result<Vec<serde_json::Value>, String> {
    use rayon::prelude::*;

    // Load photo paths from catalog
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    let mut photo_paths: Vec<(String, String)> = Vec::new();
    for photo_id in &photo_ids {
        if let Ok(Some(photo)) = catalog.get_photo(photo_id) {
            photo_paths.push((photo_id.clone(), photo.file_path.clone()));
        }
    }

    drop(catalog_lock);

    // Render in parallel (limit to 4 concurrent to avoid OOM)
    let results: Vec<serde_json::Value> = photo_paths
        .par_iter()
        .map(|(photo_id, photo_path)| {
            let recipe = ocps_core::EditRecipe::default();
            let path = std::path::Path::new(photo_path);

            match crate::render::render_photo(path, &recipe, 256, 256) {
                Ok(result) => serde_json::json!({
                    "photo_id": photo_id,
                    "data_uri": result.data_uri,
                }),
                Err(e) => serde_json::json!({
                    "photo_id": photo_id,
                    "error": e,
                }),
            }
        })
        .collect();

    // Cache successful renders
    let mut cache_lock = state.preview_cache.lock().unwrap();
    for result in &results {
        if let (Some(photo_id), Some(data_uri)) = (
            result.get("photo_id").and_then(|v| v.as_str()),
            result.get("data_uri").and_then(|v| v.as_str()),
        ) {
            let base64_part = data_uri
                .strip_prefix("data:image/jpeg;base64,")
                .unwrap_or(data_uri);
            cache_lock.put(
                photo_id,
                ocps_core::preview_cache::CachedPreview {
                    data_base64: base64_part.to_string(),
                    width: 256,
                    height: 256,
                    generated_at: std::time::SystemTime::now(),
                },
            );
        }
    }

    Ok(results)
}

/// Get a single photo by ID (for loupe view)
#[tauri::command]
pub fn get_photo(
    state: tauri::State<'_, AppState>,
    photo_id: String,
) -> Result<serde_json::Value, String> {
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    let photo = catalog
        .get_photo(&photo_id)
        .map_err(|e| format!("Failed to get photo: {}", e))?
        .ok_or("Photo not found".to_string())?;

    Ok(serde_json::json!({
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
    }))
}

/// Get photo metadata including EXIF data
#[tauri::command]
pub fn get_photo_metadata(
    state: tauri::State<'_, AppState>,
    photo_id: String,
) -> Result<serde_json::Value, String> {
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock
        .as_ref()
        .ok_or("No catalog open".to_string())?;

    let photo = catalog
        .get_photo(&photo_id)
        .map_err(|e| format!("Failed to get photo: {}", e))?
        .ok_or("Photo not found".to_string())?;

    // Format camera info
    let camera = match (&photo.camera_make, &photo.camera_model) {
        (Some(make), Some(model)) => format!("{} {}", make, model),
        (Some(make), None) => make.clone(),
        (None, Some(model)) => model.clone(),
        _ => "Unknown".to_string(),
    };

    // Format file size
    let file_size_mb = photo.file_size as f64 / (1024.0 * 1024.0);

    Ok(serde_json::json!({
        "camera": camera,
        "lens": "—",  // TODO: Add to catalog schema
        "iso": "—",   // TODO: Add to catalog schema
        "aperture": "—",  // TODO: Add to catalog schema
        "shutter_speed": "—",  // TODO: Add to catalog schema
        "focal_length": "—",  // TODO: Add to catalog schema
        "date_taken": photo.date_taken.as_deref().unwrap_or("—"),
        "dimensions": match (photo.width, photo.height) {
            (Some(w), Some(h)) => format!("{} × {}", w, h),
            _ => "—".to_string(),
        },
        "file_size": format!("{:.1} MB", file_size_mb),
        "file_type": std::path::Path::new(&photo.file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("—")
            .to_uppercase(),
    }))
}

/// Auto white balance - simple implementation
#[tauri::command]
pub fn auto_white_balance(
    _state: tauri::State<'_, AppState>,
    _photo_id: String,
) -> Result<serde_json::Value, String> {
    // For now, return a reasonable default
    // Full implementation would analyze the image for grey areas
    Ok(serde_json::json!({
        "temperature": 5500,
        "tint": 0,
    }))
}

// ===== CATALOG MAINTENANCE COMMANDS =====

/// Verify catalog database integrity
#[tauri::command]
pub fn verify_catalog_integrity(state: State<AppState>) -> Result<bool, CommandError> {
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock.as_ref().ok_or_else(CommandError::catalog_not_open)?;

    catalog
        .verify_integrity()
        .map_err(|e| CommandError::internal_error("verify_catalog_integrity", e))
}

/// Vacuum catalog database to reclaim space
#[tauri::command]
pub fn vacuum_catalog(state: State<AppState>) -> Result<(), CommandError> {
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock.as_ref().ok_or_else(CommandError::catalog_not_open)?;

    catalog
        .vacuum()
        .map_err(|e| CommandError::internal_error("vacuum_catalog", e))
}

/// Backup catalog to specified path
#[tauri::command]
pub fn backup_catalog(state: State<AppState>, backup_path: String) -> Result<(), CommandError> {
    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock.as_ref().ok_or_else(CommandError::catalog_not_open)?;

    let path = std::path::Path::new(&backup_path);
    catalog
        .create_backup(path)
        .map_err(|e| CommandError::internal_error("backup_catalog", e))
}

// ===== UNDO/REDO COMMANDS =====

/// Undo last edit for a photo
#[tauri::command]
pub fn undo_edit(state: State<AppState>, photo_id: String) -> Result<serde_json::Value, CommandError> {
    let mut histories = state.histories.lock().unwrap();

    let history = histories
        .get_mut(&photo_id)
        .ok_or_else(|| CommandError::invalid_input("photo_id", "no edit history found"))?;

    let recipe = history
        .undo()
        .ok_or_else(|| CommandError::invalid_input("undo", "already at oldest state"))?;

    // Save to catalog
    let recipe_clone = recipe.clone();
    drop(histories);

    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock.as_ref().ok_or_else(CommandError::catalog_not_open)?;

    let recipe_json = serde_json::to_string(&recipe_clone)
        .map_err(|e| CommandError::internal_error("serialize recipe", e))?;

    catalog
        .save_edit(&photo_id, &recipe_json)
        .map_err(|e| CommandError::catalog_error("save_edit", e))?;

    serde_json::to_value(&recipe_clone)
        .map_err(|e| CommandError::internal_error("convert to JSON", e))
}

/// Redo last undone edit for a photo
#[tauri::command]
pub fn redo_edit(state: State<AppState>, photo_id: String) -> Result<serde_json::Value, CommandError> {
    let mut histories = state.histories.lock().unwrap();

    let history = histories
        .get_mut(&photo_id)
        .ok_or_else(|| CommandError::invalid_input("photo_id", "no edit history found"))?;

    let recipe = history
        .redo()
        .ok_or_else(|| CommandError::invalid_input("redo", "already at newest state"))?;

    // Save to catalog
    let recipe_clone = recipe.clone();
    drop(histories);

    let catalog_lock = state.catalog.lock().unwrap();
    let catalog = catalog_lock.as_ref().ok_or_else(CommandError::catalog_not_open)?;

    let recipe_json = serde_json::to_string(&recipe_clone)
        .map_err(|e| CommandError::internal_error("serialize recipe", e))?;

    catalog
        .save_edit(&photo_id, &recipe_json)
        .map_err(|e| CommandError::catalog_error("save_edit", e))?;

    serde_json::to_value(&recipe_clone)
        .map_err(|e| CommandError::internal_error("convert to JSON", e))
}

/// Get edit history for a photo
#[tauri::command]
pub fn get_edit_history(state: State<AppState>, photo_id: String) -> Result<Vec<serde_json::Value>, CommandError> {
    let histories = state.histories.lock().unwrap();

    let history = histories
        .get(&photo_id)
        .ok_or_else(|| CommandError::invalid_input("photo_id", "no edit history found"))?;

    let entries = history
        .entries_for_display()
        .into_iter()
        .map(|(description, is_current)| {
            serde_json::json!({
                "description": description,
                "is_current": is_current,
            })
        })
        .collect();

    Ok(entries)
}
