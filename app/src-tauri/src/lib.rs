use tauri::Manager;

mod commands;
mod render;
mod perf;

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
            commands::open_catalog,
            commands::close_catalog,
            commands::new_catalog,
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
            commands::check_tethered_camera,
            commands::discover_cameras,
            commands::connect_camera,
            commands::disconnect_camera,
            commands::tether_capture,
            commands::render_preview,
            commands::render_thumbnail,
            commands::render_thumbnails_batch,
            commands::get_photo,
            commands::get_photo_metadata,
            commands::auto_white_balance,
            commands::verify_catalog_integrity,
            commands::vacuum_catalog,
            commands::backup_catalog,
            commands::undo_edit,
            commands::redo_edit,
            commands::get_edit_history,
            commands::get_display_info,
            commands::get_marketplace_plugins,
            commands::search_marketplace,
            commands::install_plugin,
            commands::uninstall_plugin,
            commands::merge_hdr_photos,
            commands::semantic_search,
            commands::enqueue_exports,
            commands::get_export_queue_status,
            commands::cancel_export_job,
            commands::retry_failed_exports,
            commands::get_all_export_jobs,
            commands::get_perf_stats,
            commands::get_recent_perf_events,
            commands::clear_perf_stats,
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
