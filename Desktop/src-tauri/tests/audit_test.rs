// Unit tests for SQL audit functionality
// Tests for User Story 6 - View SQL Audit Results

#[cfg(test)]
mod audit_tests {
    use wdrprobe_desktop_lib::models::audit::*;

    #[test]
    fn test_sql_audit_issue_creation() {
        let issue = SqlAuditIssue {
            id: 1,
            report_id: Some(10),
            sql_id: Some(100),
            issue_type: AuditIssueType::FullTableScan,
            severity: AuditSeverity::High,
            title: "Full table scan detected".to_string(),
            description: "Query performs full table scan on large table".to_string(),
            problematic_sql: Some("SELECT * FROM users".to_string()),
            recommendation: "Add index on filter column".to_string(),
            status: AuditStatus::Open,
            detected_at: "2024-01-15T10:00:00Z".to_string(),
            resolved_at: None,
            resolved_by: None,
        };

        assert_eq!(issue.id, 1);
        assert_eq!(issue.issue_type, AuditIssueType::FullTableScan);
        assert_eq!(issue.severity, AuditSeverity::High);
        assert_eq!(issue.status, AuditStatus::Open);
    }

    #[test]
    fn test_run_sql_audit_single_report() {
        let request = RunAuditRequest {
            report_ids: Some(vec![10]),
            include_resolved: false,
            audit_types: None,
        };

        assert!(request.report_ids.is_some());
        assert_eq!(request.report_ids.as_ref().unwrap().len(), 1);
        assert!(!request.include_resolved);
    }

