# Quickstart: WDRProbe Desktop Backend Implementation

**Date**: 2025-12-22
**Feature**: Implement Desktop Tauri Backend

## Overview

This guide provides step-by-step instructions for implementing the Tauri Rust backend for WDRProbe desktop application. The backend replaces mock API calls with real functionality including WDR report parsing, SQLite storage, execution plan analysis, and threshold management.

## Prerequisites

### Development Environment
- **Rust**: Version 1.75 or higher
- **Node.js**: Version 18 or higher (for frontend)
- **Tauri CLI**: Install with `cargo install tauri-cli`
- **Git**: Latest version

### System Requirements
- **OS**: Windows 10+, macOS 10.15+, or Linux (Ubuntu 20.04+)
- **Memory**: Minimum 4GB RAM, 8GB recommended
- **Storage**: 2GB free space for database and files

### Knowledge Requirements
- Rust programming language
- Tauri framework basics
- SQLite database operations
- SQL parsing and execution plan analysis

## Project Structure

```
Desktop/
├── src-tauri/                    # Tauri Rust backend
│   ├── src/
│   │   ├── commands/            # IPC command implementations
│   │   ├── database/            # SQLite schema and operations
│   │   ├── parsers/             # File parsing logic
│   │   ├── models/              # Data models
│   │   └── utils/               # Utility functions
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── frontend/                    # React frontend (already implemented)
│   ├── src/
│   │   ├── services/            # API service layer (needs Tauri IPC integration)
│   │   └── ...
│   └── package.json
│
└── tests/                       # Test directories
    ├── unit/
    ├── integration/
    └── contract/
```

## Implementation Steps

### Step 1: Setup Rust Dependencies

Edit `Desktop/src-tauri/Cargo.toml`:

```toml
[package]
name = "wdrprobe-desktop"
version = "0.1.0"
edition = "2021"

[dependencies]
# Tauri dependencies
tauri = { version = "1.5", features = ["api-all"] }
tauri-build = { version = "1.5" }

# Database
rusqlite = { version = "0.31", features = ["bundled", "chrono", "serde_json"] }
r2d2 = "0.8"
r2d2-sqlite = "0.23"

# Parsing
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
scraper = "0.17"
regex = "1.10"
nom = "7.1"
memmap2 = "0.9"

# Async runtime
tokio = { version = "1.34", features = ["full"] }
tokio-util = { version = "0.7", features = ["codec"] }

# Utilities
thiserror = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }

# Testing
mockall = { version = "0.12", optional = true }
tempfile = { version = "3.8", optional = true }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
test = ["mockall", "tempfile"]

[dev-dependencies]
rstest = "0.18"
```

### Step 2: Initialize Database Module

Create `Desktop/src-tauri/src/database/mod.rs`:

```rust
pub mod schema;
pub mod operations;

use rusqlite::{Connection, Result};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use std::sync::Arc;

pub type DatabasePool = Arc<Pool<SqliteConnectionManager>>;
pub type DatabaseConnection = PooledConnection<SqliteConnectionManager>;

/// Initialize the database connection pool
pub fn init_database(database_url: &str) -> Result<DatabasePool, String> {
    let manager = SqliteConnectionManager::file(database_url)
        .withpragma("journal_mode", "WAL")
        .withpragma("synchronous", "NORMAL")
        .withpragma("foreign_keys", "ON");

    let pool = Pool::new(manager)
        .map_err(|e| format!("Failed to create database pool: {}", e))?;

    Ok(Arc::new(pool))
}

/// Get a connection from the pool
pub fn get_connection(pool: &DatabasePool) -> Result<DatabaseConnection, String> {
    pool.get()
        .map_err(|e| format!("Failed to get database connection: {}", e))
}
```

### Step 3: Create Database Schema

Create `Desktop/src-tauri/src/database/schema.rs`:

