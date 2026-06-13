// Dashboard data models
// Contains models for dashboard display

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanceSummary {
    pub instance_name: String,
    pub status: InstanceStatus,
    pub health_score: i32,
    pub active_issues: i32,
    pub report_count: u64,
    pub last_report_time: Option<DateTime<Utc>>,
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
    pub instance_name: Option<String>,
    pub cpu: String, // Formatted as percentage string
    pub mem: String, // Formatted as percentage string
    pub tps: String, // Formatted with units
    pub qps: String, // Formatted with units
    pub health_distribution: Vec<HealthDistributionItem>,
    pub trend_data: Vec<TrendDataPoint>,
    pub hot_issues: Vec<HotIssue>,
    // Backend-only fields for calculations
    #[serde(skip)]
    pub cpu_usage_percent: f64,
    #[serde(skip)]
    pub memory_usage_percent: f64,
    #[serde(skip)]
    pub tps_raw: f64,
    #[serde(skip)]
    pub qps_raw: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HealthDistributionItem {
    pub name: String,
    pub value: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrendDataPoint {
    pub time: String,
    pub value: u32,
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
pub enum AuditSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FindingCategory {
    Sql,
    Wait,
    Object,
    System,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ReportStatus {
    SuccessfullyImported,
    ImportFailed(String),
    PartiallyImported,
}
