//! Plugin API v1.0 — Stable Plugin Interface
//!
//! This module defines the stable contract between OCPS and plugins.
//! Once released, this API MUST remain backwards compatible.
//!
//! Version 1.0.0 — Initial stable release

/// Plugin API version. Bump on breaking changes.
pub const PLUGIN_API_VERSION: u32 = 1;

/// Functions that the host provides to plugins (via WASM imports)
///
/// All functions use i32/f32/i64 (WASM primitive types) for ABI compatibility.
///
/// ## Logging
/// - `env.ocps_log(msg_ptr: i32, msg_len: i32)` - Log a message to host console
///
/// ## Image Access (requires `read_image` permission)
/// - `env.ocps_get_image_width() -> i32` - Get current image width
/// - `env.ocps_get_image_height() -> i32` - Get current image height
/// - `env.ocps_get_pixel(x: i32, y: i32, channel: i32) -> f32` - Get pixel value (0.0-1.0)
///   - channel: 0=R, 1=G, 2=B
/// - `env.ocps_set_pixel(x: i32, y: i32, channel: i32, value: f32)` - Set pixel value (requires `write_image`)
///
/// ## Metadata Access (requires `read_metadata` permission)
/// - `env.ocps_get_metadata(key_ptr: i32, key_len: i32, out_ptr: i32, out_max: i32) -> i32`
///   - Returns: bytes written, or -1 if key not found
///   - Common keys: "iso", "aperture", "shutter_speed", "focal_length", "camera_make", "camera_model"
///
/// ## UI Functions (always available)
/// - `env.ocps_show_progress(current: i32, total: i32)` - Update progress bar
/// - `env.ocps_show_message(msg_ptr: i32, msg_len: i32)` - Show toast/notification
///
/// ## Memory Management
/// - `env.ocps_alloc(size: i32) -> i32` - Allocate memory in plugin's linear memory
/// - `env.ocps_free(ptr: i32)` - Free allocated memory
pub mod host_functions {
    // This module is documentation-only.
    // Actual implementation is in host.rs via wasmtime linker.
}

/// Functions that plugins must export
///
/// ## Required Exports
/// - `plugin_init() -> i32` - Initialize plugin (0 = success, non-zero = error code)
/// - `plugin_info(out_ptr: i32) -> i32` - Write plugin info as JSON, return length
///
/// ## Optional Exports (depend on plugin type)
/// - `process_image(width: i32, height: i32) -> i32` - Process current image (for image_filter type)
/// - `get_parameters(out_ptr: i32) -> i32` - Return JSON parameter schema
/// - `on_photo_selected(photo_id_ptr: i32, photo_id_len: i32)` - Called when photo selection changes (for ui_panel type)
/// - `on_export(output_path_ptr: i32, output_path_len: i32) -> i32` - Export handler (for export type)
///
/// ## Plugin Info JSON Schema
/// ```json
/// {
///   "name": "My Plugin",
///   "version": "1.0.0",
///   "author": "Plugin Author",
///   "description": "What this plugin does",
///   "type": "image_filter",
///   "api_version": 1
/// }
/// ```
///
/// ## Parameter Schema (returned by `get_parameters`)
/// ```json
/// {
///   "parameters": [
///     {
///       "id": "intensity",
///       "type": "slider",
///       "label": "Intensity",
///       "min": 0.0,
///       "max": 1.0,
///       "default": 0.5
///     },
///     {
///       "id": "mode",
///       "type": "choice",
///       "label": "Mode",
///       "options": ["soft", "medium", "hard"],
///       "default": "medium"
///     }
///   ]
/// }
/// ```
pub mod plugin_exports {
    // This module is documentation-only.
    // Plugins implement these as WASM exports.
}

/// Plugin types supported by the API
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginType {
    /// Image processing filter (exports: process_image)
    ImageFilter,
    /// Import/Export handler (exports: on_export)
    ImportExport,
    /// Metadata processor (exports: process_metadata)
    Metadata,
    /// UI panel extension (exports: on_photo_selected, get_ui)
    UiPanel,
    /// Catalog extension (exports: on_catalog_change)
    Catalog,
    /// Integration with external services (may need network permission)
    Integration,
    /// AI/ML feature (exports: process_image with ML models)
    AiMl,
    /// Camera tethering (exports: start_tether, stop_tether)
    Tethering,
}

impl PluginType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ImageFilter => "image_filter",
            Self::ImportExport => "import_export",
            Self::Metadata => "metadata",
            Self::UiPanel => "ui_panel",
            Self::Catalog => "catalog",
            Self::Integration => "integration",
            Self::AiMl => "ai_ml",
            Self::Tethering => "tethering",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "image_filter" => Some(Self::ImageFilter),
            "import_export" => Some(Self::ImportExport),
            "metadata" => Some(Self::Metadata),
            "ui_panel" => Some(Self::UiPanel),
            "catalog" => Some(Self::Catalog),
            "integration" => Some(Self::Integration),
            "ai_ml" => Some(Self::AiMl),
            "tethering" => Some(Self::Tethering),
            _ => None,
        }
    }
}

/// Error codes returned by plugin functions
#[repr(i32)]
pub enum PluginErrorCode {
    Success = 0,
    InvalidParameter = 1,
    OutOfMemory = 2,
    PermissionDenied = 3,
    NotSupported = 4,
    InternalError = 5,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version_is_1() {
        assert_eq!(PLUGIN_API_VERSION, 1, "Plugin API version must be 1 for initial stable release");
    }

    #[test]
    fn test_plugin_type_roundtrip() {
        let types = [
            PluginType::ImageFilter,
            PluginType::ImportExport,
            PluginType::Metadata,
            PluginType::UiPanel,
            PluginType::Catalog,
            PluginType::Integration,
            PluginType::AiMl,
            PluginType::Tethering,
        ];

        for plugin_type in &types {
            let s = plugin_type.as_str();
            let parsed = PluginType::parse(s).expect("Should parse back");
            assert_eq!(&parsed, plugin_type, "Roundtrip failed for {:?}", plugin_type);
        }
    }

    #[test]
    fn test_plugin_type_from_str_invalid() {
        assert!(PluginType::parse("invalid").is_none());
        assert!(PluginType::parse("").is_none());
        assert!(PluginType::parse("ImageFilter").is_none()); // case sensitive
    }
}