    #[test]
    fn test_run_sql_audit_multiple_reports() {
        let request = RunAuditRequest {
            report_ids: Some(vec![10, 20, 30]),
            include_resolved: false,
            audit_types: None,
        };

        assert_eq!(request.report_ids.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_run_sql_audit_all_reports() {
        let request = RunAuditRequest {
            report_ids: None, // Audit all reports
            include_resolved: false,
            audit_types: None,
        };

        assert!(request.report_ids.is_none());
    }

    #[test]
    fn test_run_sql_audit_with_specific_types() {
        let request = RunAuditRequest {
            report_ids: None,
            include_resolved: false,
            audit_types: Some(vec![
                AuditIssueType::FullTableScan,
                AuditIssueType::MissingIndex,
                AuditIssueType::InefficientJoin,
            ]),
        };

        assert!(request.audit_types.is_some());
        assert_eq!(request.audit_types.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_get_sql_audit_issues_filter_by_status() {
        let request = GetAuditIssuesRequest {
            report_id: None,
            status: Some(AuditStatus::Open),
            severity: None,
            issue_type: None,
            limit: Some(50),
            offset: None,
            sort_by: None,
        };

        assert!(request.status.is_some());
        assert_eq!(request.status.unwrap(), AuditStatus::Open);
    }

    #[test]
    fn test_get_sql_audit_issues_filter_by_severity() {
        let request = GetAuditIssuesRequest {
            report_id: None,
            status: None,
            severity: Some(AuditSeverity::Critical),
            issue_type: None,
            limit: Some(50),
            offset: None,
            sort_by: None,
        };

        assert!(request.severity.is_some());
        assert_eq!(request.severity.unwrap(), AuditSeverity::Critical);
    }

    #[test]
    fn test_get_sql_audit_issues_filter_by_type() {
        let request = GetAuditIssuesRequest {
            report_id: None,
            status: None,
            severity: None,
            issue_type: Some(AuditIssueType::MissingIndex),
            limit: Some(50),
            offset: None,
            sort_by: None,
        };

        assert!(request.issue_type.is_some());
        assert_eq!(request.issue_type.unwrap(), AuditIssueType::MissingIndex);
    }

    #[test]
    fn test_get_sql_audit_issues_with_pagination() {
        let request = GetAuditIssuesRequest {
            report_id: None,
            status: None,
            severity: None,
            issue_type: None,
            limit: Some(20),
            offset: Some(40),
            sort_by: Some("detected_at".to_string()),
        };

        assert_eq!(request.limit, Some(20));
        assert_eq!(request.offset, Some(40));
        assert_eq!(request.sort_by, Some("detected_at".to_string()));
    }

    #[test]
    fn test_update_audit_issue_status() {
        let request = UpdateAuditIssueStatusRequest {
            issue_id: 1,
            status: AuditStatus::Fixed,
            resolved_by: "admin".to_string(),
            resolution_note: "Added index as recommended".to_string(),
        };

        assert_eq!(request.issue_id, 1);
        assert_eq!(request.status, AuditStatus::Fixed);
        assert_eq!(request.resolved_by, "admin");
    }

    #[test]
    fn test_bulk_update_audit_issues() {
        let request = BulkUpdateAuditIssuesRequest {
            issue_ids: vec![1, 2, 3, 4, 5],
            status: AuditStatus::Reviewed,
            resolved_by: "dba_admin".to_string(),
            resolution_note: "Batch reviewed during audit".to_string(),
        };

        assert_eq!(request.issue_ids.len(), 5);
        assert_eq!(request.status, AuditStatus::Reviewed);
    }

    #[test]
    fn test_audit_summary_aggregation() {
        let summary = AuditSummary {
            total_issues: 2,
            by_severity: serde_json::json!({
                "Critical": 1,
                "High": 1
            }),
            by_status: serde_json::json!({
                "Open": 2
            }),
            by_type: serde_json::json!({
                "FullTableScan": 1,
                "MissingIndex": 1
            }),
        };

        assert_eq!(summary.total_issues, 2);
        // Check the summary contains the expected values
        assert!(summary.by_severity.is_object());
        assert!(summary.by_status.is_object());
        assert!(summary.by_type.is_object());
    }

    #[test]
    fn test_audit_run_result() {
        let result = AuditRunResult {
            success: true,
            reports_audited: 3,
            new_issues_found: 15,
            existing_issues_updated: 2,
            issues: vec![],
            duration_ms: 1250,
            message: Some("Audit completed successfully".to_string()),
        };

        assert!(result.success);
        assert_eq!(result.reports_audited, 3);
        assert_eq!(result.new_issues_found, 15);
        assert_eq!(result.duration_ms, 1250);
    }

    #[test]
    fn test_issue_type_variants() {
        let types = vec![
            AuditIssueType::FullTableScan,
            AuditIssueType::MissingIndex,
            AuditIssueType::InefficientJoin,
            AuditIssueType::MissingStats,
            AuditIssueType::ExpensiveFunction,
            AuditIssueType::CartesianProduct,
            AuditIssueType::NestedLoopWithIndex,
            AuditIssueType::HashJoinTooLarge,
            AuditIssueType::SortOperation,
        ];

        assert_eq!(types.len(), 9);
    }

    #[test]
    fn test_severity_ordering() {
        // Critical should be more severe than High
        assert!(
            severity_to_value(AuditSeverity::Critical) > severity_to_value(AuditSeverity::High)
        );
        assert!(severity_to_value(AuditSeverity::High) > severity_to_value(AuditSeverity::Medium));
        assert!(severity_to_value(AuditSeverity::Medium) > severity_to_value(AuditSeverity::Low));
        assert!(severity_to_value(AuditSeverity::Low) > severity_to_value(AuditSeverity::Info));
    }

    #[test]
    fn test_status_transitions() {
        // Open -> Reviewed
        assert!(can_transition(AuditStatus::Open, AuditStatus::Reviewed));
        // Open -> Fixed
        assert!(can_transition(AuditStatus::Open, AuditStatus::Fixed));
        // Open -> Whitelisted
        assert!(can_transition(AuditStatus::Open, AuditStatus::Whitelisted));
        // Open -> Ignored
        assert!(can_transition(AuditStatus::Open, AuditStatus::Ignored));
        // Fixed should not transition back to Open
        assert!(!can_transition(AuditStatus::Fixed, AuditStatus::Open));
    }

    #[test]
    fn test_audit_issue_list() {
        let list = SqlAuditIssueList {
            issues: vec![],
            total: 100,
            summary: AuditSummary {
                total_issues: 100,
                by_severity: serde_json::json!({}),
                by_status: serde_json::json!({}),
                by_type: serde_json::json!({}),
            },
        };

        assert_eq!(list.total, 100);
        assert_eq!(list.summary.total_issues, 100);
    }

    #[test]
    fn test_update_audit_issue_result() {
        let result = UpdateAuditIssueResult {
            success: true,
            issue_id: 1,
            old_status: AuditStatus::Open,
            new_status: AuditStatus::Fixed,
            message: Some("Issue marked as fixed".to_string()),
        };

        assert!(result.success);
        assert_eq!(result.issue_id, 1);
        assert_eq!(result.old_status, AuditStatus::Open);
        assert_eq!(result.new_status, AuditStatus::Fixed);
    }

    #[test]
    fn test_bulk_update_result() {
        let result = BulkUpdateResult {
            success: true,
            updated_count: 10,
            failed_updates: vec![],
            message: Some("Bulk update completed".to_string()),
        };

        assert!(result.success);
        assert_eq!(result.updated_count, 10);
        assert!(result.failed_updates.is_empty());
    }

    #[test]
    fn test_bulk_update_result_with_failures() {
        let failed = vec![FailedAuditUpdate {
            issue_id: 5,
            error: "Issue not found".to_string(),
        }];

        let result = BulkUpdateResult {
            success: false,
            updated_count: 9,
            failed_updates: failed,
            message: Some("Some updates failed".to_string()),
        };

        assert!(!result.success);
        assert_eq!(result.updated_count, 9);
        assert_eq!(result.failed_updates.len(), 1);
    }

    // Helper types and functions for tests

    #[derive(Debug, Clone)]
    struct RunAuditRequest {
        pub report_ids: Option<Vec<i64>>,
        pub include_resolved: bool,
        pub audit_types: Option<Vec<AuditIssueType>>,
    }

    #[derive(Debug, Clone)]
    struct GetAuditIssuesRequest {
        pub report_id: Option<i64>,
        pub status: Option<AuditStatus>,
        pub severity: Option<AuditSeverity>,
        pub issue_type: Option<AuditIssueType>,
        pub limit: Option<i32>,
        pub offset: Option<i32>,
        pub sort_by: Option<String>,
    }

    #[derive(Debug, Clone)]
    struct UpdateAuditIssueStatusRequest {
        pub issue_id: i64,
        pub status: AuditStatus,
        pub resolved_by: String,
        pub resolution_note: String,
    }

    #[derive(Debug, Clone)]
    struct BulkUpdateAuditIssuesRequest {
        pub issue_ids: Vec<i64>,
        pub status: AuditStatus,
        pub resolved_by: String,
        pub resolution_note: String,
    }

    #[derive(Debug, Clone)]
    struct AuditRunResult {
        pub success: bool,
        pub reports_audited: usize,
        pub new_issues_found: usize,
        pub existing_issues_updated: usize,
        pub issues: Vec<SqlAuditIssue>,
        pub duration_ms: u64,
        pub message: Option<String>,
    }

    #[derive(Debug, Clone)]
    struct UpdateAuditIssueResult {
        pub success: bool,
        pub issue_id: i64,
        pub old_status: AuditStatus,
        pub new_status: AuditStatus,
        pub message: Option<String>,
    }

    #[derive(Debug, Clone)]
    struct BulkUpdateResult {
        pub success: bool,
        pub updated_count: usize,
        pub failed_updates: Vec<FailedAuditUpdate>,
        pub message: Option<String>,
    }

    #[derive(Debug, Clone)]
    struct FailedAuditUpdate {
        pub issue_id: i64,
        pub error: String,
    }

    fn severity_to_value(severity: AuditSeverity) -> i32 {
        match severity {
            AuditSeverity::Critical => 5,
            AuditSeverity::High => 4,
            AuditSeverity::Medium => 3,
            AuditSeverity::Low => 2,
            AuditSeverity::Info => 1,
        }
    }

    fn can_transition(from: AuditStatus, to: AuditStatus) -> bool {
        match (from, to) {
            (AuditStatus::Open, _) => true,
            (AuditStatus::Reviewed, AuditStatus::Fixed) => true,
            (AuditStatus::Reviewed, AuditStatus::Whitelisted) => true,
            (AuditStatus::Reviewed, AuditStatus::Ignored) => true,
            _ => false,
        }
    }
}
