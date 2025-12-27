// Comparison data models
// Contains models for WDR report comparison

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WdrComparison {
    pub id: i64,
    pub source_report_id: i64,
    pub target_report_id: i64,
    pub created_at: String,
    pub comparison_type: String,
    pub summary: ComparisonSummary,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComparisonSummary {
    pub performance_score_change: i32,
    pub status: String,
    pub conclusion: String,
    pub key_findings: Vec<KeyFinding>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyFinding {
    pub category: String,
    pub metric: String,
    pub change_percent: f64,
    pub severity: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqlComparisonMetric {
    pub sql_id: Option<i64>,
    pub sql_text_hash: String,
    pub source_metrics: SqlMetrics,
    pub target_metrics: SqlMetrics,
    pub change_percentages: SqlChangePercentages,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComparisonDetails {
    pub comparison_id: i64,
    pub category: String,
    pub metrics: Vec<serde_json::Value>,
    pub total_count: i64,
}

// Additional types for commands

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComparisonListResponse {
    pub comparisons: Vec<WdrComparisonListItem>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WdrComparisonListItem {
    pub id: i64,
    pub source_report_id: i64,
    pub target_report_id: i64,
    pub source_instance: Option<String>,
    pub target_instance: Option<String>,
    pub created_at: String,
    pub comparison_type: String,
    pub performance_score_change: i32,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateComparisonRequest {
    pub source_report_id: i64,
    pub target_report_id: i64,
    pub comparison_type: Option<String>,
    pub custom_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateComparisonResult {
    pub success: bool,
    pub comparison_id: i64,
    pub message: String,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetComparisonsParams {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetComparisonDetailsParams {
    pub comparison_id: i64,
    pub category: String,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteComparisonRequest {
    pub comparison_id: i64,
    pub confirm: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteResult {
    pub success: bool,
    pub deleted_comparison_id: i64,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComparisonMetric {
    pub metric_name: String,
    pub source_value: f64,
    pub target_value: f64,
    pub change_percent: f64,
    pub trend: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChartData {
    pub comparison_id: i64,
    pub chart_type: String,
    pub datasets: Vec<ChartDataset>,
    pub labels: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChartDataset {
    pub label: String,
    pub source_data: Vec<f64>,
    pub target_data: Vec<f64>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BarChartData {
    pub labels: Vec<String>,
    pub source_values: Vec<f64>,
    pub target_values: Vec<f64>,
    pub change_percentages: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LineChartData {
    pub time_points: Vec<String>,
    pub source_series: Vec<f64>,
    pub target_series: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScatterPoint {
    pub x: f64,
    pub y: f64,
    pub label: String,
    pub sql_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScatterChartData {
    pub points: Vec<ScatterPoint>,
}
