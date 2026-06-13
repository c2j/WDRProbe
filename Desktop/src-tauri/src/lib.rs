// WDRProbe Desktop Library
// Tauri command layer + adapters (core logic in wdrprobe-core)

pub mod adapters;
pub mod commands;

// Re-export core modules for backwards compatibility
// This allows tests and code to use wdrprobe_desktop_lib::models::Foo etc.
pub use wdrprobe_core::database;
pub use wdrprobe_core::models;
pub use wdrprobe_core::parsers;
pub use wdrprobe_core::progress;
pub use wdrprobe_core::utils;

// Convenience re-exports (matching the old lib.rs pattern)
pub use wdrprobe_core::database::{init_database, initialize_schema, DatabaseOperations, DatabasePool};
pub use wdrprobe_core::models::*;
pub use wdrprobe_core::progress::*;
pub use wdrprobe_core::utils::*;

// Result type for the library
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
