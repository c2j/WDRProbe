// Database CRUD operations
// Provides all database operations for the application

use super::DatabasePool;
use crate::models::dashboard::WdrReportSummary;
use crate::models::*;
use rusqlite::{params, Result};

/// Database operations trait
pub trait DatabaseOperations {
    fn create_wdr_report(&self, report: &WdrReport) -> Result<i64>;
    fn get_wdr_report(&self, id: i64) -> Result<Option<WdrReport>>;
    fn list_wdr_reports(&self, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<WdrReport>>;
    fn delete_wdr_report(&self, id: i64) -> Result<()>;

    fn create_top_sql(&self, sql: &TopSql) -> Result<i64>;
    fn get_top_sqls_by_report(&self, report_id: i64) -> Result<Vec<TopSql>>;
    fn get_hot_sqls(&self, limit: Option<i32>) -> Result<Vec<TopSql>>;

    fn create_threshold_config(&self, threshold: &ThresholdConfig) -> Result<i64>;
    fn update_threshold_config(&self, config_key: &str, value: f64, changed_by: &str)
        -> Result<()>;
    fn get_threshold_configs(&self, category: Option<&str>) -> Result<Vec<ThresholdConfig>>;
    fn get_threshold_config(&self, config_key: &str) -> Result<Option<ThresholdConfig>>;

    fn create_audit_log(&self, log: &AuditLog) -> Result<i64>;
    fn get_audit_logs(&self, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<AuditLog>>;

    fn create_database_stats(&self, stats: &DatabaseStats) -> Result<i64>;
    fn get_database_stats(&self, report_id: i64) -> Result<Vec<DatabaseStats>>;

    fn create_cache_io_stats(&self, io: &CacheIoStats) -> Result<i64>;
    fn get_cache_io_stats(&self, report_id: i64) -> Result<Vec<CacheIoStats>>;

    fn create_object_stats(&self, obj: &ObjectStats) -> Result<i64>;
    fn get_object_stats(&self, report_id: i64) -> Result<Vec<ObjectStats>>;

    fn create_efficiency_metrics(&self, metrics: &EfficiencyMetrics) -> Result<i64>;
    fn get_efficiency_metrics(&self, report_id: i64) -> Result<Option<EfficiencyMetrics>>;

    fn create_load_profile(&self, profile: &LoadProfile) -> Result<i64>;
    fn get_load_profile(&self, report_id: i64) -> Result<Option<LoadProfile>>;

    fn create_execution_plan(&self, plan: &crate::models::SqlExecutionPlan) -> Result<i64>;
    fn get_execution_plan_by_sql(
        &self,
        sql_id: i64,
    ) -> Result<Option<crate::models::SqlExecutionPlan>>;
    fn get_execution_plans_by_report(
        &self,
        report_id: i64,
    ) -> Result<Vec<crate::models::SqlExecutionPlan>>;
    fn delete_execution_plan(&self, plan_id: i64) -> Result<()>;

    // Comparison operations
    fn create_comparison(
        &self,
        source_report_id: i64,
        target_report_id: i64,
        comparison_type: &str,
        summary: &crate::models::comparison::ComparisonSummary,
    ) -> Result<i64>;
    fn get_comparison_summary(
        &self,
        comparison_id: i64,
    ) -> Result<Option<crate::models::comparison::ComparisonSummary>>;
    fn get_comparisons(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
        sort_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> Result<Vec<crate::models::comparison::WdrComparisonListItem>>;
    fn count_comparisons(&self) -> Result<i64>;
    fn delete_comparison(&self, comparison_id: i64) -> Result<()>;
    fn create_sql_comparison_metric(
        &self,
        comparison_id: i64,
        metric: &crate::models::comparison::SqlComparisonMetric,
    ) -> Result<i64>;
    fn get_comparison_details(
        &self,
        comparison_id: i64,
        category: &str,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<crate::models::comparison::ComparisonDetails>;
    fn get_comparison_chart_data(
        &self,
        comparison_id: i64,
        chart_type: &str,
    ) -> Result<crate::models::comparison::ChartData>;

    fn get_instance_summaries(&self) -> Result<Vec<crate::models::dashboard::InstanceSummary>>;
    fn get_dashboard_metrics(
        &self,
        instance_name: Option<&str>,
    ) -> Result<crate::models::dashboard::DashboardMetrics>;
}

impl DatabaseOperations for DatabasePool {
    fn create_wdr_report(&self, report: &WdrReport) -> Result<i64> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        conn.execute(
            r#"
            INSERT INTO wdr_reports (
                instance_name, generation_time, snapshot_start, snapshot_end,
                file_path, file_size, status
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                report.instance_name,
                report.generation_time,
                report.snapshot_start,
                report.snapshot_end,
                report.file_path,
                report.file_size,
                report.status
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    fn get_wdr_report(&self, id: i64) -> Result<Option<WdrReport>> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let mut stmt = conn.prepare("SELECT * FROM wdr_reports WHERE id = ?")?;

        let mut report_iter = stmt.query_map(params![id], |row| {
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
        })?;

        match report_iter.next() {
            Some(Ok(report)) => Ok(Some(report)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    fn list_wdr_reports(&self, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<WdrReport>> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let limit_clause = match limit {
            Some(l) => format!("LIMIT {}", l),
            None => "".to_string(),
        };

        let offset_clause = match offset {
            Some(o) => format!("OFFSET {}", o),
            None => "".to_string(),
        };

        let query = format!(
            "SELECT * FROM wdr_reports ORDER BY created_at DESC {} {}",
            limit_clause, offset_clause
        );

        let mut stmt = conn.prepare(&query)?;

        let report_iter = stmt.query_map([], |row| {
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
        })?;

        let mut reports = Vec::new();
        for report in report_iter {
            reports.push(report?);
        }

        Ok(reports)
    }

    fn delete_wdr_report(&self, id: i64) -> Result<()> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        conn.execute("DELETE FROM wdr_reports WHERE id = ?", params![id])?;

        Ok(())
    }

    fn create_top_sql(&self, sql: &TopSql) -> Result<i64> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        conn.execute(
            r#"
            INSERT INTO top_sqls (
                report_id, sql_id, sql_text, executions, total_elapsed_time,
                cpu_time, io_time, buffer_gets, disk_reads, rows_processed,
                first_load_time, last_load_time, is_hot_sql, rank_by_time
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                sql.report_id,
                sql.sql_id,
                sql.sql_text,
                sql.executions,
                sql.total_elapsed_time,
                sql.cpu_time,
                sql.io_time,
                sql.buffer_gets,
                sql.disk_reads,
                sql.rows_processed,
                sql.first_load_time,
                sql.last_load_time,
                sql.is_hot_sql,
                sql.rank_by_time
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    fn get_top_sqls_by_report(&self, report_id: i64) -> Result<Vec<TopSql>> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let mut stmt =
            conn.prepare("SELECT * FROM top_sqls WHERE report_id = ? ORDER BY rank_by_time")?;

        let sql_iter = stmt.query_map(params![report_id], |row| {
            Ok(TopSql {
                id: row.get("id")?,
                report_id: row.get("report_id")?,
                sql_id: row.get("sql_id")?,
                sql_text: row.get("sql_text")?,
                executions: row.get("executions")?,
                total_elapsed_time: row.get("total_elapsed_time")?,
                cpu_time: row.get("cpu_time")?,
                io_time: row.get("io_time")?,
                buffer_gets: row.get("buffer_gets")?,
                disk_reads: row.get("disk_reads")?,
                rows_processed: row.get("rows_processed")?,
                first_load_time: row.get("first_load_time")?,
                last_load_time: row.get("last_load_time")?,
                is_hot_sql: row.get("is_hot_sql")?,
                rank_by_time: row.get("rank_by_time")?,
            })
        })?;

        let mut sqls = Vec::new();
        for sql in sql_iter {
            sqls.push(sql?);
        }

        Ok(sqls)
    }

    fn get_hot_sqls(&self, limit: Option<i32>) -> Result<Vec<TopSql>> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let limit_clause = match limit {
            Some(l) => format!("LIMIT {}", l),
            None => "".to_string(),
        };

        let query = format!(
            "SELECT * FROM top_sqls WHERE is_hot_sql = 1 ORDER BY rank_by_time {}",
            limit_clause
        );

        let mut stmt = conn.prepare(&query)?;

        let sql_iter = stmt.query_map([], |row| {
            Ok(TopSql {
                id: row.get("id")?,
                report_id: row.get("report_id")?,
                sql_id: row.get("sql_id")?,
                sql_text: row.get("sql_text")?,
                executions: row.get("executions")?,
                total_elapsed_time: row.get("total_elapsed_time")?,
                cpu_time: row.get("cpu_time")?,
                io_time: row.get("io_time")?,
                buffer_gets: row.get("buffer_gets")?,
                disk_reads: row.get("disk_reads")?,
                rows_processed: row.get("rows_processed")?,
                first_load_time: row.get("first_load_time")?,
                last_load_time: row.get("last_load_time")?,
                is_hot_sql: row.get("is_hot_sql")?,
                rank_by_time: row.get("rank_by_time")?,
            })
        })?;

        let mut sqls = Vec::new();
        for sql in sql_iter {
            sqls.push(sql?);
        }

        Ok(sqls)
    }

    fn create_threshold_config(&self, threshold: &ThresholdConfig) -> Result<i64> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        conn.execute(
            r#"
            INSERT INTO threshold_configs (
                category, data_type, config_key, value, default_value,
                min_value, max_value, description, updated_by
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                threshold.category,
                threshold.data_type,
                threshold.config_key,
                threshold.value,
                threshold.default_value,
                threshold.min_value,
                threshold.max_value,
                threshold.description,
                threshold.updated_by
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    fn update_threshold_config(
        &self,
        config_key: &str,
        value: f64,
        changed_by: &str,
    ) -> Result<()> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        conn.execute(
            "UPDATE threshold_configs SET value = ?, updated_at = CURRENT_TIMESTAMP, updated_by = ? WHERE config_key = ?",
            params![value, changed_by, config_key]
        )?;

        Ok(())
    }

    fn get_threshold_configs(
        &self,
        category: Option<&str>,
    ) -> Result<Vec<ThresholdConfig>, rusqlite::Error> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let query = match category {
            Some(_cat) => "SELECT * FROM threshold_configs WHERE category = ? ORDER BY config_key",
            None => "SELECT * FROM threshold_configs ORDER BY category, config_key",
        };

        let mut thresholds = Vec::new();

        if let Some(cat) = category {
            let mut stmt = conn.prepare(query)?;
            let iter = stmt.query_map(params![cat], |row| {
                Ok(ThresholdConfig {
                    id: row.get("id")?,
                    category: row.get("category")?,
                    data_type: row.get("data_type")?,
                    config_key: row.get("config_key")?,
                    value: row.get("value")?,
                    default_value: row.get("default_value")?,
                    min_value: row.get("min_value")?,
                    max_value: row.get("max_value")?,
                    description: row.get("description")?,
                    updated_at: row.get("updated_at")?,
                    updated_by: row.get("updated_by")?,
                })
            })?;

            for threshold in iter {
                thresholds.push(threshold?);
            }
        } else {
            let mut stmt = conn.prepare(query)?;
            let iter = stmt.query_map([], |row| {
                Ok(ThresholdConfig {
                    id: row.get("id")?,
                    category: row.get("category")?,
                    data_type: row.get("data_type")?,
                    config_key: row.get("config_key")?,
                    value: row.get("value")?,
                    default_value: row.get("default_value")?,
                    min_value: row.get("min_value")?,
                    max_value: row.get("max_value")?,
                    description: row.get("description")?,
                    updated_at: row.get("updated_at")?,
                    updated_by: row.get("updated_by")?,
                })
            })?;

            for threshold in iter {
                thresholds.push(threshold?);
            }
        }

        Ok(thresholds)
    }

    fn get_threshold_config(
        &self,
        config_key: &str,
    ) -> Result<Option<ThresholdConfig>, rusqlite::Error> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let mut stmt = conn.prepare("SELECT * FROM threshold_configs WHERE config_key = ?")?;

        let mut iter = stmt.query_map(params![config_key], |row| {
            Ok(ThresholdConfig {
                id: row.get("id")?,
                category: row.get("category")?,
                data_type: row.get("data_type")?,
                config_key: row.get("config_key")?,
                value: row.get("value")?,
                default_value: row.get("default_value")?,
                min_value: row.get("min_value")?,
                max_value: row.get("max_value")?,
                description: row.get("description")?,
                updated_at: row.get("updated_at")?,
                updated_by: row.get("updated_by")?,
            })
        })?;

        match iter.next() {
            Some(Ok(threshold)) => Ok(Some(threshold)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    fn create_audit_log(&self, log: &AuditLog) -> Result<i64> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        conn.execute(
            r#"
            INSERT INTO audit_logs (
                user_id, action, entity_type, entity_id, old_value,
                new_value, ip_address, success, error_message, details
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                log.user_id,
                log.action,
                log.entity_type,
                log.entity_id,
                log.old_value,
                log.new_value,
                log.ip_address,
                log.success,
                log.error_message,
                log.details
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    fn get_audit_logs(&self, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<AuditLog>> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let limit_clause = match limit {
            Some(l) => format!("LIMIT {}", l),
            None => "".to_string(),
        };

        let offset_clause = match offset {
            Some(o) => format!("OFFSET {}", o),
            None => "".to_string(),
        };

        let query = format!(
            "SELECT * FROM audit_logs ORDER BY timestamp DESC {} {}",
            limit_clause, offset_clause
        );

        let mut stmt = conn.prepare(&query)?;

        let log_iter = stmt.query_map([], |row| {
            Ok(AuditLog {
                id: row.get("id")?,
                timestamp: row.get("timestamp")?,
                user_id: row.get("user_id")?,
                action: row.get("action")?,
                entity_type: row.get("entity_type")?,
                entity_id: row.get("entity_id")?,
                old_value: row.get("old_value")?,
                new_value: row.get("new_value")?,
                ip_address: row.get("ip_address")?,
                success: row.get("success")?,
                error_message: row.get("error_message")?,
                details: row.get("details")?,
            })
        })?;

        let mut logs = Vec::new();
        for log in log_iter {
            logs.push(log?);
        }

        Ok(logs)
    }

    fn create_database_stats(&self, stats: &DatabaseStats) -> Result<i64> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        conn.execute(
            r#"
            INSERT INTO database_stats (
                report_id, db_name, backends, xact_commit, xact_rollback,
                blks_read, blks_hit, tuple_returned, tuple_fetched,
                tuple_inserted, tuple_updated, tuple_deleted, conflicts,
                temp_files, temp_bytes, deadlocks, blk_read_time, blk_write_time, stats_reset
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                stats.report_id,
                stats.db_name,
                stats.backends,
                stats.xact_commit,
                stats.xact_rollback,
                stats.blks_read,
                stats.blks_hit,
                stats.tuple_returned,
                stats.tuple_fetched,
                stats.tuple_inserted,
                stats.tuple_updated,
                stats.tuple_deleted,
                stats.conflicts,
                stats.temp_files,
                stats.temp_bytes,
                stats.deadlocks,
                stats.blk_read_time,
                stats.blk_write_time,
                &stats.stats_reset
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    fn get_database_stats(&self, report_id: i64) -> Result<Vec<DatabaseStats>> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let mut stmt =
            conn.prepare("SELECT * FROM database_stats WHERE report_id = ? ORDER BY db_name")?;

        let stats_iter = stmt.query_map(params![report_id], |row| {
            Ok(DatabaseStats {
                id: row.get("id")?,
                report_id: row.get("report_id")?,
                db_name: row.get("db_name")?,
                backends: row.get("backends")?,
                xact_commit: row.get("xact_commit")?,
                xact_rollback: row.get("xact_rollback")?,
                blks_read: row.get("blks_read")?,
                blks_hit: row.get("blks_hit")?,
                tuple_returned: row.get("tuple_returned")?,
                tuple_fetched: row.get("tuple_fetched")?,
                tuple_inserted: row.get("tuple_inserted")?,
                tuple_updated: row.get("tuple_updated")?,
                tuple_deleted: row.get("tuple_deleted")?,
                conflicts: row.get("conflicts")?,
                temp_files: row.get("temp_files")?,
                temp_bytes: row.get("temp_bytes")?,
                deadlocks: row.get("deadlocks")?,
                blk_read_time: row.get("blk_read_time")?,
                blk_write_time: row.get("blk_write_time")?,
                stats_reset: row.get("stats_reset")?,
            })
        })?;

        let mut stats = Vec::new();
        for stat in stats_iter {
            stats.push(stat?);
        }

        Ok(stats)
    }

    fn create_cache_io_stats(&self, io: &CacheIoStats) -> Result<i64> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        conn.execute(
            r#"
            INSERT INTO cache_io_stats (
                report_id, schema_name, object_name, object_type,
                heap_blks_read, heap_blks_hit, heap_blks_hit_ratio,
                idx_blks_read, idx_blks_hit, idx_blks_hit_ratio,
                toast_blks_read, toast_blks_hit, toast_blks_hit_ratio,
                tidx_blks_read, tidx_blks_hit, tidx_blks_hit_ratio
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                io.report_id,
                io.schema_name,
                io.object_name,
                io.object_type,
                io.heap_blks_read,
                io.heap_blks_hit,
                io.heap_blks_hit_ratio,
                io.idx_blks_read,
                io.idx_blks_hit,
                io.idx_blks_hit_ratio,
                io.toast_blks_read,
                io.toast_blks_hit,
                io.toast_blks_hit_ratio,
                io.tidx_blks_read,
                io.tidx_blks_hit,
                io.tidx_blks_hit_ratio
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    fn get_cache_io_stats(&self, report_id: i64) -> Result<Vec<CacheIoStats>> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let mut stmt = conn.prepare(
            "SELECT * FROM cache_io_stats WHERE report_id = ? ORDER BY schema_name, object_name",
        )?;

        let stats_iter = stmt.query_map(params![report_id], |row| {
            Ok(CacheIoStats {
                id: row.get("id")?,
                report_id: row.get("report_id")?,
                schema_name: row.get("schema_name")?,
                object_name: row.get("object_name")?,
                object_type: row.get("object_type")?,
                heap_blks_read: row.get("heap_blks_read")?,
                heap_blks_hit: row.get("heap_blks_hit")?,
                heap_blks_hit_ratio: row.get("heap_blks_hit_ratio")?,
                idx_blks_read: row.get("idx_blks_read")?,
                idx_blks_hit: row.get("idx_blks_hit")?,
                idx_blks_hit_ratio: row.get("idx_blks_hit_ratio")?,
                toast_blks_read: row.get("toast_blks_read")?,
                toast_blks_hit: row.get("toast_blks_hit")?,
                toast_blks_hit_ratio: row.get("toast_blks_hit_ratio")?,
                tidx_blks_read: row.get("tidx_blks_read")?,
                tidx_blks_hit: row.get("tidx_blks_hit")?,
                tidx_blks_hit_ratio: row.get("tidx_blks_hit_ratio")?,
            })
        })?;

        let mut stats = Vec::new();
        for stat in stats_iter {
            stats.push(stat?);
        }

        Ok(stats)
    }

    fn create_object_stats(&self, obj: &ObjectStats) -> Result<i64> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        conn.execute(
            r#"
            INSERT INTO object_stats (
                report_id, schema_name, object_name, object_type,
                total_scans, seq_scans, idx_scans, seq_reads, idx_reads,
                inserts, updates, deletes, dead_tuples, needs_vacuum
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                obj.report_id,
                obj.schema_name,
                obj.object_name,
                obj.object_type,
                obj.total_scans,
                obj.seq_scans,
                obj.idx_scans,
                obj.seq_reads,
                obj.idx_reads,
                obj.inserts,
                obj.updates,
                obj.deletes,
                obj.dead_tuples,
                obj.needs_vacuum
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    fn get_object_stats(&self, report_id: i64) -> Result<Vec<ObjectStats>> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let mut stmt = conn.prepare(
            "SELECT * FROM object_stats WHERE report_id = ? ORDER BY schema_name, object_name",
        )?;

        let stats_iter = stmt.query_map(params![report_id], |row| {
            Ok(ObjectStats {
                id: row.get("id")?,
                report_id: row.get("report_id")?,
                schema_name: row.get("schema_name")?,
                object_name: row.get("object_name")?,
                object_type: row.get("object_type")?,
                total_scans: row.get("total_scans")?,
                seq_scans: row.get("seq_scans")?,
                idx_scans: row.get("idx_scans")?,
                seq_reads: row.get("seq_reads")?,
                idx_reads: row.get("idx_reads")?,
                inserts: row.get("inserts")?,
                updates: row.get("updates")?,
                deletes: row.get("deletes")?,
                dead_tuples: row.get("dead_tuples")?,
                needs_vacuum: row.get("needs_vacuum")?,
            })
        })?;

        let mut stats = Vec::new();
        for stat in stats_iter {
            stats.push(stat?);
        }

        Ok(stats)
    }

    fn get_instance_summaries(&self) -> Result<Vec<crate::models::dashboard::InstanceSummary>> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let mut stmt = conn.prepare(
            "SELECT
                instance_name,
                COUNT(*) as report_count,
                MAX(created_at) as last_report_time,
                SUM(CASE WHEN status = 'SuccessfullyImported' THEN 1 ELSE 0 END) as successful_reports,
                SUM(CASE WHEN status = 'ImportFailed' THEN 1 ELSE 0 END) as failed_reports
             FROM wdr_reports
             GROUP BY instance_name
             ORDER BY instance_name"
        )?;

        let summary_iter = stmt.query_map([], |row| {
            let instance_name: String = row.get("instance_name")?;
            let report_count: i64 = row.get("report_count")?;
            let successful_reports: i64 = row.get("successful_reports")?;
            let failed_reports: i64 = row.get("failed_reports")?;
            let last_report_time: String = row.get("last_report_time")?;

            // Calculate health score based on successful vs failed imports
            let health_score = if report_count > 0 {
                ((successful_reports as f64 / report_count as f64) * 100.0) as i32
            } else {
                100
            };

            // Determine status based on recent activity and health
            let status = if failed_reports > successful_reports {
                crate::models::dashboard::InstanceStatus::Critical
            } else if failed_reports > 0 {
                crate::models::dashboard::InstanceStatus::Warning
            } else {
                crate::models::dashboard::InstanceStatus::Healthy
            };

            // Parse last report time
            let last_report_time = chrono::DateTime::parse_from_rfc3339(&last_report_time)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc));

            Ok(crate::models::dashboard::InstanceSummary {
                instance_name,
                status,
                health_score,
                active_issues: failed_reports as i32,
                report_count: report_count as u64,
                last_report_time,
            })
        })?;

        let mut summaries = Vec::new();
        for summary in summary_iter {
            summaries.push(summary?);
        }

        Ok(summaries)
    }

    fn get_dashboard_metrics(
        &self,
        instance_name: Option<&str>,
    ) -> Result<crate::models::dashboard::DashboardMetrics> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        // Get recent reports for trend calculation
        let mut stmt = if let Some(_name) = instance_name {
            conn.prepare(
                "SELECT * FROM wdr_reports
                 WHERE instance_name = ?
                 ORDER BY generation_time DESC
                 LIMIT 10",
            )?
        } else {
            conn.prepare(
                "SELECT * FROM wdr_reports
                 ORDER BY generation_time DESC
                 LIMIT 10",
            )?
        };

        let mut recent_reports = Vec::new();
        let mut total_cpu = 0.0;
        let mut total_memory = 0.0;
        let mut count = 0;

        // Use a helper function to avoid closure type issues
        fn process_row(row: &rusqlite::Row) -> rusqlite::Result<WdrReportSummary> {
            let id: i64 = row.get("id")?;
            let instance_name: String = row.get("instance_name")?;
            let generation_time_str: String = row.get("generation_time")?;
            let snapshot_start_str: String = row.get("snapshot_start")?;
            let snapshot_end_str: String = row.get("snapshot_end")?;
            let status_str: String = row.get("status")?;

            // Parse status
            let status = match status_str.as_str() {
                "SuccessfullyImported" => {
                    crate::models::dashboard::ReportStatus::SuccessfullyImported
                }
                "PartiallyImported" => crate::models::dashboard::ReportStatus::PartiallyImported,
                s if s.starts_with("ImportFailed(") => {
                    let error = s
                        .strip_prefix("ImportFailed(")
                        .and_then(|s| s.strip_suffix(")"))
                        .unwrap_or("Unknown error")
                        .to_string();
                    crate::models::dashboard::ReportStatus::ImportFailed(error)
                }
                _ => crate::models::dashboard::ReportStatus::ImportFailed(
                    "Unknown status".to_string(),
                ),
            };

            // Parse timestamps
            let generation_time = chrono::DateTime::parse_from_rfc3339(&generation_time_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            let snapshot_start = chrono::DateTime::parse_from_rfc3339(&snapshot_start_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            let snapshot_end = chrono::DateTime::parse_from_rfc3339(&snapshot_end_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            Ok(WdrReportSummary {
                id,
                instance_name,
                generation_time,
                snapshot_start,
                snapshot_end,
                status,
            })
        }

        if let Some(name) = instance_name {
            let rows = stmt.query_map(params![name], process_row)?;
            for row in rows {
                let summary = row?;
                // In a real implementation, you would calculate actual metrics from report data
                // For now, we'll use placeholder values
                total_cpu += 45.0;
                total_memory += 62.0;
                count += 1;
                recent_reports.push(summary);
            }
        } else {
            let rows = stmt.query_map([], process_row)?;
            for row in rows {
                let summary = row?;
                // In a real implementation, you would calculate actual metrics from report data
                // For now, we'll use placeholder values
                total_cpu += 45.0;
                total_memory += 62.0;
                count += 1;
                recent_reports.push(summary);
            }
        }

        // Calculate averages
        let avg_cpu = if count > 0 {
            total_cpu / count as f64
        } else {
            0.0
        };
        let avg_memory = if count > 0 {
            total_memory / count as f64
        } else {
            0.0
        };

        // Generate trend data (simplified)
        let mut trend_data = Vec::new();
        let base_time = chrono::Utc::now();
        for i in 0..10 {
            trend_data.push(crate::models::dashboard::TrendPoint {
                timestamp: base_time - chrono::Duration::hours((10 - i) as i64),
                cpu: avg_cpu + (i as f64 * 2.0) % 20.0,
                memory: avg_memory + (i as f64 * 1.5) % 15.0,
                tps: 1250.0 + (i as f64 * 50.0) % 200.0,
                qps: 3400.0 + (i as f64 * 100.0) % 500.0,
            });
        }

        // Get hot issues (simplified)
        let _hot_issues = vec![
            crate::models::dashboard::HotIssue {
                title: "High CPU Usage".to_string(),
                count: 5,
                severity: crate::models::dashboard::AuditSeverity::High,
                category: crate::models::dashboard::FindingCategory::System,
            },
            crate::models::dashboard::HotIssue {
                title: "Slow Queries".to_string(),
                count: 12,
                severity: crate::models::dashboard::AuditSeverity::Critical,
                category: crate::models::dashboard::FindingCategory::Sql,
            },
        ];

        // Format the values for frontend
        let cpu = format!("{:.0}%", avg_cpu);
        let mem = format!("{:.0}%", avg_memory);
        let tps = format!("{:.1}k", 1250.0 / 1000.0);
        let qps = format!("{:.1}k", 3400.0 / 1000.0);

        // Generate health distribution based on instance health
        let health_distribution = vec![
            crate::models::dashboard::HealthDistributionItem {
                name: "Healthy".to_string(),
                value: if avg_cpu < 70.0 { 90 } else { 60 },
            },
            crate::models::dashboard::HealthDistributionItem {
                name: "Warning".to_string(),
                value: if avg_cpu < 70.0 { 8 } else { 30 },
            },
            crate::models::dashboard::HealthDistributionItem {
                name: "Critical".to_string(),
                value: if avg_cpu < 70.0 { 2 } else { 10 },
            },
        ];

        // Convert trend data to frontend format
        let trend_data: Vec<crate::models::dashboard::TrendDataPoint> = (0..6)
            .map(|i| {
                let hour = 8 + i;
                crate::models::dashboard::TrendDataPoint {
                    time: format!("{:02}:00", hour),
                    value: (avg_cpu as u32 + (i * 5) % 20).min(100),
                }
            })
            .collect();

        // Format hot issues for frontend
        let hot_issues = vec![
            crate::models::dashboard::HotIssue {
                title: "High CPU Usage".to_string(),
                count: 5,
                severity: crate::models::dashboard::AuditSeverity::High,
                category: crate::models::dashboard::FindingCategory::System,
            },
            crate::models::dashboard::HotIssue {
                title: "Slow Queries".to_string(),
                count: 12,
                severity: crate::models::dashboard::AuditSeverity::Critical,
                category: crate::models::dashboard::FindingCategory::Sql,
            },
        ];

        Ok(crate::models::dashboard::DashboardMetrics {
            instance_name: instance_name.map(|s| s.to_string()),
            cpu,
            mem,
            tps,
            qps,
            health_distribution,
            trend_data,
            hot_issues,
            // Store raw values for internal use
            cpu_usage_percent: avg_cpu,
            memory_usage_percent: avg_memory,
            tps_raw: 1250.0,
            qps_raw: 3400.0,
        })
    }

    fn create_efficiency_metrics(&self, metrics: &EfficiencyMetrics) -> Result<i64> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        conn.execute(
            r#"
            INSERT INTO efficiency_metrics (
                report_id, buffer_hit_percent, cpu_efficiency_percent,
                soft_parse_rate_percent, hard_parse_rate_percent, execution_efficiency_percent
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#,
            params![
                metrics.report_id,
                metrics.buffer_hit_percent,
                metrics.cpu_efficiency_percent,
                metrics.soft_parse_rate_percent,
                metrics.hard_parse_rate_percent,
                metrics.execution_efficiency_percent
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    fn get_efficiency_metrics(&self, report_id: i64) -> Result<Option<EfficiencyMetrics>> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let mut stmt = conn.prepare("SELECT * FROM efficiency_metrics WHERE report_id = ?")?;

        let mut metrics_iter = stmt.query_map(params![report_id], |row| {
            Ok(EfficiencyMetrics {
                report_id: row.get("report_id")?,
                buffer_hit_percent: row.get("buffer_hit_percent")?,
                cpu_efficiency_percent: row.get("cpu_efficiency_percent")?,
                soft_parse_rate_percent: row.get("soft_parse_rate_percent")?,
                hard_parse_rate_percent: row.get("hard_parse_rate_percent")?,
                execution_efficiency_percent: row.get("execution_efficiency_percent")?,
            })
        })?;

        match metrics_iter.next() {
            Some(Ok(metrics)) => Ok(Some(metrics)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    fn create_load_profile(&self, profile: &LoadProfile) -> Result<i64> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        conn.execute(
            r#"
            INSERT INTO load_profile (
                report_id, db_time_per_sec, cpu_time_per_sec, io_requests_per_sec,
                total_transactions, commits_per_sec, rollbacks_per_sec
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                profile.report_id,
                profile.db_time_per_sec,
                profile.cpu_time_per_sec,
                profile.io_requests_per_sec,
                profile.total_transactions,
                profile.commits_per_sec,
                profile.rollbacks_per_sec
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    fn get_load_profile(&self, report_id: i64) -> Result<Option<LoadProfile>> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let mut stmt = conn.prepare("SELECT * FROM load_profile WHERE report_id = ?")?;

        let mut profile_iter = stmt.query_map(params![report_id], |row| {
            Ok(LoadProfile {
                report_id: row.get("report_id")?,
                db_time_per_sec: row.get("db_time_per_sec")?,
                cpu_time_per_sec: row.get("cpu_time_per_sec")?,
                io_requests_per_sec: row.get("io_requests_per_sec")?,
                total_transactions: row.get("total_transactions")?,
                commits_per_sec: row.get("commits_per_sec")?,
                rollbacks_per_sec: row.get("rollbacks_per_sec")?,
            })
        })?;

        match profile_iter.next() {
            Some(Ok(profile)) => Ok(Some(profile)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    fn create_execution_plan(&self, plan: &crate::models::SqlExecutionPlan) -> Result<i64> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        // Serialize plan_tree to JSON
        let plan_json = serde_json::to_string(&plan.plan_tree)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        conn.execute(
            r#"
            INSERT INTO execution_plans (
                sql_id, plan_tree, created_at, source
            ) VALUES (?, ?, ?, ?)
            "#,
            params![plan.sql_id, plan_json, plan.created_at, plan.source],
        )?;

        Ok(conn.last_insert_rowid())
    }

    fn get_execution_plan_by_sql(
        &self,
        sql_id: i64,
    ) -> Result<Option<crate::models::SqlExecutionPlan>> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let mut stmt = conn.prepare("SELECT * FROM execution_plans WHERE sql_id = ?")?;

        let mut plan_iter = stmt.query_map(params![sql_id], |row| {
            let plan_tree_json: String = row.get("plan_tree")?;
            let plan_tree: crate::models::ExecutionPlanNode = serde_json::from_str(&plan_tree_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            Ok(crate::models::SqlExecutionPlan {
                id: row.get("id")?,
                sql_id: row.get("sql_id")?,
                plan_tree,
                created_at: row.get("created_at")?,
                source: row.get("source")?,
            })
        })?;

        match plan_iter.next() {
            Some(Ok(plan)) => Ok(Some(plan)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    fn get_execution_plans_by_report(
        &self,
        report_id: i64,
    ) -> Result<Vec<crate::models::SqlExecutionPlan>> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let mut stmt = conn.prepare(
            "SELECT ep.* FROM execution_plans ep
             JOIN top_sqls ts ON ep.sql_id = ts.id
             WHERE ts.report_id = ?",
        )?;

        let plan_iter = stmt.query_map(params![report_id], |row| {
            let plan_tree_json: String = row.get("plan_tree")?;
            let plan_tree: crate::models::ExecutionPlanNode = serde_json::from_str(&plan_tree_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            Ok(crate::models::SqlExecutionPlan {
                id: row.get("id")?,
                sql_id: row.get("sql_id")?,
                plan_tree,
                created_at: row.get("created_at")?,
                source: row.get("source")?,
            })
        })?;

        let mut plans = Vec::new();
        for plan in plan_iter {
            plans.push(plan?);
        }

        Ok(plans)
    }

    fn delete_execution_plan(&self, plan_id: i64) -> Result<()> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        conn.execute("DELETE FROM execution_plans WHERE id = ?", params![plan_id])?;

        Ok(())
    }

    fn create_comparison(
        &self,
        source_report_id: i64,
        target_report_id: i64,
        comparison_type: &str,
        summary: &crate::models::comparison::ComparisonSummary,
    ) -> Result<i64> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        // Serialize key_findings to JSON
        let findings_json = serde_json::to_string(&summary.key_findings)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        conn.execute(
            r#"
            INSERT INTO wdr_comparisons (
                source_report_id, target_report_id, comparison_type,
                performance_score_change, status, conclusion, key_findings
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                source_report_id,
                target_report_id,
                comparison_type,
                summary.performance_score_change,
                summary.status,
                summary.conclusion,
                findings_json
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    fn get_comparison_summary(
        &self,
        comparison_id: i64,
    ) -> Result<Option<crate::models::comparison::ComparisonSummary>> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let mut stmt = conn.prepare("SELECT * FROM wdr_comparisons WHERE id = ?")?;

        let mut summary_iter = stmt.query_map(params![comparison_id], |row| {
            let findings_json: String = row.get("key_findings")?;
            let key_findings: Vec<crate::models::comparison::KeyFinding> =
                serde_json::from_str(&findings_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            Ok(crate::models::comparison::ComparisonSummary {
                performance_score_change: row.get("performance_score_change")?,
                status: row.get("status")?,
                conclusion: row.get("conclusion")?,
                key_findings,
                created_at: row.get("created_at")?,
            })
        })?;

        match summary_iter.next() {
            Some(Ok(summary)) => Ok(Some(summary)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    fn get_comparisons(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
        sort_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> Result<Vec<crate::models::comparison::WdrComparisonListItem>> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let sort_column = match sort_by {
            Some("created_at") | None => "created_at",
            Some("performance_score") => "performance_score_change",
            Some("status") => "status",
            _ => "created_at",
        };

        let order = match sort_order {
            Some("ASC") => "ASC",
            _ => "DESC",
        };

        let limit_clause = match limit {
            Some(l) => format!("LIMIT {}", l),
            None => "".to_string(),
        };

        let offset_clause = match offset {
            Some(o) => format!("OFFSET {}", o),
            None => "".to_string(),
        };

        let query = format!(
            "SELECT c.*,
                    s.instance_name as source_instance,
                    t.instance_name as target_instance
             FROM wdr_comparisons c
             LEFT JOIN wdr_reports s ON c.source_report_id = s.id
             LEFT JOIN wdr_reports t ON c.target_report_id = t.id
             ORDER BY c.{} {} {} {}",
            sort_column, order, limit_clause, offset_clause
        );

        let mut stmt = conn.prepare(&query)?;

        let comparison_iter = stmt.query_map([], |row| {
            Ok(crate::models::comparison::WdrComparisonListItem {
                id: row.get("id")?,
                source_report_id: row.get("source_report_id")?,
                target_report_id: row.get("target_report_id")?,
                source_instance: row.get("source_instance")?,
                target_instance: row.get("target_instance")?,
                created_at: row.get("created_at")?,
                comparison_type: row.get("comparison_type")?,
                performance_score_change: row.get("performance_score_change")?,
                status: row.get("status")?,
            })
        })?;

        let mut comparisons = Vec::new();
        for comparison in comparison_iter {
            comparisons.push(comparison?);
        }

        Ok(comparisons)
    }

    fn count_comparisons(&self) -> Result<i64> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let mut stmt = conn.prepare("SELECT COUNT(*) as count FROM wdr_comparisons")?;
        let count = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    fn delete_comparison(&self, comparison_id: i64) -> Result<()> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        conn.execute(
            "DELETE FROM wdr_comparisons WHERE id = ?",
            params![comparison_id],
        )?;

        Ok(())
    }

    fn create_sql_comparison_metric(
        &self,
        comparison_id: i64,
        metric: &crate::models::comparison::SqlComparisonMetric,
    ) -> Result<i64> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        // Serialize metrics to JSON
        let source_metrics_json = serde_json::to_string(&metric.source_metrics)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let target_metrics_json = serde_json::to_string(&metric.target_metrics)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let changes_json = serde_json::to_string(&metric.change_percentages)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        conn.execute(
            r#"
            INSERT INTO sql_comparison_metrics (
                comparison_id, sql_id, sql_text_hash,
                source_metrics, target_metrics, change_percentages
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#,
            params![
                comparison_id,
                metric.sql_id,
                metric.sql_text_hash,
                source_metrics_json,
                target_metrics_json,
                changes_json
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    fn get_comparison_details(
        &self,
        comparison_id: i64,
        category: &str,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<crate::models::comparison::ComparisonDetails> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        let limit_clause = match limit {
            Some(l) => format!("LIMIT {}", l),
            None => "".to_string(),
        };

        let offset_clause = match offset {
            Some(o) => format!("OFFSET {}", o),
            None => "".to_string(),
        };

        let query = format!(
            "SELECT * FROM sql_comparison_metrics WHERE comparison_id = ? {} {}",
            limit_clause, offset_clause
        );

        let mut stmt = conn.prepare(&query)?;

        let metrics_iter = stmt.query_map(params![comparison_id], |row| {
            let source_json: String = row.get("source_metrics")?;
            let target_json: String = row.get("target_metrics")?;
            let changes_json: String = row.get("change_percentages")?;

            let source_metrics: crate::models::comparison::SqlMetrics =
                serde_json::from_str(&source_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            let target_metrics: crate::models::comparison::SqlMetrics =
                serde_json::from_str(&target_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            let change_percentages: crate::models::comparison::SqlChangePercentages =
                serde_json::from_str(&changes_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            let metric = crate::models::comparison::SqlComparisonMetric {
                sql_id: row.get("sql_id")?,
                sql_text_hash: row.get("sql_text_hash")?,
                source_metrics,
                target_metrics,
                change_percentages,
            };

            Ok(serde_json::to_value(metric)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?)
        })?;

        let mut metrics = Vec::new();
        for metric in metrics_iter {
            metrics.push(metric?);
        }

        // Get total count
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sql_comparison_metrics WHERE comparison_id = ?",
            params![comparison_id],
            |row| row.get(0),
        )?;

        Ok(crate::models::comparison::ComparisonDetails {
            comparison_id,
            category: category.to_string(),
            metrics,
            total_count: count,
        })
    }

    fn get_comparison_chart_data(
        &self,
        comparison_id: i64,
        chart_type: &str,
    ) -> Result<crate::models::comparison::ChartData> {
        let conn = self.get().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                    extended_code: 0,
                },
                Some(e.to_string()),
            )
        })?;

        // Get SQL metrics for the comparison
        let mut stmt =
            conn.prepare("SELECT * FROM sql_comparison_metrics WHERE comparison_id = ? LIMIT 20")?;

        let metrics_iter = stmt.query_map(params![comparison_id], |row| {
            let source_json: String = row.get("source_metrics")?;
            let target_json: String = row.get("target_metrics")?;

            let source_metrics: crate::models::comparison::SqlMetrics =
                serde_json::from_str(&source_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            let target_metrics: crate::models::comparison::SqlMetrics =
                serde_json::from_str(&target_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            Ok((source_metrics, target_metrics))
        })?;

        let mut labels = Vec::new();
        let mut source_data = Vec::new();
        let mut target_data = Vec::new();

        for (i, metric_result) in metrics_iter.enumerate() {
            let (source, target) = metric_result?;
            labels.push(format!("SQL {}", i + 1));
            source_data.push(source.total_elapsed_time);
            target_data.push(target.total_elapsed_time);
        }

        let dataset = crate::models::comparison::ChartDataset {
            label: "Elapsed Time".to_string(),
            source_data,
            target_data,
            color: Some("#4a90d9".to_string()),
        };

        Ok(crate::models::comparison::ChartData {
            comparison_id,
            chart_type: chart_type.to_string(),
            datasets: vec![dataset],
            labels,
        })
    }
}
