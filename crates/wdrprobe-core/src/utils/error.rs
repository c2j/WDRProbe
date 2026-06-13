// Error handling infrastructure
// Custom error types for the application

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WdrProbeError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Connection pool error: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    #[error("GaussDB error: {0}")]
    GaussDb(String),

    #[error("Threshold error: {0}")]
    Threshold(String),

    #[error("Audit error: {0}")]
    Audit(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<WdrProbeError> for String {
    fn from(error: WdrProbeError) -> Self {
        error.to_string()
    }
}
