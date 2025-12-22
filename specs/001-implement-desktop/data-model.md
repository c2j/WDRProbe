# Data Model: WDRProbe Desktop Backend

**Date**: 2025-12-22
**Feature**: Implement Desktop Tauri Backend

## Overview

This document defines the data models for the WDRProbe desktop application backend. All models are implemented as Rust structs with Serde serialization for IPC communication between frontend and backend.

## Core Entities

### 1. WDR Report

Represents a single WDR (Workload Diagnosis Report) imported from a database instance.

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WdrReport {
    pub id: i64,                                    // Unique identifier
    pub instance_name: String,                      // Database instance name
    pub generation_time: DateTime<Utc>,             // When report was generated
    pub snapshot_start: DateTime<Utc>,              // Snapshot period start
    pub snapshot_end: DateTime<Utc>,                // Snapshot period end
    pub file_path: Option<String>,                  // Original file location
    pub file_size: Option<u64>,                     // File size in bytes
    pub status: ReportStatus,                       // Import status
    pub created_at: DateTime<Utc>,                  // Import timestamp
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ReportStatus {
    SuccessfullyImported,
    ImportFailed(String),    // Error message if failed
    PartiallyImported,       // Some data extracted
}
```

**Relationships**:
- One WDR Report has many Top SQL entries
- One WDR Report has many Object Statistics
- One WDR Report has one Efficiency Metric
- One WDR Report has one Load Profile

### 2. Efficiency Metrics

Performance efficiency indicators for a WDR report.

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EfficiencyMetrics {
    pub report_id: i64,                             // Foreign key to WdrReport
    pub buffer_hit_percent: f64,                    // Buffer cache hit ratio
    pub cpu_efficiency_percent: f64,                // CPU utilization efficiency
    pub soft_parse_rate_percent: f64,               // Soft parse percentage
    pub hard_parse_rate_percent: f64,               // Hard parse percentage
    pub execution_efficiency_percent: f64,          // SQL execution efficiency
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoadProfile {
    pub report_id: i64,                             // Foreign key to WdrReport
    pub db_time_per_sec: f64,                       // DB time per second
    pub cpu_time_per_sec: f64,                      // CPU time per second
    pub io_requests_per_sec: f64,                   // IO requests per second
    pub total_transactions: u64,                    // Total transaction count
    pub commits_per_sec: f64,                       // Commit rate
    pub rollbacks_per_sec: f64,                     // Rollback rate
}
```

### 3. SQL Statistics

Top SQL queries and their performance metrics.

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopSql {
    pub id: i64,                                    // Unique identifier
    pub report_id: i64,                             // Foreign key to WdrReport
    pub sql_id: Option<String>,                     // SQL ID (if available)
    pub sql_text: String,                           // Full SQL query text
    pub executions: u64,                            // Number of executions
    pub total_elapsed_time: f64,                    // Total time (ms)
    pub cpu_time: f64,                              // CPU time (ms)
    pub io_time: f64,                               // IO time (ms)
    pub buffer_gets: u64,                           // Buffer gets
    pub disk_reads: u64,                            // Disk reads
    pub rows_processed: u64,                        // Rows processed
    pub first_load_time: DateTime<Utc>,             // First occurrence
    pub last_load_time: DateTime<Utc>,              // Last occurrence
    pub is_hot_sql: bool,                           // Flag for hot SQL detection
    pub rank_by_time: Option<i32>,                  // Ranking by execution time
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqlExecutionPlan {
    pub id: i64,                                    // Unique identifier
    pub sql_id: Option<i64>,                        // Foreign key to TopSql (optional for manual SQL)
    pub plan_tree: ExecutionPlanNode,               // Root node of plan tree
    pub created_at: DateTime<Utc>,                  // Plan generation timestamp
    pub source: PlanSource,                         // Where plan came from
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PlanSource {
    FromWdrReport(i64),    // Report ID
    UserProvided,          // Manually pasted SQL
    HotSql(i64),          // Hot SQL from report
}
```

### 4. Execution Plan Tree

Recursive tree structure representing SQL execution plans.

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionPlanNode {
    pub operation: String,                          // Operator type (Seq Scan, Hash Join, etc.)
    pub cost: f64,                                  // Estimated cost
    pub rows: u64,                                  // Estimated rows
    pub actual_rows: Option<u64>,                   // Actual rows (from ANALYZE)
    pub actual_time: Option<f64>,                   // Actual execution time (ms)
    pub width: Option<u32>,                         // Row width in bytes
    pub children: Vec<ExecutionPlanNode>,           // Child nodes
    pub node_details: PlanNodeDetails,              // Operator-specific details
    pub warnings: Vec<String>,                      // Optimization warnings
    pub suggestions: Vec<String>,                   // Optimization suggestions
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanNodeDetails {
    pub output: Option<Vec<String>>,                // Output columns
    pub filter: Option<String>,                     // Filter condition
    pub buffers: Option<String>,                    // Buffer usage
    pub join_type: Option<String>,                  // Join type (INNER, LEFT, etc.)
    pub hash_keys: Option<Vec<String>>,             // Hash join keys
    pub index_name: Option<String>,                 // Index used (if any)
    pub table_name: Option<String>,                 // Table name
}
```

### 5. Object Statistics

Table and index statistics from WDR reports.

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObjectStats {
    pub id: i64,                                    // Unique identifier
    pub report_id: i64,                             // Foreign key to WdrReport
    pub schema_name: String,                        // Schema name
    pub object_name: String,                        // Table/index name
    pub object_type: ObjectType,                    // TABLE, INDEX, etc.
    pub total_scans: u64,                           // Total scan count
    pub seq_scans: u64,                             // Sequential scan count
    pub idx_scans: u64,                             // Index scan count
    pub seq_reads: u64,                             // Sequential reads
    pub idx_reads: u64,                             // Index reads
    pub inserts: u64,                               // Insert count
    pub updates: u64,                               // Update count
    pub deletes: u64,                               // Delete count
    pub dead_tuples: u64,                           // Dead tuple count
    pub needs_vacuum: bool,                         // Flag for vacuum requirement
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ObjectType {
    Table,
    Index,
    View,
    Sequence,
}
```

### 6. Report Comparison

Comparison results between two WDR reports.

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WdrComparison {
    pub id: i64,                                    // Unique identifier
    pub source_report_id: i64,                      // First report ID
    pub target_report_id: i64,                      // Second report ID
    pub created_at: DateTime<Utc>,                  // Comparison timestamp
    pub comparison_type: ComparisonType,            // Type of comparison
    pub summary: ComparisonSummary,                 // Overall summary
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ComparisonType {
    TimeBased,      // Compare different time periods
    InstanceBased,  // Compare different instances
    AdHoc,         // Manual comparison
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComparisonSummary {
    pub performance_score_change: i32,              // Overall score difference (-100 to +100)
    pub status: ComparisonStatus,                   // Overall status
    pub conclusion: String,                         // Text summary
    pub key_findings: Vec<KeyFinding>,              // Critical changes identified
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ComparisonStatus {
    Improved,              // Performance improved
    Degraded,              // Performance degraded
    NoSignificantChange,   // No major change
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyFinding {
    pub category: FindingCategory,                  // SQL, Wait, Object, System
    pub metric: String,                             // Metric name
    pub change_percent: f64,                        // Percentage change
    pub severity: FindingSeverity,                  // Severity level
    pub description: String,                        // Human-readable description
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FindingCategory {
    Sql,
    Wait,
    Object,
    System,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FindingSeverity {
    Critical,   // >50% degradation
    Warning,    // 20-50% degradation
    Info,       // <20% change or improvement
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqlComparisonMetric {
    pub sql_id: Option<i64>,                        // Foreign key to TopSql
    pub sql_text_hash: String,                      // Hash of SQL text for matching
    pub source_metrics: SqlMetrics,                 // Metrics from source report
    pub target_metrics: SqlMetrics,                 // Metrics from target report
    pub change_percentages: SqlChangePercentages,   // Calculated changes
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqlMetrics {
    pub executions: u64,
    pub total_elapsed_time: f64,
    pub cpu_time: f64,
    pub io_time: f64,
    pub buffer_gets: u64,
    pub disk_reads: u64,
    pub rows_processed: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqlChangePercentages {
    pub executions: Option<f64>,
    pub elapsed_time: Option<f64>,
    pub cpu_time: Option<f64>,
    pub io_time: Option<f64>,
    pub buffer_gets: Option<f64>,
    pub disk_reads: Option<f64>,
    pub rows_processed: Option<f64>,
}
```

### 7. Threshold Configuration

Per-Constitution threshold settings with DTO format.

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThresholdConfig {
    pub id: i64,                                    // Unique identifier
    pub category: ThresholdCategory,                // SQL, Wait, System, AI
    pub data_type: ThresholdDataType,               // FLOAT, INTEGER, PERCENTAGE
    pub config_key: String,                         // Configuration key name
    pub value: f64,                                 // Threshold value
    pub default_value: f64,                         // Default value
    pub min_value: Option<f64>,                     // Minimum allowed value
    pub max_value: Option<f64>,                     // Maximum allowed value
    pub description: Option<String>,                // Human-readable description
    pub updated_at: DateTime<Utc>,                  // Last update timestamp
    pub updated_by: Option<String>,                 // User who updated (for audit)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ThresholdCategory {
    Sql,
    Wait,
    System,
    Ai,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ThresholdDataType {
    Float,
    Integer,
    Percentage,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThresholdUpdateRequest {
    pub value: f64,
    pub changed_by: String,
    pub change_reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThresholdTemplate {
    pub name: String,
    pub description: String,
    pub category: ThresholdCategory,
    pub thresholds: Vec<ThresholdConfig>,
}
```

### 8. SQL Audit Issues

Automatically detected SQL performance problems.

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqlAuditIssue {
    pub id: i64,                                    // Unique identifier
    pub report_id: Option<i64>,                     // Associated report (optional)
    pub sql_id: Option<i64>,                        // Associated SQL (optional)
    pub issue_type: AuditIssueType,                 // Type of issue
    pub severity: AuditSeverity,                    // Severity level
    pub title: String,                              // Issue title
    pub description: String,                        // Detailed description
    pub problematic_sql: Option<String>,            // SQL that caused issue
    pub recommendation: String,                     // Optimization suggestion
    pub status: AuditStatus,                        // Issue status
    pub detected_at: DateTime<Utc>,                 // Detection timestamp
    pub resolved_at: Option<DateTime<Utc>>,         // Resolution timestamp
    pub resolved_by: Option<String>,                // Who resolved it
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AuditIssueType {
    FullTableScan,          // Sequential scan on large table
    MissingIndex,           // Index could improve performance
    InefficientJoin,        // Poor join strategy
    MissingStats,           // Statistics not up to date
    ExpensiveFunction,      // Use of expensive functions
    CartesianProduct,       // Potential cartesian join
    NestedLoopWithIndex,    // Inefficient nested loop
    HashJoinTooLarge,       // Hash table too large
    SortOperation,          // Unnecessary sort
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AuditSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AuditStatus {
    Open,           // Not reviewed
    Reviewed,       // Reviewed but not fixed
    Whitelisted,    // Known issue, ignore
    Fixed,          // Issue resolved
    Ignored,        // Ignore in future scans
}
```

### 9. Audit Log

System operation audit trail per Constitution.

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditLog {
    pub id: i64,                                    // Unique identifier
    pub timestamp: DateTime<Utc>,                   // Operation timestamp
    pub user_id: Option<String>,                    // User performing action
    pub action: AuditAction,                        // Action performed
    pub entity_type: String,                        // Type of entity affected
    pub entity_id: Option<i64>,                     // ID of entity affected
    pub old_value: Option<String>,                  // Previous value (for updates)
    pub new_value: Option<String>,                  // New value (for updates)
    pub ip_address: Option<String>,                 // IP address (if applicable)
    pub success: bool,                              // Whether action succeeded
    pub error_message: Option<String>,              // Error if failed
    pub details: Option<String>,                    // Additional details
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AuditAction {
    Create,
    Read,
    Update,
    Delete,
    Import,
    Export,
    Login,
    Logout,
    ThresholdUpdate,
    ReportDelete,
    ConfigurationChange,
}
```

### 10. Dashboard Metrics

Aggregated metrics for dashboard display.

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanceSummary {
    pub instance_name: String,
    pub status: InstanceStatus,
    pub health_score: i32,                          // 0-100 health score
    pub active_issues: i32,                         // Number of open issues
    pub last_report_time: Option<DateTime<Utc>>,
    pub report_count: u64,                          // Total reports
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InstanceStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardMetrics {
    pub instance_name: Option<String>,              // If filtering by instance
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub tps: f64,                                   // Transactions per second
    pub qps: f64,                                   // Queries per second
    pub trend_data: Vec<TrendPoint>,                // Historical data points
    pub hot_issues: Vec<HotIssue>,                  // Top issues
    pub recent_reports: Vec<WdrReportSummary>,      // Recent report summaries
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrendPoint {
    pub timestamp: DateTime<Utc>,
    pub cpu: f64,
    pub memory: f64,
    pub tps: f64,
    pub qps: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HotIssue {
    pub title: String,
    pub count: u64,
    pub severity: AuditSeverity,
    pub category: FindingCategory,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WdrReportSummary {
    pub id: i64,
    pub instance_name: String,
    pub generation_time: DateTime<Utc>,
    pub snapshot_start: DateTime<Utc>,
    pub snapshot_end: DateTime<Utc>,
    pub status: ReportStatus,
}
```

## Database Schema

The SQLite database schema mirrors these models with the following tables:

```sql
-- Core tables
CREATE TABLE wdr_reports (
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

CREATE TABLE efficiency_metrics (
    report_id INTEGER NOT NULL,
    buffer_hit_percent REAL NOT NULL,
    cpu_efficiency_percent REAL NOT NULL,
    soft_parse_rate_percent REAL NOT NULL,
    hard_parse_rate_percent REAL NOT NULL,
    execution_efficiency_percent REAL NOT NULL,
    FOREIGN KEY (report_id) REFERENCES wdr_reports(id)
);

-- SQL statistics
CREATE TABLE top_sqls (
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

-- Execution plans
CREATE TABLE execution_plans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sql_id INTEGER,
    plan_tree TEXT NOT NULL,  -- JSON serialized
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    source TEXT NOT NULL,
    FOREIGN KEY (sql_id) REFERENCES top_sqls(id)
);

-- Comparisons
CREATE TABLE wdr_comparisons (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_report_id INTEGER NOT NULL,
    target_report_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    comparison_type TEXT NOT NULL,
    summary TEXT NOT NULL,  -- JSON serialized
    FOREIGN KEY (source_report_id) REFERENCES wdr_reports(id),
    FOREIGN KEY (target_report_id) REFERENCES wdr_reports(id)
);

-- Thresholds
CREATE TABLE threshold_configs (
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

-- Audit
CREATE TABLE sql_audit_issues (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    report_id INTEGER,
    sql_id INTEGER,
    issue_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    problematic_sql TEXT,
    recommendation TEXT NOT NULL,
    status TEXT NOT NULL,
    detected_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    resolved_at DATETIME,
    resolved_by TEXT,
    FOREIGN KEY (report_id) REFERENCES wdr_reports(id),
    FOREIGN KEY (sql_id) REFERENCES top_sqls(id)
);

CREATE TABLE audit_logs (
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

-- Indexes for performance
CREATE INDEX idx_top_sqls_report_id ON top_sqls(report_id);
CREATE INDEX idx_top_sqls_hot_sql ON top_sqls(is_hot_sql);
CREATE INDEX idx_wdr_reports_instance ON wdr_reports(instance_name);
CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp);
CREATE INDEX idx_threshold_configs_category ON threshold_configs(category);
```

## Validation Rules

All entities implement validation:

1. **Required fields**: All mandatory fields must be present
2. **Value ranges**: Numeric values within min/max bounds (if defined)
3. **Referential integrity**: Foreign key relationships enforced
4. **Data types**: Strict type checking on serialization/deserialization
5. **Business rules**:
   - Threshold values must be non-negative
   - Report comparison requires two different reports
   - Audit log entries required for all threshold updates
   - Hot SQL detection based on execution time thresholds

## Serialization

All models use Serde for JSON serialization:

```rust
// Example: Serialization for IPC
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct WdrReportList(pub Vec<WdrReport>);

// Frontend receives JSON:
// {"0":{"id":1,"instance_name":"prod-db-01",...}}

// Deserialization from IPC:
let reports: Vec<WdrReport> = serde_json::from_str(&payload)?;
```

## State Transitions

### Report Status
```
SuccessfullyImported → ImportFailed (if corrupted)
PartiallyImported → SuccessfullyImported (on retry)
```

### Audit Issue Status
```
Open → Reviewed → Fixed/Whitelisted/Ignored
Open → Whitelisted (immediate)
Open → Fixed (after resolution)
```

### Comparison Status
```
NoSignificantChange → Improved/Degraded (based on metrics)
```

## Notes

- All timestamps use UTC
- All monetary values stored as REAL (f64)
- Boolean values stored as INTEGER (0/1) in SQLite
- JSON fields stored as TEXT in SQLite
- Enum values stored as TEXT for compatibility
- Large text fields (SQL, recommendations) can be up to several MB
- Database supports transactions for batch operations
