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
            commands::compute_histogram,
            commands::get_presets,
            commands::apply_preset,
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
