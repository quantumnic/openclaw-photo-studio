use tauri::Manager;

mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(commands::AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::get_version,
            commands::get_catalog_info,
            commands::decode_raw_info,
            commands::import_folder,
            commands::get_photos,
            commands::update_rating,
            commands::update_flag,
            commands::update_color_label,
            commands::get_catalog_stats,
            commands::save_edit_recipe,
            commands::load_edit_recipe,
            commands::copy_edit,
            commands::paste_edit,
            commands::reset_edit,
            commands::export_photo_jpeg,
            commands::export_photos_batch,
            commands::compute_histogram,
            commands::get_presets,
            commands::apply_preset,
            commands::get_keywords,
            commands::add_keyword_to_photos,
            commands::batch_update_rating,
            commands::batch_update_flag,
            commands::batch_update_color_label,
            commands::get_geo_photos,
            commands::import_lightroom_catalog,
            commands::import_preset_file,
            commands::get_plugins,
            commands::scan_plugins,
            commands::get_supported_formats,
            commands::set_sidecar_mode,
            commands::get_sidecar_mode,
            commands::get_thumbnail,
            commands::get_preview,
            commands::invalidate_preview,
            commands::get_cache_stats,
            commands::render_preview_with_recipe,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running OpenClaw Photo Studio");
}
