use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

/// État global partagé aux commandes Tauri.
pub struct AppState {
    pub db: Mutex<Connection>,
    /// `%APPDATA%\com.wlarp.app` — contient `data.sqlite` et `files/`.
    pub app_data: PathBuf,
}