# Database Performance Optimization

## SQLite Configuration

### Pragma Settings

The database is initialized with optimized SQLite settings:

```sql
PRAGMA journal_mode = WAL;        -- Write-Ahead Logging for better concurrency
PRAGMA synchronous = NORMAL;       -- Faster writes with safety
PRAGMA cache_size = -64000;        -- 64MB cache for in-memory data
PRAGMA temp_store = MEMORY;        -- Store temp tables in memory
PRAGMA mmap_size = 268435456;      -- 256MB memory-mapped I/O
```

### Benefits

- **WAL Mode**: Allows readers to proceed without blocking writers
- **Normal Sync**: Balances safety and performance (fsync every ~1000ms)
- **Large Cache**: Reduces disk I/O for hot data
- **Memory Temp Store**: Faster temporary table operations
- **mmap I/O**: Reduced system call overhead

## Indexes

### Single Column Indexes

| Index | Table | Columns | Purpose |
|-------|-------|---------|---------|
| `idx_wdr_reports_instance` | wdr_reports | instance_name | Filter by instance |
| `idx_wdr_reports_gen_time` | wdr_reports | generation_time | Sort by time |
| `idx_wdr_reports_status` | wdr_reports | status | Filter by status |
| `idx_wdr_reports_created` | wdr_reports | created_at | Recent imports |
| `idx_top_sqls_report_id` | top_sqls | report_id | Query SQL by report |
| `idx_top_sqls_hot_sql` | top_sqls | is_hot_sql | Filter hot SQLs |
| `idx_top_sqls_rank` | top_sqls | rank_by_time | Sort by rank |
| `idx_audit_issues_report` | sql_audit_issues | report_id | Query issues by report |
| `idx_audit_issues_status` | sql_audit_issues | status | Filter by status |
| `idx_audit_issues_severity` | sql_audit_issues | severity | Filter by severity |
| `idx_audit_issues_type` | sql_audit_issues | issue_type | Filter by type |
| `idx_audit_logs_timestamp` | audit_logs | timestamp | Query logs by time |
| `idx_audit_logs_action` | audit_logs | action | Query logs by action |
| `idx_audit_logs_entity` | audit_logs | entity_type, entity_id | Query logs by entity |

### Composite Indexes

| Index | Table | Columns | Purpose |
|-------|-------|---------|---------|
| `idx_audit_issues_report_status` | sql_audit_issues | report_id, status | Filter issues by report & status |
| `idx_audit_issues_report_severity` | sql_audit_issues | report_id, severity | Filter issues by report & severity |
| `idx_wdr_reports_instance_time` | wdr_reports | instance_name, generation_time DESC | Get latest reports per instance |
| `idx_top_sqls_report_rank` | top_sqls | report_id, rank_by_time | Get ranked SQLs by report |

### Index Guidelines

1. **Filter First, Then Sort**: Use composite indexes where the first column is a filter
2. **Most Selective First**: Place columns with higher cardinality first
3. **Cover Indexes**: Include columns that are frequently accessed together

## Query Optimization

### Pagination

Always use `LIMIT` and `OFFSET` for large result sets:

```rust
let query = "SELECT * FROM wdr_reports ORDER BY generation_time DESC LIMIT ? OFFSET ?";
```

### Prepared Statements

The `rusqlite` library automatically prepares and caches statements.

### Batch Operations

Use transactions for multiple writes:

```rust
conn.execute("BEGIN TRANSACTION", [])?;

for item in items {
    // Execute inserts
}

conn.execute("COMMIT", [])?;
```

### Connection Pooling

The application uses `r2d2` for connection pooling:

```rust
pool: State<'_, DatabasePool>
```

Default pool size: 10 connections (configurable).

## Memory Management

### WDR Parsing

- Use streaming parsers for large WDR files
- Process data in chunks to avoid loading entire file in memory

### Execution Plans

- Store execution plans as JSON (compressed)
- Lazy load plan tree only when needed

## Monitoring

### Analyze Query Performance

```sql
EXPLAIN QUERY PLAN SELECT * FROM top_sqls WHERE report_id = ? ORDER BY rank_by_time;
```

### Check Index Usage

```sql
SELECT * FROM pragma_index_list('wdr_reports');
```

### Database Size

```sql
SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size();
```

## Best Practices

1. **Use Transactions**: Always wrap multiple writes in a transaction
2. **Avoid N+1 Queries**: Use JOINs or batch operations
3. **Prepare Statements**: Let rusqlite cache prepared statements
4. **Close Connections**: Return connections to pool promptly
5. **Profile Queries**: Use `EXPLAIN QUERY PLAN` for slow queries
6. **Vacuum Periodically**: Reclaim unused space (after large deletes)

## Performance Benchmarks

### Import Operations

| Operation | Expected Time |
|-----------|---------------|
| Import WDR (100 SQLs) | < 2 seconds |
| Import WDR (500 SQLs) | < 5 seconds |
| Create Comparison | < 1 second |

### Query Operations

| Operation | Expected Time |
|-----------|---------------|
| List reports (50 items) | < 100ms |
| Get report details | < 200ms |
| Get hot SQLs (100 items) | < 150ms |
| Run SQL audit | < 5 seconds |
| Get comparison metrics | < 500ms |

### Export Operations

| Operation | Expected Time |
|-----------|---------------|
| Export report (JSON) | < 1 second |
| Export report (CSV) | < 2 seconds |
| Export report (PDF) | < 5 seconds |

## Troubleshooting

### Slow Queries

1. Check index usage with `EXPLAIN QUERY PLAN`
2. Verify indexes exist: `SELECT * FROM pragma_index_info('table_name')`
3. Check database statistics: `ANALYZE` command

### High Memory Usage

1. Reduce `cache_size` if needed
2. Process large imports in smaller batches
3. Close unused connections

### Lock Contention

1. Ensure WAL mode is enabled
2. Keep transactions short
3. Avoid long-running reads during writes
