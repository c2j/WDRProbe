// Export and Import data models
// Per Constitution Principle IX - Audit trail for all operations

use serde::{Deserialize, Serialize};

/// Export format options
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Csv,
    Pdf,
}

/// Import data type options
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ImportType {
    Reports,
    Comparisons,
    Thresholds,
    AuditIssues,
}

/// Data integrity check types
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum IntegrityCheckType {
    Checksum,
    RecordCount,
    SchemaValidation,
}

/// Entity types for integrity checks
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum EntityType {
    WdrReport,
    TopSql,
    Comparison,
    Threshold,
    AuditIssue,
}

/// Request to export a WDR report
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportWdrReportRequest {
    pub report_id: i64,
    pub format: ExportFormat,
    pub include_sql_details: bool,
    pub include_comparison_data: bool,
    pub export_path: Option<String>,
}

/// Result of an export operation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportResult {
    pub success: bool,
    pub export_path: String,
    pub record_count: usize,
    pub file_size: u64,
    pub format: ExportFormat,
    pub message: Option<String>,
}

/// Request to import data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportDataRequest {
    pub import_path: String,
    pub validate_only: bool,
    pub overwrite_existing: bool,
    pub import_types: Vec<ImportType>,
}

/// Result of an import operation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportResult {
    pub success: bool,
    pub records_imported: usize,
    pub records_skipped: usize,
    pub records_failed: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub validation_errors: Vec<String>,
    pub message: Option<String>,
}

/// Data integrity check result
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataIntegrityCheck {
    pub check_type: IntegrityCheckType,
    pub entity_type: EntityType,
    pub entity_id: Option<i64>,
    pub expected_hash: Option<String>,
    pub actual_hash: Option<String>,
    pub passed: bool,
    pub message: Option<String>,
}

/// Batch export request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchExportRequest {
    pub report_ids: Vec<i64>,
    pub format: ExportFormat,
    pub combine: bool,
    pub export_directory: String,
}

/// Batch export result
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchExportResult {
    pub success: bool,
    exports_completed: usize,
    exports_failed: usize,
    export_results: Vec<ExportResult>,
    message: Option<String>,
}

/// CSV export data for WDR report
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WdrReportCsvData {
    pub id: i64,
    pub instance_name: String,
    pub generation_time: String,
    pub sql_count: usize,
    pub total_elapsed_time: f64,
    pub status: String,
}

/// PDF export metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PdfExportMetadata {
    pub title: String,
    pub author: String,
    pub subject: String,
    pub keywords: Vec<String>,
    pub creation_date: String,
    pub report_id: i64,
}

/// Export file manifest
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportManifest {
    pub version: String,
    pub export_date: String,
    pub export_type: String,
    pub item_count: usize,
    pub checksums: Vec<String>,
    pub metadata: serde_json::Value,
}
