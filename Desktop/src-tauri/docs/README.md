# WDRProbe Desktop - Backend Documentation

## Overview

WDRProbe Desktop is a Tauri-based desktop application for analyzing GaussDB WDR (Workload Diagnosis Report) files. The backend is written in Rust and provides database management, WDR parsing, SQL analysis, and performance comparison capabilities.

## Technology Stack

- **Rust** 1.75+ - Core backend language
- **Tauri** 2.x - Desktop application framework
- **SQLite** (rusqlite) - Embedded database
- **serde** - JSON serialization
- **regex** - SQL parsing
- **chrono** - Date/time handling

## Project Structure

```
src-tauri/
├── src/
│   ├── main.rs              # Application entry point, Tauri setup
│   ├── lib.rs               # Library exports
│   ├── database/            # Database layer
│   │   ├── mod.rs           # Module exports
│   │   ├── pool.rs          # Connection pooling
│   │   ├── schema.rs        # SQLite schema definitions
│   │   └── operations.rs    # CRUD operations
│   ├── models/              # Data models
│   │   ├── mod.rs           # Module exports
│   │   ├── dashboard.rs     # Dashboard metrics
│   │   ├── report.rs        # WDR report models
│   │   ├── comparison.rs    # Comparison models
│   │   ├── execution_plan.rs # SQL execution plan models
│   │   ├── threshold.rs     # Performance threshold models
│   │   ├── audit.rs         # SQL audit models
│   │   └── export.rs        # Export/import models
│   ├── commands/            # Tauri IPC commands
│   │   ├── mod.rs           # Command exports
│   │   ├── dashboard.rs     # Dashboard commands
│   │   ├── reports.rs       # WDR report commands
│   │   ├── execution_plan.rs # Execution plan commands
│   │   ├── comparison.rs    # Comparison commands
│   │   ├── threshold.rs     # Threshold commands
│   │   ├── audit.rs         # SQL audit commands
│   │   └── export.rs        # Export/import commands
│   ├── parsers/             # WDR file parsers
│   │   ├── mod.rs           # Parser exports
│   │   ├── wdr_parser.rs    # HTML WDR parser
│   │   ├── complete_wdr_parser.rs # Full WDR parser
│   │   └── sql_parser.rs    # SQL text parser
│   └── utils/               # Utility functions
│       ├── mod.rs           # Utility exports
│       ├── audit.rs         # Audit logging utilities
│       └── gaussdb.rs       # GaussDB-specific utilities
├── tests/                   # Unit and integration tests
├── Cargo.toml               # Rust dependencies
└── tauri.conf.json          # Tauri configuration
```

## Database Schema

### Tables

1. **wdr_reports** - Imported WDR report metadata
2. **top_sqls** - Top SQL statements from WDR reports
3. **wdr_comparisons** - Report comparison records
4. **comparison_sql_details** - Detailed SQL comparisons
5. **threshold_configs** - Performance threshold configurations
6. **threshold_history** - Threshold change history
7. **sql_audit_issues** - SQL audit findings
8. **audit_logs** - Audit trail for all operations

### Indexes

- `idx_wdr_reports_instance` - Query by instance name
- `idx_wdr_reports_time` - Query by generation time
- `idx_top_sqls_report` - Query SQL by report
- `idx_audit_issues_report` - Query issues by report
- `idx_audit_issues_status` - Query issues by status
- `idx_audit_logs_entity` - Query logs by entity type/id
- `idx_audit_logs_timestamp` - Query logs by time

## IPC Commands (Frontend → Backend)

All commands are callable from the frontend using `invoke()` or Tauri's IPC bridge.

### Dashboard Commands

- `get_instance_summaries()` - Get list of all instances with metrics
- `get_dashboard_metrics(instance_name?)` - Get dashboard statistics

### Report Management Commands

- `import_wdr_report(file_path, instance_name)` - Import WDR file
- `get_wdr_reports(limit?, offset?, sort_by?)` - List reports
- `get_wdr_report_detail(id)` - Get report with SQL details
- `delete_wdr_report(id)` - Delete a report

### Execution Plan Commands

