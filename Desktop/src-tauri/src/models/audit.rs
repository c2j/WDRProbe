// SQL audit and logging data models
// Per Constitution Principle IX - Audit trail for all operations

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum AuditIssueType {
    FullTableScan,
    MissingIndex,
    InefficientJoin,
    MissingStats,
    ExpensiveFunction,
    CartesianProduct,
    NestedLoopWithIndex,
    HashJoinTooLarge,
    SortOperation,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum AuditSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum AuditStatus {
    Open,
    Reviewed,
    Whitelisted,
    Fixed,
    Ignored,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqlAuditIssue {
    pub id: i64,
    pub report_id: Option<i64>,
    pub sql_id: Option<i64>,
    pub issue_type: AuditIssueType,
    pub severity: AuditSeverity,
    pub title: String,
    pub description: String,
    pub problematic_sql: Option<String>,
    pub recommendation: String,
    pub status: AuditStatus,
    pub detected_at: String,
    pub resolved_at: Option<String>,
    pub resolved_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditLog {
    pub id: i64,
    pub timestamp: String,
    pub user_id: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<i64>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub ip_address: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqlAuditIssueList {
    pub issues: Vec<SqlAuditIssue>,
    pub total: i64,
    pub summary: AuditSummary,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditSummary {
    pub total_issues: i64,
    pub by_severity: serde_json::Value,
    pub by_status: serde_json::Value,
    pub by_type: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateAuditIssueResult {
    pub success: bool,
    pub issue_id: i64,
    pub old_status: AuditStatus,
    pub new_status: AuditStatus,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BulkUpdateResult {
    pub success: bool,
    pub updated_count: usize,
    pub failed_updates: Vec<FailedAuditUpdate>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FailedAuditUpdate {
    pub issue_id: i64,
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditLogList {
    pub logs: Vec<AuditLog>,
    pub total: i64,
}
