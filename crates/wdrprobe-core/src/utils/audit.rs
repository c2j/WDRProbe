// Audit logging utilities
// Per Constitution Principle IX - Audit trail for all operations

use crate::database::{DatabaseOperations, DatabasePool};
use crate::models::AuditLog;

pub struct AuditLogger {
    pool: DatabasePool,
}

impl AuditLogger {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Log an action to the audit trail
    pub fn log_action(
        &self,
        action: &str,
        entity_type: &str,
        entity_id: Option<i64>,
        old_value: Option<&str>,
        new_value: Option<&str>,
        user_id: Option<&str>,
        success: bool,
        error_message: Option<&str>,
        details: Option<&str>,
    ) -> Result<i64, String> {
        let log = AuditLog {
            id: 0, // Will be auto-generated
            timestamp: chrono::Utc::now().to_rfc3339(),
            user_id: user_id.map(|s| s.to_string()),
            action: action.to_string(),
            entity_type: entity_type.to_string(),
            entity_id,
            old_value: old_value.map(|s| s.to_string()),
            new_value: new_value.map(|s| s.to_string()),
            ip_address: None, // Desktop app doesn't have IP
            success,
            error_message: error_message.map(|s| s.to_string()),
            details: details.map(|s| s.to_string()),
        };

        DatabaseOperations::create_audit_log(&self.pool, &log)
            .map_err(|e| format!("Failed to create audit log: {}", e))
    }

    /// Convenience method for threshold updates
    pub fn log_threshold_update(
        &self,
        _config_key: &str,
        old_value: f64,
        new_value: f64,
        changed_by: &str,
        change_reason: &str,
    ) -> Result<i64, String> {
        self.log_action(
            "ThresholdUpdate",
            "threshold",
            None,
            Some(&old_value.to_string()),
            Some(&new_value.to_string()),
            Some(changed_by),
            true,
            None,
            Some(change_reason),
        )
    }

    /// Convenience method for report operations
    pub fn log_report_operation(
        &self,
        operation: &str, // "Import", "Delete", etc.
        report_id: i64,
        success: bool,
        error_message: Option<&str>,
    ) -> Result<i64, String> {
        self.log_action(
            operation,
            "wdr_report",
            Some(report_id),
            None,
            None,
            None, // No user in desktop app
            success,
            error_message,
            None,
        )
    }

    /// Convenience method for audit issue status changes
    pub fn log_audit_issue_update(
        &self,
        issue_id: i64,
        old_status: &str,
        new_status: &str,
        resolved_by: &str,
    ) -> Result<i64, String> {
        self.log_action(
            "AuditIssueUpdate",
            "sql_audit_issue",
            Some(issue_id),
            Some(old_status),
            Some(new_status),
            Some(resolved_by),
            true,
            None,
            None,
        )
    }
}