- `get_wdr_hot_sqls(report_id, limit?)` - Get top SQLs
- `get_execution_plan(sql_id)` - Get execution plan for SQL
- `analyze_execution_plan(sql_id)` - Analyze plan with suggestions

### Comparison Commands

- `create_comparison(source_report_id, target_report_id, type?, custom_name?)` - Create comparison
- `get_comparisons(limit?, offset?, sort_by?)` - List comparisons
- `get_comparison_detail(id)` - Get comparison details
- `get_comparison_metrics(comparison_id, category)` - Get comparison metrics
- `delete_comparison(id)` - Delete comparison

### Threshold Commands

- `get_threshold_configs()` - Get all thresholds
- `get_threshold_template(template_name)` - Get template
- `update_threshold(updates)` - Update thresholds (batch)
- `reset_to_template(template_name)` - Reset to template

### Audit Commands

- `run_sql_audit(report_ids?, include_resolved?, audit_types?)` - Run audit
- `get_sql_audit_issues(report_id?, status?, severity?, issue_type?, limit?, offset?, sort_by?)` - List issues
- `update_audit_issue_status(issue_id, status, resolved_by, resolution_note?)` - Update issue
- `bulk_update_audit_issues(issue_ids, status, resolved_by, resolution_note?)` - Bulk update
- `get_audit_summary()` - Get issue summary

### Export/Import Commands

- `export_wdr_report(report_id, format, include_sql_details, include_comparison_data, export_path?)` - Export report
- `export_comparison(comparison_id, format, export_path?)` - Export comparison
- `import_data(import_path, validate_only, overwrite_existing, import_types)` - Import data
- `validate_data_integrity(check_type, entity_type, entity_id?, expected_hash?)` - Validate integrity

## Data Models

### Core Models

- `WdrReport` - WDR report metadata
- `TopSql` - SQL statement with metrics
- `WdrComparison` - Comparison record
- `SqlExecutionPlan` - Execution plan tree
- `ThresholdConfig` - Performance threshold
- `SqlAuditIssue` - Audit finding
- `AuditLog` - Audit trail entry

### DTO Pattern (Constitution IV)

All request/response structs follow the DTO pattern:
- Structs are in `models/` directory
- All fields are public
- Serde serialization/deserialization
- Clone derived for pass-by-value

## Audit Logging (Constitution IX)

All write operations are logged to the `audit_logs` table:

```rust
DatabaseOperations::create_audit_log(pool_ref, &AuditLog {
    id: 0,
    timestamp: chrono::Utc::now().to_rfc3339(),
    user_id: None,
    action: "OPERATION_NAME".to_string(),
    entity_type: "entity_type".to_string(),
    entity_id: Some(id),
    old_value: None,
    new_value: Some(json_string),
    ip_address: None,
    success: true,
    error_message: None,
    details: Some(description),
});
```

## Error Handling

All errors are returned as `String` for simplicity in IPC communication:

```rust
#[tauri::command(rename_all = "camelCase")]
pub async fn some_command(
    pool: State<'_, DatabasePool>,
) -> Result<ResponseType, String> {
    // Return Ok(data) on success
    // Return Err(error_message) on failure
}
```

## Testing

### Run all tests:
```bash
cargo test
```

### Run specific test:
```bash
cargo test test_name
```

### Run with output:
```bash
cargo test -- --nocapture
```

## Building

### Development build:
```bash
cargo build
```

### Release build:
```bash
cargo build --release
```

### Tauri build:
```bash
npm run tauri build
```

## Performance Considerations

1. **Database Connection Pooling** - Using `r2d2` for SQLite connections
2. **Indexing** - Key columns are indexed for fast queries
3. **Pagination** - Large result sets support limit/offset
4. **Async Processing** - Long-running operations use async

## Configuration

Configuration is managed through:
- `tauri.conf.json` - Tauri app configuration
- `Cargo.toml` - Rust dependencies
- Environment variables for runtime configuration

## Frontend Integration

The frontend (React + TypeScript) calls backend commands via Tauri's IPC:

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Call a command
const result = await invoke('getWdrReports', {
  limit: 50,
  offset: 0,
  sortBy: 'generationTime'
});
```

## License

This project is part of WDRProbe Desktop.