```rust
use rusqlite::{Connection, Result};

/// Initialize the database schema
pub fn initialize_schema(conn: &Connection) -> Result<()> {
    // Enable foreign keys
    conn.execute_batch("PRAGMA foreign_keys = ON")?;

    // Create tables
    conn.execute_batch(r#"
        -- WDR Reports
        CREATE TABLE IF NOT EXISTS wdr_reports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            instance_name TEXT NOT NULL,
            generation_time DATETIME NOT NULL,
            snapshot_start DATETIME NOT NULL,
            snapshot_end DATETIME NOT NULL,
            file_path TEXT,
            file_size INTEGER,
            status TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        -- Top SQL
        CREATE TABLE IF NOT EXISTS top_sqls (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            report_id INTEGER NOT NULL,
            sql_id TEXT,
            sql_text TEXT NOT NULL,
            executions INTEGER NOT NULL,
            total_elapsed_time REAL NOT NULL,
            cpu_time REAL NOT NULL,
            io_time REAL NOT NULL,
            buffer_gets INTEGER NOT NULL,
            disk_reads INTEGER NOT NULL,
            rows_processed INTEGER NOT NULL,
            first_load_time DATETIME NOT NULL,
            last_load_time DATETIME NOT NULL,
            is_hot_sql BOOLEAN NOT NULL DEFAULT 0,
            rank_by_time INTEGER,
            FOREIGN KEY (report_id) REFERENCES wdr_reports(id)
        );

        -- Threshold Configurations
        CREATE TABLE IF NOT EXISTS threshold_configs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category TEXT NOT NULL,
            data_type TEXT NOT NULL,
            config_key TEXT NOT NULL,
            value REAL NOT NULL,
            default_value REAL NOT NULL,
            min_value REAL,
            max_value REAL,
            description TEXT,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_by TEXT
        );

        -- Audit Logs
        CREATE TABLE IF NOT EXISTS audit_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            user_id TEXT,
            action TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            entity_id INTEGER,
            old_value TEXT,
            new_value TEXT,
            ip_address TEXT,
            success BOOLEAN NOT NULL,
            error_message TEXT,
            details TEXT
        );

        -- Indexes
        CREATE INDEX IF NOT EXISTS idx_wdr_reports_instance ON wdr_reports(instance_name);
        CREATE INDEX IF NOT EXISTS idx_top_sqls_report_id ON top_sqls(report_id);
        CREATE INDEX IF NOT EXISTS idx_threshold_configs_category ON threshold_configs(category);
    "#)?;

    Ok(())
}
```

### Step 4: Implement Data Models

Create `Desktop/src-tauri/src/models/mod.rs`:

```rust
pub mod report;
pub mod threshold;
pub mod audit;

pub use report::*;
pub use threshold::*;
pub use audit::*;
```

Create `Desktop/src-tauri/src/models/report.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WdrReport {
    pub id: i64,
    pub instance_name: String,
    pub generation_time: String,
    pub snapshot_start: String,
    pub snapshot_end: String,
    pub file_path: Option<String>,
    pub file_size: Option<u64>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopSql {
    pub id: i64,
    pub report_id: i64,
    pub sql_id: Option<String>,
    pub sql_text: String,
    pub executions: u64,
    pub total_elapsed_time: f64,
    pub cpu_time: f64,
    pub io_time: f64,
    pub buffer_gets: u64,
    pub disk_reads: u64,
    pub rows_processed: u64,
    pub is_hot_sql: bool,
    pub rank_by_time: Option<i32>,
}
```

### Step 5: Implement IPC Commands

Create `Desktop/src-tauri/src/commands/dashboard.rs`:

```rust
use crate::database::DatabasePool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InstanceSummary {
    pub instance_name: String,
    pub status: String,
    pub health_score: i32,
    pub active_issues: i32,
    pub report_count: u64,
}

#[tauri::command]
pub async fn get_instance_summaries(
    pool: tauri::State<'_, DatabasePool>,
) -> Result<Vec<InstanceSummary>, String> {
    let conn = crate::database::get_connection(&pool)
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT instance_name, COUNT(*) as report_count
             FROM wdr_reports
             GROUP BY instance_name"
        )
        .map_err(|e| e.to_string())?;

    let report_iter = stmt
        .query_map([], |row| {
            Ok(InstanceSummary {
                instance_name: row.get("instance_name")?,
                status: "Healthy".to_string(),
                health_score: 90,
                active_issues: 0,
                report_count: row.get("report_count")?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut summaries = Vec::new();
    for summary in report_iter {
        summaries.push(summary.map_err(|e| e.to_string())?);
    }

    Ok(summaries)
}
```

Create `Desktop/src-tauri/src/commands/reports.rs`:

