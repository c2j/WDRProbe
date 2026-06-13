// WDR Report data models
// Contains models for WDR reports and related data

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
pub struct EfficiencyMetrics {
    pub report_id: i64,
    pub buffer_hit_percent: f64,
    pub cpu_efficiency_percent: f64,
    pub soft_parse_rate_percent: f64,
    pub hard_parse_rate_percent: f64,
    pub execution_efficiency_percent: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoadProfile {
    pub report_id: i64,
    pub db_time_per_sec: f64,
    pub cpu_time_per_sec: f64,
    pub io_requests_per_sec: f64,
    pub total_transactions: u64,
    pub commits_per_sec: f64,
    pub rollbacks_per_sec: f64,
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
    pub first_load_time: String,
    pub last_load_time: String,
    pub is_hot_sql: bool,
    pub rank_by_time: Option<i32>,
}

impl TopSql {
    /// Calculate hash of SQL text for matching comparisons
    /// Uses simple hex encoding of normalized SQL text
    pub fn sql_text_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Normalize SQL: remove extra whitespace and convert to uppercase for comparison
        let normalized = self
            .sql_text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_uppercase();

        let mut hasher = DefaultHasher::new();
        normalized.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

// Database statistics for each database in the report
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseStats {
    pub id: i64,
    pub report_id: i64,
    pub db_name: String,
    pub backends: u64,
    pub xact_commit: u64,
    pub xact_rollback: u64,
    pub blks_read: u64,
    pub blks_hit: u64,
    pub tuple_returned: u64,
    pub tuple_fetched: u64,
    pub tuple_inserted: u64,
    pub tuple_updated: u64,
    pub tuple_deleted: u64,
    pub conflicts: u64,
    pub temp_files: u64,
    pub temp_bytes: u64,
    pub deadlocks: u64,
    pub blk_read_time: f64,
    pub blk_write_time: f64,
    pub stats_reset: Option<String>,
}

// Cache IO statistics for tables and indexes
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheIoStats {
    pub id: i64,
    pub report_id: i64,
    pub schema_name: String,
    pub object_name: String,
    pub object_type: String, // table or index
    pub heap_blks_read: u64,
    pub heap_blks_hit: u64,
    pub heap_blks_hit_ratio: f64,
    pub idx_blks_read: u64,
    pub idx_blks_hit: u64,
    pub idx_blks_hit_ratio: f64,
    pub toast_blks_read: u64,
    pub toast_blks_hit: u64,
    pub toast_blks_hit_ratio: f64,
    pub tidx_blks_read: u64,
    pub tidx_blks_hit: u64,
    pub tidx_blks_hit_ratio: f64,
}

// Object statistics (similar to existing but enhanced)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObjectStats {
    pub id: i64,
    pub report_id: i64,
    pub schema_name: String,
    pub object_name: String,
    pub object_type: String,
    pub total_scans: u64,
    pub seq_scans: u64,
    pub idx_scans: u64,
    pub seq_reads: u64,
    pub idx_reads: u64,
    pub inserts: u64,
    pub updates: u64,
    pub deletes: u64,
    pub dead_tuples: u64,
    pub needs_vacuum: bool,
}

// Complete WDR report with all sections
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompleteWdrReport {
    pub report: WdrReport,
    pub efficiency: EfficiencyMetrics,
    pub load_profile: LoadProfile,
    pub database_stats: Vec<DatabaseStats>,
    pub top_sql: Vec<TopSql>,
    pub cache_io_stats: Vec<CacheIoStats>,
    pub object_stats: Vec<ObjectStats>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WdrReportDetail {
    pub id: i64,
    pub instance_name: String,
    pub generation_time: String,
    pub snapshot_start: String,
    pub snapshot_end: String,
    pub status: String,
    pub efficiency: EfficiencyMetrics,
    pub load_profile: LoadProfile,
    pub top_sql: Vec<TopSql>,
    pub object_stats: Vec<ObjectStats>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WdrReportListResponse {
    pub reports: Vec<WdrReport>,
    pub total: i64,
}
