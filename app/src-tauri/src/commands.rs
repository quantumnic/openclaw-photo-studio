use std::sync::Mutex;
use tauri::State;

/// Application state holding the catalog and edit clipboard
pub struct AppState {
    pub catalog: Mutex<Option<ocps_catalog::Catalog>>,
    pub clipboard: Mutex<Option<ocps_core::EditClipboard>>,
    pub preset_library: Mutex<ocps_core::PresetLibrary>,
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

        Self {
            catalog: Mutex::new(None),
            clipboard: Mutex::new(None),
            preset_library: Mutex::new(preset_library),
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

    catalog
        .save_edit(&photo_id, &recipe_json)
        .map_err(|e| format!("Failed to save edit: {}", e))
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

/// Batch export multiple photos to JPEG
#[tauri::command]
pub fn export_photos_batch(
    state: State<AppState>,
    photo_ids: Vec<String>,
    output_folder: String,
    quality: u32,
    resize_long_edge: Option<u32>,
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

    for photo_id in photo_ids.iter() {
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

        // Construct output path
        let input_path = std::path::Path::new(&photo.file_path);
        let file_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("photo");
        let output_path = output_dir.join(format!("{}.jpg", file_stem));

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