```rust
use crate::database::DatabasePool;
use crate::models::{WdrReport, TopSql};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WdrReportListResponse {
    pub reports: Vec<WdrReport>,
    pub total: i64,
}

#[tauri::command]
pub async fn get_wdr_reports(
    pool: tauri::State<'_, DatabasePool>,
) -> Result<WdrReportListResponse, String> {
    let conn = crate::database::get_connection(&pool)
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT * FROM wdr_reports ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;

    let report_iter = stmt
        .query_map([], |row| {
            Ok(WdrReport {
                id: row.get("id")?,
                instance_name: row.get("instance_name")?,
                generation_time: row.get("generation_time")?,
                snapshot_start: row.get("snapshot_start")?,
                snapshot_end: row.get("snapshot_end")?,
                file_path: row.get("file_path")?,
                file_size: row.get("file_size")?,
                status: row.get("status")?,
                created_at: row.get("created_at")?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut reports = Vec::new();
    for report in report_iter {
        reports.push(report.map_err(|e| e.to_string())?);
    }

    Ok(WdrReportListResponse {
        reports,
        total: reports.len() as i64,
    })
}
```

### Step 6: Implement WDR File Parser

Create `Desktop/src-tauri/src/parsers/wdr_parser.rs`:

```rust
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedWdrReport {
    pub instance_name: String,
    pub generation_time: String,
    pub snapshot_start: String,
    pub snapshot_end: String,
    pub sql_statistics: Vec<SqlStatistic>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SqlStatistic {
    pub sql_text: String,
    pub executions: u64,
    pub total_elapsed_time: f64,
    pub cpu_time: f64,
    pub buffer_gets: u64,
    pub disk_reads: u64,
}

/// Parse HTML WDR report
pub fn parse_html_wdr_report(file_path: &str) -> Result<ParsedWdrReport, String> {
    // Read file
    let mut file = File::open(file_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    let mut html_content = String::new();
    file.read_to_string(&mut html_content)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Parse HTML
    let document = Html::parse_document(&html_content);

    // Extract instance name
    let instance_selector = Selector::parse(".instance-name").unwrap();
    let instance_name = document
        .select(&instance_selector)
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or_else(|| "Unknown".to_string());

    // Extract SQL statistics (example selector - adjust based on actual WDR format)
    let sql_selector = Selector::parse(".sql-stat").unwrap();
    let sql_elements = document.select(&sql_selector);

    let mut sql_statistics = Vec::new();
    for element in sql_elements {
        // Parse each SQL statistic
        // This is a simplified example - actual implementation will vary
        sql_statistics.push(SqlStatistic {
            sql_text: element.text().collect(),
            executions: 0,
            total_elapsed_time: 0.0,
            cpu_time: 0.0,
            buffer_gets: 0,
            disk_reads: 0,
        });
    }

    Ok(ParsedWdrReport {
        instance_name,
        generation_time: chrono::Utc::now().to_rfc3339(),
        snapshot_start: chrono::Utc::now().to_rfc3339(),
        snapshot_end: chrono::Utc::now().to_rfc3339(),
        sql_statistics,
    })
}
```

### Step 7: Implement Threshold Configuration

Create `Desktop/src-tauri/src/commands/threshold.rs`:

```rust
use crate::database::DatabasePool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub id: i64,
    pub category: String,
    pub data_type: String,
    pub config_key: String,
    pub value: f64,
    pub default_value: f64,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub description: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThresholdUpdateRequest {
    pub config_key: String,
    pub value: f64,
    pub changed_by: String,
    pub change_reason: String,
}

#[tauri::command]
pub async fn update_threshold(
    pool: tauri::State<'_, DatabasePool>,
    request: ThresholdUpdateRequest,
) -> Result<String, String> {
    // Validate DTO format (Constitution IV)
    if request.changed_by.is_empty() {
        return Err("changed_by is required".to_string());
    }
    if request.change_reason.is_empty() {
        return Err("change_reason is required".to_string());
    }

    let conn = crate::database::get_connection(&pool)
        .map_err(|e| e.to_string())?;

    // Update threshold
    conn.execute(
        "UPDATE threshold_configs SET value = ?, updated_at = CURRENT_TIMESTAMP, updated_by = ? WHERE config_key = ?",
        &[&request.value.to_string(), &request.changed_by, &request.config_key],
    )
    .map_err(|e| e.to_string())?;

    // Log to audit (Constitution IX)
    conn.execute(
        "INSERT INTO audit_logs (action, entity_type, new_value, user_id, success, details) VALUES (?, ?, ?, ?, ?, ?)",
        &[
            "ThresholdUpdate",
            "threshold",
            &request.value.to_string(),
            &request.changed_by,
            &"true",
            &request.change_reason,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok("Threshold updated successfully".to_string())
}
```

