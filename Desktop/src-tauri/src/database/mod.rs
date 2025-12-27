// Database module
// Handles SQLite database operations and connection management

pub mod operations;
pub mod schema;

use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use std::sync::Arc;

pub type DatabasePool = Arc<Pool<SqliteConnectionManager>>;
pub type DatabaseConnection = PooledConnection<SqliteConnectionManager>;

/// Initialize the database connection pool
pub fn init_database(database_url: &str) -> Result<DatabasePool, String> {
    let manager = SqliteConnectionManager::file(database_url);

    let pool = Pool::new(manager).map_err(|e| format!("Failed to create database pool: {}", e))?;

    Ok(Arc::new(pool))
}

/// Get a connection from the pool
pub fn get_connection(pool: &DatabasePool) -> Result<DatabaseConnection, String> {
    pool.get()
        .map_err(|e| format!("Failed to get database connection: {}", e))
}

pub use operations::DatabaseOperations;
pub use schema::initialize_schema;
