// Database schema initialization
// Creates all necessary tables and indexes for the application

use rusqlite::{Connection, Result};

/// Initialize the database schema
pub fn initialize_schema(conn: &Connection) -> Result<()> {
    // Enable foreign keys
    conn.execute_batch("PRAGMA foreign_keys = ON")?;

    // Create tables
    conn.execute_batch(
        r#"
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

        -- Efficiency Metrics
        CREATE TABLE IF NOT EXISTS efficiency_metrics (
            report_id INTEGER NOT NULL,
            buffer_hit_percent REAL NOT NULL,
            cpu_efficiency_percent REAL NOT NULL,
            soft_parse_rate_percent REAL NOT NULL,
            hard_parse_rate_percent REAL NOT NULL,
            execution_efficiency_percent REAL NOT NULL,
            FOREIGN KEY (report_id) REFERENCES wdr_reports(id)
        );

        -- Load Profile
        CREATE TABLE IF NOT EXISTS load_profile (
            report_id INTEGER NOT NULL,
            db_time_per_sec REAL NOT NULL,
            cpu_time_per_sec REAL NOT NULL,
            io_requests_per_sec REAL NOT NULL,
            total_transactions INTEGER NOT NULL,
            commits_per_sec REAL NOT NULL,
            rollbacks_per_sec REAL NOT NULL,
            FOREIGN KEY (report_id) REFERENCES wdr_reports(id)
        );

        -- Database Statistics
        CREATE TABLE IF NOT EXISTS database_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            report_id INTEGER NOT NULL,
            db_name TEXT NOT NULL,
            backends INTEGER NOT NULL,
            xact_commit INTEGER NOT NULL,
            xact_rollback INTEGER NOT NULL,
            blks_read INTEGER NOT NULL,
            blks_hit INTEGER NOT NULL,
            tuple_returned INTEGER NOT NULL,
            tuple_fetched INTEGER NOT NULL,
            tuple_inserted INTEGER NOT NULL,
            tuple_updated INTEGER NOT NULL,
            tuple_deleted INTEGER NOT NULL,
            conflicts INTEGER NOT NULL,
            temp_files INTEGER NOT NULL,
            temp_bytes INTEGER NOT NULL,
            deadlocks INTEGER NOT NULL,
            blk_read_time REAL NOT NULL,
            blk_write_time REAL NOT NULL,
            stats_reset DATETIME,
            FOREIGN KEY (report_id) REFERENCES wdr_reports(id)
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

        -- Execution Plans
        CREATE TABLE IF NOT EXISTS execution_plans (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sql_id INTEGER,
            plan_tree TEXT NOT NULL,  -- JSON serialized
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            source TEXT NOT NULL,
            FOREIGN KEY (sql_id) REFERENCES top_sqls(id)
        );

        -- Cache IO Statistics
        CREATE TABLE IF NOT EXISTS cache_io_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            report_id INTEGER NOT NULL,
            schema_name TEXT NOT NULL,
            object_name TEXT NOT NULL,
            object_type TEXT NOT NULL,
            heap_blks_read INTEGER NOT NULL,
            heap_blks_hit INTEGER NOT NULL,
            heap_blks_hit_ratio REAL NOT NULL,
            idx_blks_read INTEGER NOT NULL,
            idx_blks_hit INTEGER NOT NULL,
            idx_blks_hit_ratio REAL NOT NULL,
            toast_blks_read INTEGER NOT NULL,
            toast_blks_hit INTEGER NOT NULL,
            toast_blks_hit_ratio REAL NOT NULL,
            tidx_blks_read INTEGER NOT NULL,
            tidx_blks_hit INTEGER NOT NULL,
            tidx_blks_hit_ratio REAL NOT NULL,
            FOREIGN KEY (report_id) REFERENCES wdr_reports(id)
        );

        -- Object Statistics
        CREATE TABLE IF NOT EXISTS object_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            report_id INTEGER NOT NULL,
            schema_name TEXT NOT NULL,
            object_name TEXT NOT NULL,
            object_type TEXT NOT NULL,
            total_scans INTEGER NOT NULL,
            seq_scans INTEGER NOT NULL,
            idx_scans INTEGER NOT NULL,
            seq_reads INTEGER NOT NULL,
            idx_reads INTEGER NOT NULL,
            inserts INTEGER NOT NULL,
            updates INTEGER NOT NULL,
            deletes INTEGER NOT NULL,
            dead_tuples INTEGER NOT NULL,
            needs_vacuum BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (report_id) REFERENCES wdr_reports(id)
        );

        -- Comparisons
        CREATE TABLE IF NOT EXISTS wdr_comparisons (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_report_id INTEGER NOT NULL,
            target_report_id INTEGER NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            comparison_type TEXT NOT NULL,
            performance_score_change INTEGER NOT NULL,
            status TEXT NOT NULL,
            conclusion TEXT NOT NULL,
            key_findings TEXT NOT NULL,  -- JSON serialized array of KeyFinding
            FOREIGN KEY (source_report_id) REFERENCES wdr_reports(id),
            FOREIGN KEY (target_report_id) REFERENCES wdr_reports(id)
        );

        -- SQL Comparison Metrics
        CREATE TABLE IF NOT EXISTS sql_comparison_metrics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            comparison_id INTEGER NOT NULL,
            sql_id INTEGER,
            sql_text_hash TEXT NOT NULL,
            source_metrics TEXT NOT NULL,  -- JSON serialized SqlMetrics
            target_metrics TEXT NOT NULL,  -- JSON serialized SqlMetrics
            change_percentages TEXT NOT NULL,  -- JSON serialized SqlChangePercentages
            FOREIGN KEY (comparison_id) REFERENCES wdr_comparisons(id)
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

        -- SQL Audit Issues
        CREATE TABLE IF NOT EXISTS sql_audit_issues (
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
    "#,
    )?;

    // Create indexes for performance
    conn.execute_batch(r#"
        CREATE INDEX IF NOT EXISTS idx_wdr_reports_instance ON wdr_reports(instance_name);
        CREATE INDEX IF NOT EXISTS idx_wdr_reports_gen_time ON wdr_reports(generation_time);
        CREATE INDEX IF NOT EXISTS idx_wdr_reports_status ON wdr_reports(status);
        CREATE INDEX IF NOT EXISTS idx_wdr_reports_created ON wdr_reports(created_at);

        CREATE INDEX IF NOT EXISTS idx_top_sqls_report_id ON top_sqls(report_id);
        CREATE INDEX IF NOT EXISTS idx_top_sqls_hot_sql ON top_sqls(is_hot_sql);
        CREATE INDEX IF NOT EXISTS idx_top_sqls_rank ON top_sqls(rank_by_time);

        CREATE INDEX IF NOT EXISTS idx_execution_plans_sql_id ON execution_plans(sql_id);
        CREATE INDEX IF NOT EXISTS idx_execution_plans_source ON execution_plans(source);

        CREATE INDEX IF NOT EXISTS idx_object_stats_report_id ON object_stats(report_id);
        CREATE INDEX IF NOT EXISTS idx_object_stats_type ON object_stats(object_type);

        CREATE INDEX IF NOT EXISTS idx_comparisons_source ON wdr_comparisons(source_report_id);
        CREATE INDEX IF NOT EXISTS idx_comparisons_target ON wdr_comparisons(target_report_id);
        CREATE INDEX IF NOT EXISTS idx_comparisons_created ON wdr_comparisons(created_at);
        CREATE INDEX IF NOT EXISTS idx_comparisons_score ON wdr_comparisons(performance_score_change);
        CREATE INDEX IF NOT EXISTS idx_comparisons_status ON wdr_comparisons(status);

        CREATE INDEX IF NOT EXISTS idx_sql_metrics_comparison ON sql_comparison_metrics(comparison_id);
        CREATE INDEX IF NOT EXISTS idx_sql_metrics_hash ON sql_comparison_metrics(sql_text_hash);

        CREATE INDEX IF NOT EXISTS idx_threshold_configs_category ON threshold_configs(category);
        CREATE INDEX IF NOT EXISTS idx_threshold_configs_key ON threshold_configs(config_key);

        CREATE INDEX IF NOT EXISTS idx_audit_issues_report ON sql_audit_issues(report_id);
        CREATE INDEX IF NOT EXISTS idx_audit_issues_status ON sql_audit_issues(status);
        CREATE INDEX IF NOT EXISTS idx_audit_issues_severity ON sql_audit_issues(severity);
        CREATE INDEX IF NOT EXISTS idx_audit_issues_type ON sql_audit_issues(issue_type);

        CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp);
        CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
        CREATE INDEX IF NOT EXISTS idx_audit_logs_entity ON audit_logs(entity_type, entity_id);

        -- Composite indexes for common query patterns
        CREATE INDEX IF NOT EXISTS idx_audit_issues_report_status ON sql_audit_issues(report_id, status);
        CREATE INDEX IF NOT EXISTS idx_audit_issues_report_severity ON sql_audit_issues(report_id, severity);
        CREATE INDEX IF NOT EXISTS idx_wdr_reports_instance_time ON wdr_reports(instance_name, generation_time DESC);
        CREATE INDEX IF NOT EXISTS idx_top_sqls_report_rank ON top_sqls(report_id, rank_by_time);
    "#)?;

    // Optimize SQLite settings for performance
    conn.execute_batch(r#"
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA cache_size = -64000;  -- 64MB cache
        PRAGMA temp_store = MEMORY;
        PRAGMA mmap_size = 268435456;  -- 256MB mmap
    "#)?;

    Ok(())
}