### Step 8: Register Commands in lib.rs

Update `Desktop/src-tauri/src/lib.rs`:

```rust
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/commands

mod commands;
mod database;
mod models;
mod parsers;
mod utils;

use commands::dashboard::{get_instance_summaries};
use commands::reports::{get_wdr_reports};
use commands::threshold::{update_threshold, get_threshold_configs};
use database::{init_database, DatabasePool, initialize_schema};
use tauri::Manager;

#[cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

pub type Result<T> = std::result::Result<T, anyhow::Error>;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path_resolver()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            std::fs::create_dir_all(&app_data_dir)
                .expect("Failed to create app data directory");

            let db_path = app_data_dir.join("wdrprobe.db");

            // Initialize database
            let pool = init_database(db_path.to_str().unwrap())
                .expect("Failed to initialize database");

            // Initialize schema
            let conn = pool.get()
                .expect("Failed to get database connection");
            initialize_schema(&conn)
                .expect("Failed to initialize schema");

            // Store database pool in app state
            app.manage(pool);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_instance_summaries,
            get_wdr_reports,
            update_threshold,
            get_threshold_configs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Step 9: Implement Frontend Service Layer

Update `Desktop/frontend/src/services/api.ts` (replace mock calls):

```typescript
// Dashboard API
export const getInstanceSummaries = async (): Promise<InstanceSummary[]> => {
    return await window.__TAURI__.invoke<InstanceSummary[]>('get_instance_summaries');
};

// Reports API
export const getWdrReports = async (): Promise<WdrReport[]> => {
    return await window.__TAURI__.invoke<WdrReport[]>('get_wdr_reports');
};

// Threshold API
export const updateThreshold = async (
    configKey: string,
    value: number,
    changedBy: string,
    changeReason: string
): Promise<void> => {
    await window.__TAURI__.invoke('update_threshold', {
        configKey,
        value,
        changedBy,
        changeReason
    });
};
```

### Step 10: Run Tests

Create `Desktop/src-tauri/tests/dashboard_test.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_get_instance_summaries() {
        // Setup test database
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = init_database(db_path.to_str().unwrap()).unwrap();

        // Call function
        let result = get_instance_summaries(pool.clone()).await;

        // Assert
        assert!(result.is_ok());
    }
}
```

Run tests:
```bash
cd Desktop/src-tauri
cargo test
```

## Running the Application

### Development Mode

1. **Start the backend**:
```bash
cd Desktop/src-tauri
cargo tauri dev
```

2. **Or start frontend separately** (if needed):
```bash
cd Desktop
npm run tauri dev
```

### Production Build

```bash
cd Desktop
npm run tauri build
```

The build artifacts will be stored in `Desktop/src-tauri/target/release/bundle/`.

## Common Issues and Solutions

### Issue: Database locked
**Solution**: Enable WAL mode in database initialization

### Issue: Tauri commands not found
**Solution**: Ensure commands are registered in `lib.rs`

### Issue: Frontend can't call backend
**Solution**: Check that `window.__TAURI__` is available and commands are properly typed

### Issue: File parsing errors
**Solution**: Verify file format matches expected WDR structure

## Performance Tips

1. **Use connection pooling** for database operations
2. **Enable WAL mode** for better concurrency
3. **Use prepared statements** for repeated queries
4. **Implement caching** for frequently accessed data
5. **Use async/await** for I/O operations
6. **Stream large file imports** to avoid memory issues

## Next Steps

1. Implement remaining IPC commands
2. Add comprehensive test coverage
3. Optimize database queries
4. Add error handling and validation
5. Implement progress reporting for long operations
6. Add export/import functionality
7. Integrate with GaussDB for EXPLAIN analysis

## References

- [Tauri Documentation](https://tauri.app/)
- [Rust SQLite (rusqlite)](https://docs.rs/rusqlite/)
- [Serde (Serialization)](https://serde.rs/)
- [GaussDB Documentation](../docs/gaussdb.md)
- [IPC Interface Design](../docs/desktop-IPC.md)
- [Constitution](../.specify/memory/constitution.md)

## Support

For issues or questions:
1. Check the [FAQ](../docs/FAQ.md)
2. Review the [troubleshooting guide](../docs/troubleshooting.md)
3. Open an issue in the repository
