# WDRProbe Developer Guide

> For developers integrating with WDRProbe, building MCP tools, extending the API, or contributing to the codebase.

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Backend Module Hierarchy](#2-backend-module-hierarchy)
3. [Database Schema Reference](#3-database-schema-reference)
4. [IPC Command API Reference](#4-ipc-command-api-reference)
5. [Parser Internals](#5-parser-internals)
6. [Error Handling](#6-error-handling)
7. [Frontend Architecture](#7-frontend-architecture)
8. [Internationalization (i18n)](#8-internationalization-i18n)
9. [Build System](#9-build-system)
10. [Testing Infrastructure](#10-testing-infrastructure)
11. [Extension Points & MCP Integration](#11-extension-points--mcp-integration)

---

## 1. Architecture Overview

WDRProbe follows a **Tauri v1** architecture with a clear separation between the Rust backend and React frontend.

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri Window                          │
│  ┌───────────────────────────────────────────────────┐  │
│  │              React Frontend (WebView)              │  │
│  │  ┌──────────┐ ┌──────────┐ ┌───────────────────┐ │  │
│  │  │  Pages   │ │ Contexts │ │  apiService.ts    │ │  │
│  │  │ (11 pg)  │ │ (I18n,   │ │ (IPC wrappers +   │ │  │
│  │  │          │ │  Plan,   │ │  mock fallbacks)  │ │  │
│  │  │          │ │  WDR)    │ │                   │ │  │
│  │  └──────────┘ └──────────┘ └────────┬──────────┘ │  │
│  └─────────────────────────────────────┼─────────────┘  │
│                                        │ invoke()       │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┼ ─ ─ ─ ─ ─ ─ ─  │
│  ┌─────────────────────────────────────┼─────────────┐  │
│  │           Rust Backend              │             │  │
│  │           ┌───────────┐             │             │  │
│  │           │ Commands  │ ◄───────────┘             │  │
│  │           │ (38 cmds) │                           │  │
│  │           └─────┬─────┘                           │  │
│  │     ┌───────────┼───────────┐                     │  │
│  │  ┌──┴──┐  ┌─────┴────┐  ┌──┴──────┐              │  │
│  │  │ DB  │  │ Parsers  │  │ Models  │              │  │
│  │  │ Pool│  │ (WDR,SQL)│  │ (Serde) │              │  │
│  │  └──┬──┘  └──────────┘  └─────────┘              │  │
│  │     │                                              │  │
│  │  ┌──┴──────────────┐                              │  │
│  │  │ SQLite (WAL)    │                              │  │
│  │  │ wdrprobe.db     │                              │  │
│  │  └─────────────────┘                              │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Data Flow

1. User interacts with React frontend
2. Frontend calls `invoke("command_name", { args })` via Tauri IPC
3. Rust `#[tauri::command]` handler receives the call with `State<DatabasePool>` injection
4. Handler calls `DatabaseOperations` trait methods or parsers
5. Result serialized via Serde and returned as JSON to frontend
6. Frontend updates React state and re-renders

### Key Architecture Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Desktop framework | Tauri v1 (not v2) | Stable, well-tested, smaller binary size |
| Database | SQLite (bundled) | No external dependency, WAL mode for concurrency |
| Connection pool | r2d2 + r2d2_sqlite | Thread-safe pooling for Tauri's multi-threaded IPC |
| HTML parsing | scraper | CSS selector-based, good for structured HTML reports |
| SQL parsing | nom | Combinator parser, good for execution plan text format |
| Large file handling | memmap2 | Memory-mapped file I/O for large WDR reports |
| State management | React Context | Lightweight, no external dependency |
| i18n | Custom Context | Full control over translation keys, no library overhead |

---

## 2. Backend Module Hierarchy

All Rust backend code is in `Desktop/src-tauri/src/`:

```
src/
├── main.rs                         # Entry point: Tauri Builder, DB init, command registration
├── lib.rs                          # Library exports (for testing)
│
├── commands/                       # Tauri IPC command handlers
│   ├── mod.rs                      # Module declarations
│   ├── dashboard.rs                # Instance summaries, dashboard metrics
│   ├── reports.rs                  # WDR import, list, detail, delete, hot SQLs
│   ├── execution_plan.rs           # Plan parsing, analysis, save, optimization
│   ├── comparison.rs               # Report comparison, summary, chart data
│   ├── threshold.rs                # Threshold CRUD, templates, history, validation
│   ├── audit.rs                    # SQL audit detection, issue management
│   └── export.rs                   # Export/import, data integrity validation
│
├── database/                       # SQLite database layer
│   ├── mod.rs                      # DatabasePool type, init_database()
│   ├── schema.rs                   # DDL, indexes, PRAGMA settings, default data
│   └── operations.rs               # DatabaseOperations trait + impl on DatabasePool
│
├── parsers/                        # Report and plan parsers
│   ├── mod.rs
│   ├── wdr_parser.rs               # Basic HTML WDR parser (metadata + SQLs)
│   ├── complete_wdr_parser.rs      # Full WDR parser (all sections)
│   └── sql_parser.rs               # Execution plan parser (JSON + text formats)
│
├── models/                         # Domain data models
│   ├── mod.rs
│   ├── report.rs                   # WdrReport, TopSql, EfficiencyMetrics, etc.
│   ├── execution_plan.rs           # ExecutionPlanNode, PlanMetadata, etc.
│   ├── dashboard.rs                # InstanceSummary, DashboardMetrics
│   ├── comparison.rs               # WdrComparison, ComparisonSummary, chart types
│   ├── threshold.rs                # ThresholdConfig, templates, validation
│   ├── audit.rs                    # SqlAuditIssue, AuditLog, enums
│   └── export.rs                   # Export/import models
│
├── utils/                          # Cross-cutting utilities
│   ├── mod.rs
│   ├── error.rs                    # WdrProbeError enum (11 variants)
│   ├── audit.rs                    # AuditLogger for audit trail
│   └── gaussdb.rs                  # GaussDB helpers (EXPLAIN JSON, HypoIndex)
│
└── progress/                       # Progress reporting for long operations
    └── mod.rs                      # ProgressReporter, ProgressUpdate, ProgressState
```

### Module Dependencies

```
main.rs
  ├── commands/* (all 7 modules)
  │     ├── database/* (DatabasePool, DatabaseOperations)
  │     ├── models/* (domain structs)
  │     ├── parsers/* (WDR, SQL parsers)
  │     ├── utils/* (error, audit, gaussdb)
  │     └── progress/* (ProgressReporter)
  └── database/* (init, schema)
```

---

## 3. Database Schema Reference

Database: SQLite with WAL mode, 64MB cache, 256MB mmap.

Location: `{app_data_dir}/wdrprobe.db`

### Tables Overview

| # | Table | Purpose | Key Indexes |
|---|-------|---------|-------------|
| 1 | `wdr_reports` | WDR report metadata | instance_name, generation_time, status |
| 2 | `efficiency_metrics` | Instance efficiency percentages | report_id (FK) |
| 3 | `load_profile` | Time model and load profile | report_id (FK) |
| 4 | `database_stats` | Per-database statistics | report_id (FK) |
| 5 | `top_sqls` | SQL statements and performance metrics | report_id, is_hot_sql, rank_by_time |
| 6 | `execution_plans` | Saved execution plans (JSON) | sql_id, source |
| 7 | `cache_io_stats` | Cache I/O for tables/indexes | report_id, object_type |
| 8 | `object_stats` | Table/index usage statistics | report_id, object_type |
| 9 | `wdr_comparisons` | Report comparison records | source_report_id, target_report_id |
| 10 | `sql_comparison_metrics` | SQL-level comparison metrics | comparison_id, sql_text_hash |
| 11 | `threshold_configs` | Threshold configurations | category, config_key |
| 12 | `sql_audit_issues` | Detected SQL audit issues | report_id, status, severity, issue_type |
| 13 | `audit_logs` | Operation audit trail | timestamp, action, entity_type+entity_id |

### SQLite Performance Settings

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000;      -- 64MB cache
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 268435456;    -- 256MB mmap
PRAGMA foreign_keys = ON;
```

### Default Data Seeding

On first launch, the schema initializer seeds:

**14 default threshold configurations**:
- SQL (5): `sql_top_time`, `sql_scan_rows`, `sql_cpu_time`, `sql_io_time`, `sql_buffer_gets`
- WAIT (3): `wait_max_lock`, `wait_max_io`, `wait_max_lwlock`
- SYSTEM (4): `sys_cpu_usage`, `sys_memory_usage`, `sys_disk_io`, `sys_buffer_hit`
- AI (2): `ai_sample_size`, `ai_confidence`

**5 sample SQL audit issues**: FullTableScan, MissingIndex, InefficientJoin, ExpensiveFunction, MissingStats

### DatabaseOperations Trait

The `DatabaseOperations` trait in `database/operations.rs` defines all CRUD operations. It is implemented for `DatabasePool` (which is `r2d2::Pool<SqliteConnectionManager>`).

Key method groups:
- **Report ops**: `create_wdr_report`, `get_wdr_report`, `list_wdr_reports`, `delete_wdr_report`
- **SQL ops**: `create_top_sql`, `get_top_sqls_by_report`, `get_hot_sqls`
- **Metrics ops**: `create_efficiency_metrics`, `create_load_profile`, `create_database_stats`, `create_cache_io_stats`, `create_object_stats` (+ corresponding getters)
- **Plan ops**: `create_execution_plan`, `get_execution_plan_by_sql`, `delete_execution_plan`
- **Comparison ops**: `create_comparison`, `get_comparison_summary`, `get_comparison_details`, `get_comparison_chart_data`
- **Threshold ops**: `create_threshold_config`, `update_threshold_config`, `get_threshold_configs`, `get_threshold_config`
- **Audit ops**: `create_audit_log`, `get_audit_logs`
- **Dashboard ops**: `get_instance_summaries`, `get_dashboard_metrics`

---

## 4. IPC Command API Reference

All 38 Tauri IPC commands. Commands use `#[tauri::command(rename_all = "camelCase")]` for JS-friendly naming.

### Calling Convention

```typescript
// Frontend (TypeScript)
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('commandName', { arg1: value1, arg2: value2 });
```

```rust
// Backend (Rust)
#[tauri::command]
pub async fn command_name(
    pool: State<'_, DatabasePool>,
    arg1: String,
    arg2: Option<i32>,
) -> Result<ResponseType, String> { ... }
```

> All commands return `Result<T, String>` where the error is a descriptive string.

---

### 4.1 Dashboard Commands (2)

#### `get_instance_summaries`

Returns instance health summaries for the dashboard.

```typescript
// No parameters
const summaries: InstanceSummary[] = await invoke('get_instance_summaries');
```

**Response**: `InstanceSummary[]` — each contains `instanceName`, `status` (Healthy/Warning/Critical), `healthScore`, `activeIssues`, `reportCount`, `lastReportTime`.

#### `get_dashboard_metrics`

Returns aggregated dashboard metrics, optionally filtered by instance.

```typescript
const metrics: DashboardMetrics = await invoke('get_dashboard_metrics', {
  instanceName: 'prod-db-01' // Optional<string>
});
```

**Response**: `DashboardMetrics` — `cpu`, `mem`, `tps`, `qps` (formatted strings), `healthDistribution`, `trendData`, `hotIssues`.

---

### 4.2 Report Commands (5)

#### `import_wdr_report`

Parses and imports a WDR HTML file, storing all sections to the database.

```typescript
const report: WdrReport = await invoke('import_wdr_report', {
  filePath: '/path/to/report.html',
  instanceName: 'prod-db-01',
  description: 'Optional description' // Optional
});
```

**Parses**: Report metadata, efficiency metrics, load profile, database stats, top SQLs (max 200), cache I/O stats, object stats.

#### `get_wdr_reports`

Returns a paginated list of stored WDR reports.

```typescript
const response: WdrReportListResponse = await invoke('get_wdr_reports', {
  limit: 50,   // Optional<i32>
  offset: 0    // Optional<i32>
});
```

#### `get_wdr_report_detail`

Returns full report details with all associated data.

```typescript
const detail: WdrReportDetail = await invoke('get_wdr_report_detail', {
  reportId: 1 // i64
});
```

**Includes**: Efficiency metrics, load profile, top SQLs, object stats, cache I/O stats, database stats.

#### `delete_wdr_report`

Deletes a report and cascade-deletes all associated data.

```typescript
await invoke('delete_wdr_report', { reportId: 1 });
```

#### `get_hot_sqls`

Returns the hottest SQL queries across all reports.

```typescript
const sqls: TopSql[] = await invoke('get_hot_sqls', { limit: 10 });
```

---

### 4.3 Execution Plan Commands (8)

#### `get_wdr_hot_sqls`

Retrieves hot SQL queries from WDR reports with sorting.

```typescript
const result: WdrHotSqlList = await invoke('get_wdr_hot_sqls', {
  reportId: 1,          // Optional<i64>
  limit: 20,            // Optional<i32>
  sortBy: 'cpu_time'    // Optional<String>: 'cpu_time' | 'executions' | 'elapsed_time'
});
```

#### `get_execution_plan`

Gets execution plan for a SQL from WDR report or user-provided text.

```typescript
const result: ExecutionPlanResponse = await invoke('get_execution_plan', {
  sqlId: 123,             // Optional<i64>
  sqlText: 'SELECT ...',  // Optional<String>
  planSource: 'wdr',      // String: 'wdr' | 'manual' | 'saved'
  reportId: 1             // Optional<i64>
});
```

**Returns**: Plan tree (`ExecutionPlanNode`), metadata (total cost, depth, node count), warnings, and suggestions.

#### `parse_execution_plan`

Parses execution plan text into a tree structure.

```typescript
const result: ParsedPlan = await invoke('parse_execution_plan', {
  planText: 'Seq Scan on public.t1  (cost=0.00..34.00 rows=2400 width=4)',
  format: 'text',    // String: 'json' | 'text' | 'sql-plan'
  source: 'manual'   // String
});
```

#### `analyze_execution_plan_command`

Analyzes a parsed plan for performance issues.

```typescript
const result: PlanAnalysis = await invoke('analyze_execution_plan_command', {
  plan: executionPlanNode,            // ExecutionPlanNode
  thresholds: { maxCost: 10000 }     // Optional<ThresholdOverrides>
});
```

**Returns**: Issues list (13 rule types), performance score (0-100), recommendations.

#### `save_execution_plan`

Saves an execution plan to the database.

```typescript
const result: SavePlanResult = await invoke('save_execution_plan', {
  sqlId: 123,                        // Optional<i64>
  sqlText: 'SELECT ...',             // Optional<String>
  planTree: executionPlanNode,       // ExecutionPlanNode
  planSource: 'manual',              // String
  reportId: 1,                       // Optional<i64>
  name: 'Before optimization'        // Optional<String>
});
```

#### `get_saved_plans`

Retrieves saved plans with filtering and pagination.

```typescript
const result: SavedPlansResponse = await invoke('get_saved_plans', {
  sqlId: 123,     // Optional<i64>
  reportId: 1,    // Optional<i64>
  limit: 20,      // Optional<i32>
  offset: 0       // Optional<i32>
});
```

#### `delete_execution_plan`

Deletes a saved plan (requires confirmation).

```typescript
const result: DeleteResult = await invoke('delete_execution_plan', {
  planId: 5,     // i64
  confirm: true  // bool
});
```

#### `generate_optimization_sql`

Generates optimization SQL based on plan analysis.

```typescript
const result: OptimizationSql = await invoke('generate_optimization_sql', {
  planId: 5,                 // i64
  optimizationType: 'index'  // String: 'index' | 'statistics' | 'rewrite'
});
```

**Returns**: SQL statement, explanation, and confidence level.

---

### 4.4 Comparison Commands (6)

#### `get_comparisons`

Lists comparisons with sorting and pagination.

```typescript
const result: ComparisonListResponse = await invoke('get_comparisons', {
  limit: 20,       // Optional<i32>
  offset: 0,       // Optional<i32>
  sortBy: 'created',  // Optional<String>
  sortOrder: 'desc'   // Optional<String>
});
```

#### `get_comparison_summary`

Returns comparison summary with verdict and key findings.

```typescript
const summary: ComparisonSummary = await invoke('get_comparison_summary', {
  comparisonId: 1 // i64
});
```

**Returns**: `status` (Improved/Degraded/Stable), `scoreChange`, `conclusion`, `keyFindings[]`.

#### `get_comparison_details`

Returns detailed comparison metrics by category.

```typescript
const details: ComparisonDetails = await invoke('get_comparison_details', {
  comparisonId: 1,        // i64
  category: 'sql',        // String: 'sql' | 'database' | 'efficiency' | ...
  limit: 50,              // Optional<i32>
  offset: 0               // Optional<i32>
});
```

#### `create_comparison`

Creates a comparison between two WDR reports.

```typescript
const result: CreateComparisonResult = await invoke('create_comparison', {
  sourceReportId: 1,      // i64
  targetReportId: 2,      // i64
  comparisonType: 'auto', // Optional<String>
  customName: 'Weekly'    // Optional<String>
});
```

**Computes**: Performance scores, SQL metric changes, key findings automatically.

#### `delete_comparison`

Deletes a comparison (requires confirmation).

```typescript
await invoke('delete_comparison', { comparisonId: 1, confirm: true });
```

#### `get_comparison_chart_data`

Returns chart-ready data for visualization.

```typescript
const data: ChartData = await invoke('get_comparison_chart_data', {
  comparisonId: 1,        // i64
  chartType: 'sql_time'   // String
});
```

---

### 4.5 Threshold Commands (9)

#### `get_threshold_configs`

Returns threshold configurations, optionally filtered by category.

```typescript
const result: ThresholdConfigList = await invoke('get_threshold_configs', {
  category: 'SQL'  // Optional<String>: 'SQL' | 'WAIT' | 'SYSTEM' | 'AI'
});
```

#### `get_threshold_config`

Returns a specific threshold by key.

```typescript
const config: ThresholdConfig = await invoke('get_threshold_config', {
  configKey: 'sql_top_time'
});
```

#### `update_threshold`

Updates a single threshold value with audit logging.

```typescript
const result: UpdateThresholdResult = await invoke('update_threshold', {
  configKey: 'sql_top_time',
  value: 2000.0,
  changedBy: 'admin',
  changeReason: 'Increased due to performance issues'
});
```

#### `batch_update_thresholds`

Updates multiple thresholds in a single transaction.

```typescript
const result: BatchUpdateResult = await invoke('batch_update_thresholds', {
  updates: [
    { configKey: 'sql_top_time', value: 2000.0 },
    { configKey: 'sql_cpu_time', value: 800.0 }
  ],
  changedBy: 'admin',
  changeReason: 'Batch adjustment'
});
```

#### `reset_threshold_to_default`

Resets a threshold to its default value.

```typescript
await invoke('reset_threshold_to_default', {
  configKey: 'sql_top_time',
  changedBy: 'admin',
  changeReason: 'Reset to default'
});
```

#### `get_threshold_templates`

Returns available preset templates.

```typescript
const templates: ThresholdTemplateList = await invoke('get_threshold_templates');
```

**Templates**: High Concurrency, Low Resource, Development, Production, GaussDB Optimized.

#### `apply_threshold_template`

Applies a template to set multiple thresholds at once.

```typescript
const result: ApplyTemplateResult = await invoke('apply_threshold_template', {
  templateName: 'High Concurrency',
  changedBy: 'admin',
  changeReason: 'Switched to high concurrency profile'
});
```

#### `get_threshold_history`

Returns change history for a specific threshold.

```typescript
const history: ThresholdHistory = await invoke('get_threshold_history', {
  configKey: 'sql_top_time',
  limit: 20  // Optional<i32>
});
```

#### `validate_threshold_value`

Validates a threshold value without saving.

```typescript
const result: ValidationResult = await invoke('validate_threshold_value', {
  configKey: 'sql_top_time',
  value: 2000.0
});
```

---

### 4.6 Audit Commands (4)

#### `run_sql_audit`

Runs SQL audit detection rules on specified reports.

```typescript
const result: AuditRunResult = await invoke('run_sql_audit', {
  reportIds: [1, 2],          // Optional<Vec<i64>>
  includeResolved: false,     // bool
  auditTypes: ['FullTableScan'] // Optional<Vec<String>>
});
```

**Detection types**: FullTableScan, MissingIndex, InefficientJoin, MissingStats, ExpensiveFunction, CartesianProduct, NestedLoopWithIndex, HashJoinTooLarge, SortOperation.

#### `get_sql_audit_issues`

Returns audit issues with filtering and pagination.

```typescript
const result: SqlAuditIssueList = await invoke('get_sql_audit_issues', {
  reportId: 1,         // Optional<i64>
  status: 'Open',      // Optional<String>
  severity: 'High',    // Optional<String>
  issueType: null,     // Optional<String>
  limit: 50,           // Optional<i32>
  offset: 0            // Optional<i32>
});
```

#### `update_audit_issue_status`

Updates a single audit issue status.

```typescript
const result = await invoke('update_audit_issue_status', {
  issueId: 5,
  status: 'Fixed',
  resolvedBy: 'admin',
  resolutionNote: 'Added index on create_time'
});
```

#### `bulk_update_audit_issues`

Bulk updates multiple audit issues.

```typescript
const result = await invoke('bulk_update_audit_issues', {
  issueIds: [1, 2, 3],
  status: 'Whitelisted',
  resolvedBy: 'admin',
  resolutionNote: 'Known acceptable pattern'
});
```

---

### 4.7 Export Commands (4)

#### `export_wdr_report`

Exports a WDR report to file.

```typescript
const result: ExportResult = await invoke('export_wdr_report', {
  reportId: 1,
  format: 'JSON',               // ExportFormat: 'JSON' | 'CSV' | 'PDF'
  includeSqlDetails: true,      // bool
  includeComparisonData: false, // bool
  exportPath: null              // Optional<String>
});
```

#### `export_comparison`

Exports comparison data to file.

```typescript
const result: ExportResult = await invoke('export_comparison', {
  comparisonId: 1,
  format: 'JSON',
  exportPath: null
});
```

#### `import_data`

Imports data from file with validation.

```typescript
const result: ImportResult = await invoke('import_data', {
  importPath: '/path/to/export.json',
  validateOnly: false,    // bool — if true, validates without importing
  overwriteExisting: false, // bool
  importTypes: ['Reports', 'Thresholds'] // Vec<String>
});
```

**Import types**: `Reports`, `Comparisons`, `Thresholds`, `AuditIssues`.

#### `validate_data_integrity`

Validates data integrity using checksum, record count, or schema validation.

```typescript
const result: DataIntegrityCheck = await invoke('validate_data_integrity', {
  checkType: 'checksum',  // String: 'checksum' | 'record_count' | 'schema'
  entityType: 'report',   // String
  entityId: 1,            // Optional<i64>
  expectedHash: null      // Optional<String>
});
```

---

## 5. Parser Internals

### 5.1 WDR HTML Parser

Located in `parsers/wdr_parser.rs` and `parsers/complete_wdr_parser.rs`.

**Library**: `scraper` (CSS selector-based HTML parser)

#### Parse Flow

```
Input: HTML file path
    │
    ├── Read file (BufReader for large files)
    │
    ├── Parse HTML → scraper::Html::parse_document()
    │
    ├── Extract metadata:
    │   ├── Instance name (multiple CSS selectors: .instance-name, h1, etc.)
    │   ├── Generation time (.generation-time, .timestamp)
    │   └── Snapshot period (.snapshot-start, .snapshot-end)
    │
    ├── Complete parser also extracts:
    │   ├── Efficiency metrics ("Instance Efficiency Percentages" table)
    │   ├── Load profile ("Time model" / "Load Profile" table)
    │   ├── Database stats ("Database Stat" table)
    │   ├── Top SQLs (tables with "Unique SQL Id" header, max 200)
    │   ├── Cache I/O stats ("User table IO activity" tables)
    │   └── Object stats ("User Tables/Index stats" tables)
    │
    └── Output: CompleteWdrReport struct
```

#### Format Detection

The parser handles two HTML formats:

| Format | Detection | Key Differences |
|--------|-----------|-----------------|
| OpenGauss v1 | CSS class patterns, table structure | Older table layout |
| OpenGauss v2 | Different CSS classes, nested structure | Newer semantic HTML |

The parser tries multiple selectors for each field, making it resilient to format variations.

#### Top SQL Extraction

```
For each table with "Unique SQL Id" header:
  ├── Extract column headers (sql_id, calls, total_time, cpu_time, ...)
  ├── Parse each row into TopSql struct
  ├── Calculate derived metrics (avg_time = total_time / calls)
  └── Assign rank_by_time (descending order)
```

### 5.2 SQL Execution Plan Parser

Located in `parsers/sql_parser.rs`.

**Library**: `nom` (combinator parser) + `serde_json` (JSON format)

#### Supported Formats

| Format | Detection | Parser |
|--------|-----------|--------|
| GaussDB JSON (`EXPLAIN FORMAT JSON`) | Starts with `[` or `{` | `parse_execution_plan_json()` |
| Text indented (`EXPLAIN`) | Contains `->` with indentation | `parse_execution_plan_text()` |
| Tabular (`EXPLAIN PERFORMANCE`) | Contains `\|` column separators | `parse_sql_plan_format()` |
| SQL + Plan combined | Contains SQL keyword followed by plan | Split and parse both parts |

#### Text Parser Algorithm

```
parse_execution_plan_text(plan_text)
    │
    ├── Detect format: is_sql_plan_format()
    │
    ├── Parse root node from first non-empty line:
    │   └── parse_plan_line(line) → (indent_level, operation, cost, rows, details)
    │
    └── Recursively parse children:
        parse_children(lines, parent_indent):
            while next line has greater indent:
                parse child node
                recurse into grandchildren
            return children[]
```

#### Analysis Engine

After parsing, the analyzer checks 13 rules:

| Rule ID | Condition | Suggestion |
|---------|-----------|------------|
| 001 | Total cost > threshold | Check missing indexes, complex joins |
| 002 | Seq Scan on table with >10K rows | Consider adding index |
| 003 | SubPlan detected | Rewrite as JOIN |
| 004 | Nested Loop without index (Cartesian) | Ensure join conditions correct |
| 005 | Many partitions scanned | Check partition pruning |
| 006 | Execution time > threshold | Identify bottleneck nodes |
| 007 | Bitmap Scan used | Check if Index Scan is better |
| 008 | Disk spill detected | Increase work_mem |
| 009 | Index Scan with filter | Include filter columns in index |
| 011 | User function in plan | Check function performance |
| 012 | UPDATE with many subqueries | Simplify update logic |
| 013 | ROWNUM on large set | Use LIMIT/OFFSET |

---

## 6. Error Handling

### WdrProbeError Enum

Defined in `utils/error.rs`:

```rust
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
```

### Error Propagation in Commands

Tauri commands return `Result<T, String>`. Errors are converted via `map_err(|e| e.to_string())`:

```rust
#[tauri::command]
pub async fn my_command(pool: State<'_, DatabasePool>) -> Result<MyType, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    let result = conn.get_data().map_err(|e| e.to_string())?;
    Ok(result)
}
```

### Frontend Error Handling

The `ErrorBoundary` component in `frontend/components/ErrorBoundary.tsx` catches unhandled React errors. API-level errors should be caught per-call in the page component:

```typescript
try {
  const data = await ApiService.getWdrReportDetail(id);
  setData(data);
} catch (err) {
  setError(err instanceof Error ? err.message : 'Unknown error');
}
```

---

## 7. Frontend Architecture

### Routing

Uses `react-router-dom` v7 with `HashRouter` (required for Tauri's custom protocol):

```typescript
// App.tsx
<HashRouter>
  <Layout>
    <Routes>
      <Route path="/" element={<Dashboard />} />
      <Route path="/wdr-analysis" element={<WDRReportAnalyze />} />
      <Route path="/wdr-comparison" element={<WDRComparison />} />
      <Route path="/reports" element={<ReportManagement />} />
      <Route path="/reports/:id" element={<ReportDetail />} />
      <Route path="/comparison" element={<ComparisonAnalysis />} />
      <Route path="/visualizer" element={<PlanVisualizer />} />
      <Route path="/plandiff" element={<PlanDiff />} />
      <Route path="/thresholds" element={<ThresholdConfig />} />
      <Route path="/sqlaudit" element={<SqlAudit />} />
      <Route path="/auditlog" element={<AuditLog />} />
    </Routes>
  </Layout>
</HashRouter>
```

### Context Providers

Three React Context providers wrap the app:

| Provider | File | Purpose |
|----------|------|---------|
| `I18nProvider` | `context/I18nContext.tsx` | Language state + `t()` translation function |
| `PlanProvider` | `context/PlanContext.tsx` | Execution plan state (parsed plan, history) |
| `WDRProvider` | `context/WDRContext.tsx` | WDR report state (uploaded reports, comparison data) |

### API Service Layer

`frontend/services/apiService.ts` wraps all Tauri IPC calls. It includes a mock fallback pattern:

```typescript
const isTauri = () => !!(window as any).__TAURI__;

export const ApiService = {
  getWdrReports: async (): Promise<WdrReport[]> => {
    if (isTauri()) return invoke('get_wdr_reports');
    return Promise.resolve(MOCK_REPORTS); // Dev fallback
  },
  // ...
};
```

This allows running the frontend with `npm run dev` (Vite-only) for UI development without the Rust backend.

### Key TypeScript Types

All types defined in `frontend/types.ts`. Key interfaces:

- `WdrReportDetail` — Full report with efficiency, load profile, host CPU, IO, memory, wait events, top SQL, object stats, configs
- `WdrTopSqlItem` — Expanded Top SQL with timing, rows, IO, sort, and hash stats
- `ComparisonSummary` — Verdict (Improved/Degraded/Stable), score change, key findings
- `EnhancedNode` — Extended execution plan node with cost breakdown, CTE info, actual stats
- `DiffNode` — Plan diff node with matched-pair information
- `PlanIssue` — Analysis rule detection result (ruleId, severity, suggestion)

---

## 8. Internationalization (i18n)

### Architecture

Custom React Context-based i18n in `frontend/context/I18nContext.tsx`.

```typescript
type Language = 'en' | 'zh';

const translations: Record<Language, Record<string, string>> = {
  en: { 'menu.wdrAnalyze': 'WDR Analysis', ... },
  zh: { 'menu.wdrAnalyze': 'WDR 分析', ... },
};

// Usage in components:
const { t, language, setLanguage } = useI18n();
<p>{t('menu.wdrAnalyze')}</p>
```

### Adding a Translation Key

1. Add the key to **both** `en` and `zh` objects in `I18nContext.tsx`
2. Use `t('your.key')` in any component
3. Support parameters: `t('wdr.issue.deadTupDesc', { table: 'orders', count: 5000 })`

### Current Translation Coverage

The app has comprehensive translations for:
- Menu items (11)
- Dashboard (16 keys)
- Reports (26 keys)
- Comparison (40+ keys)
- WDR Comparison specific (30+ keys)
- Thresholds (10 keys)
- SQL Audit (18 keys)
- Audit Log (8 keys)
- WDR Analyze + Knowledge Base (30+ keys)
- Visualizer + Knowledge Base (40+ keys)
- Visualizer Rules (13 rules × 3 fields each)
- Plan Diff (25+ keys)

---

## 9. Build System

### NPM Scripts (`Desktop/package.json`)

| Script | Command | Purpose |
|--------|---------|---------|
| `dev` | `vite` | Frontend-only dev server (port 1420) |
| `build` | `tsc && vite build` | TypeScript check + production build |
| `preview` | `vite preview` | Preview production build |
| `tauri` | `tauri` | Tauri CLI passthrough |
| `tauri:dev` | `tauri dev` | Full dev mode (Vite + Tauri) |
| `tauri:build` | `tauri build` | Production build |

### Vite Configuration (`Desktop/vite.config.ts`)

- **Port**: 1420 (strict — Tauri requires this exact port)
- **HMR**: Port 1421 for WebSocket when `TAURI_DEV_HOST` is set
- **Sourcemaps**: Enabled
- **Minification**: esbuild (unless `TAURI_DEBUG` set)
- **Manual chunks**: `vendor-react` (react + react-dom)
- **Path aliases**: `@`, `@components`, `@utils`, `@types`

### Tauri Configuration (`Desktop/src-tauri/tauri.conf.json`)

- **Identifier**: `com.wdrprobe.desktop`
- **Window**: 1200×800, resizable, centered
- **CSP**: Disabled (`null`) — intentional for dev
- **Allowlist**: FS operations, dialog (open/save), path, shell (open only)

### Rust Dependencies (`Desktop/src-tauri/Cargo.toml`)

| Category | Crates |
|----------|--------|
| Tauri | tauri 1.8 (fs, dialog, shell, path features) |
| Database | rusqlite 0.30 (bundled), r2d2 0.8, r2d2_sqlite 0.23 |
| Parsing | scraper 0.17, regex 1.10, nom 7.1, memmap2 0.9 |
| Serialization | serde 1.0, serde_json 1.0 |
| Async | tokio 1.34 (full), tokio-util 0.7 |
| Utilities | thiserror 1.0, anyhow 1.0, chrono 0.4, uuid 1.6 |
| Test deps | mockall 0.12, tempfile 3.8, rstest 0.18, criterion 0.5 |

### Test Feature Flag

```toml
[features]
test = ["mockall", "tempfile", "rstest", "criterion"]
```

Run tests with: `cargo test --features test`

---

## 10. Testing Infrastructure

### Test File Organization

See [CONTRIBUTION.md > Testing](CONTRIBUTION.md#testing) for the full test file listing and patterns.

### Test Statistics

- **21 test files** (14 domain-level + 5 integration + 1 nested + 1 algorithm)
- **~150 test cases** covering all command domains
- **Database isolation**: Each test uses `tempfile::TempDir` for a fresh SQLite DB
- **Async tests**: Use `#[tokio::test]` for async command handlers
- **Test fixtures**: Real WDR HTML files from `example/` directory

### Key Test Patterns

```rust
// Database setup helper (pattern used across test files)
fn setup_test_db() -> (TempDir, DatabasePool) {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let db_path = tmp.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).unwrap();
    let conn = pool.get().unwrap();
    initialize_schema(&conn).unwrap();
    initialize_default_thresholds(&conn).unwrap();
    (tmp, pool)
}

// Parser test with real fixtures
#[test]
fn test_parse_opengauss_v1() {
    let html = std::fs::read_to_string("../../example/opengauss_v1.html").unwrap();
    let report = parse_complete_wdr_report(&html, "test").unwrap();
    assert!(!report.top_sql.is_empty());
    assert!(report.efficiency.is_some());
}
```

---

## 11. Extension Points & MCP Integration

### Current State

WDRProbe does **not** currently expose an MCP (Model Context Protocol) server or HTTP API. All functionality is accessed through Tauri IPC (frontend ↔ backend within the desktop app).

### How to Extend

#### Option A: Expose Tauri Commands via a Local HTTP Server

To make WDRProbe's capabilities accessible to external tools (including MCP servers), you can add a local HTTP server that wraps the existing IPC commands:

1. Add an HTTP framework to `Cargo.toml`:
   ```toml
   axum = "0.7"
   tower-http = { version = "0.5", features = ["cors"] }
   ```

2. Create an HTTP server module that reuses the existing `DatabaseOperations` trait:
   ```rust
   // src/api/mod.rs
   use axum::{routing::get, Router, extract::State};
   use crate::database::DatabasePool;

   pub async fn start_http_server(pool: DatabasePool) {
       let app = Router::new()
           .route("/api/reports", get(list_reports))
           .route("/api/reports/:id", get(get_report))
           .route("/api/thresholds", get(get_thresholds))
           .with_state(pool);

       let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
       axum::serve(listener, app).await.unwrap();
   }
   ```

3. Start the server in `main.rs` setup:
   ```rust
   .setup(|app| {
       // ... existing setup ...
       let pool_clone = pool.clone();
       tauri::async_runtime::spawn(async move {
           api::start_http_server(pool_clone).await;
       });
       Ok(())
   })
   ```

#### Option B: Build an MCP Server Wrapper

Create a separate MCP server that calls WDRProbe's library crate:

1. The Rust backend is already structured as a library (`wdrprobe_desktop_lib` with `crate-type = ["rlib"]`)
2. Create a new binary crate that depends on `wdrprobe_desktop_lib`
3. Implement MCP server protocol (e.g., using `rmcp` crate)
4. Expose key operations as MCP tools:
   - `import_wdr_report(file_path)` → returns parsed report
   - `analyze_wdr(report_id)` → returns risk analysis
   - `compare_reports(source_id, target_id)` → returns comparison
   - `parse_execution_plan(plan_text)` → returns plan tree + analysis
   - `get_thresholds()` → returns current configuration

#### Option C: Direct SQLite Access

Since the database is a standard SQLite file at `{app_data_dir}/wdrprobe.db`, external tools can read it directly:

```bash
sqlite3 ~/Library/Application\ Support/com.wdrprobe.desktop/wdrprobe.db \
  "SELECT * FROM top_sqls WHERE is_hot_sql = 1 ORDER BY total_elapsed_time DESC LIMIT 10;"
```

Refer to [Section 3](#3-database-schema-reference) for the complete schema.

### Reusable Libraries

The following Rust modules can be extracted as standalone crates for reuse:

| Module | Reusability | Dependencies |
|--------|-------------|--------------|
| `parsers/wdr_parser.rs` | High — pure parsing, no Tauri dependency | scraper, regex |
| `parsers/complete_wdr_parser.rs` | High | scraper, regex |
| `parsers/sql_parser.rs` | High | nom, serde_json |
| `models/*` | High — pure Serde structs | serde |
| `utils/gaussdb.rs` | Medium | serde_json |
| `database/*` | Medium — tied to rusqlite | rusqlite, r2d2 |

### Adding New Analysis Rules

To add a new execution plan analysis rule:

1. Add the rule to `sql_parser.rs` analysis section
2. Add i18n keys in `I18nContext.tsx`:
   ```typescript
   'vis.rule.014.title': 'New Risk Type',
   'vis.rule.014.desc': 'Description of the risk ({param})',
   'vis.rule.014.sugg': 'Suggested fix',
   ```
3. The rule will automatically appear in the Visualizer issues panel

### Adding New Audit Detection Types

To add a new SQL audit detection type:

1. Add the variant to `AuditIssueType` enum in `models/audit.rs`
2. Implement detection logic in `commands/audit.rs` (`run_sql_audit`)
3. Add i18n keys for the new issue type
4. Update the frontend `SqlAudit` page to display the new type

---

> For the authoritative IPC interface specification (in Chinese), see [docs/desktop-IPC.md](desktop-IPC.md).  
> For SpecKit contract files with detailed command specifications, see [specs/001-implement-desktop/contracts/](../specs/001-implement-desktop/contracts/).  
> For contribution guidelines, see [CONTRIBUTION.md](../CONTRIBUTION.md).
