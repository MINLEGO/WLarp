use serde::ser::{Serialize, Serializer};
use std::fmt;

/// Erreur applicative sérialisable pour les commandes Tauri.
#[derive(Debug)]
pub struct AppError(pub String);

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AppError {}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError(format!("SQLite : {e}"))
    }
}
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError(format!("IO : {e}"))
    }
}
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError(format!("JSON : {e}"))
    }
}
impl From<std::str::Utf8Error> for AppError {
    fn from(e: std::str::Utf8Error) -> Self {
        AppError(format!("UTF-8 : {e}"))
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

#[allow(dead_code)]
pub fn msg<T>(m: impl Into<String>) -> Result<T> {
    Err(AppError(m.into()))
}
