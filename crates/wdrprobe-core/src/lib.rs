pub mod database;
pub mod models;
pub mod parsers;
pub mod progress;
pub mod utils;

// Re-export commonly used types
pub use database::{init_database, initialize_schema, DatabaseOperations, DatabasePool};
pub use models::*;
pub use progress::*;
pub use utils::*;

// Result type
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
