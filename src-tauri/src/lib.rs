mod commands;

use std::sync::Mutex;

use berry_storage::Database;
use tauri::Manager;

/// Application-wide state managed by Tauri.
pub struct AppState {
    /// The migrated SQLite database, opened in the app data directory.
    pub db: Mutex<Database>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Open (and migrate) the SQLite database in the OS app data dir.
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db = Database::connect(&data_dir.join("berry.db"))?;
            app.manage(AppState { db: Mutex::new(db) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::add_folder,
            commands::list_folders,
            commands::remove_folder,
            commands::list_files,
            commands::query_files,
            commands::set_file_rating,
            commands::get_library_counts,
            commands::scan_folder,
            commands::rebuild_metadata,
            commands::search_files,
            commands::search_files_by_query,
            commands::list_distinct_models,
            commands::list_distinct_samplers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
