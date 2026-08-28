//! Tauri IPC commands exposed to the frontend.
//!
//! Each command is a thin wrapper: parse the request, call into the core
//! crates, and serialize the result. Business logic lives in the `berry-*`
//! crates, not here.

use serde::Serialize;
use tauri::State;

use crate::AppState;

/// Diagnostics shown on the M1 shell to prove the full chain works
/// (frontend → IPC → Rust → SQLite).
#[derive(Serialize)]
pub struct AppInfo {
    /// Crate version from `Cargo.toml`.
    pub app_version: String,
    /// The SQLite `PRAGMA user_version` after migration.
    pub schema_version: i64,
    /// Absolute path of the opened database (empty for in-memory).
    pub database_path: String,
}

#[tauri::command]
pub fn get_app_info(state: State<'_, AppState>) -> Result<AppInfo, String> {
    let db = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let schema_version = db.user_version().map_err(|e| e.to_string())?;
    let database_path = db
        .path()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    Ok(AppInfo {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version,
        database_path,
    })
}
