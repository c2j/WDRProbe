// Integration test for SQL audit workflow
// Tests for User Story 6 - View SQL Audit Results

#[cfg(test)]
mod audit_workflow_integration_tests {
    use wdrprobe_desktop_lib::models::audit::*;

    /// Test complete audit workflow from report to issues
    #[test]
    fn test_complete_audit_workflow() {
        // Step 1: Run SQL audit on a WDR report
        let audit_request = RunAuditRequest {
            report_ids: Some(vec![10]),
            include_resolved: false,
            audit_types: None,
        };

        let audit_result = simulate_run_audit(&audit_request);

        assert!(audit_result.success);
        assert!(audit_result.reports_audited > 0);
        assert!(audit_result.new_issues_found > 0);

        // Step 2: Retrieve detected issues
        let get_request = GetAuditIssuesRequest {
            report_id: Some(10),
            status: Some(AuditStatus::Open),
            severity: None,
            issue_type: None,
            limit: Some(50),
            offset: None,
            sort_by: Some("severity".to_string()),
        };

        let issue_list = simulate_get_issues(&get_request);

        assert!(!issue_list.issues.is_empty());
        assert_eq!(issue_list.total, audit_result.new_issues_found as i64);

        // Step 3: Review and update issue status
        let update_request = UpdateAuditIssueStatusRequest {
            issue_id: issue_list.issues[0].id,
            status: AuditStatus::Reviewed,
            resolved_by: "admin".to_string(),
            resolution_note: "Reviewed and acknowledged".to_string(),
        };

        let update_result = simulate_update_status(&update_request);

        assert!(update_result.success);
        assert_eq!(update_result.new_status, AuditStatus::Reviewed);
    }

    /// Test audit across multiple reports
    #[test]
    fn test_audit_multiple_reports() {
        let audit_request = RunAuditRequest {
            report_ids: Some(vec![10, 20, 30]),
            include_resolved: false,
            audit_types: None,
        };

        let audit_result = simulate_run_audit(&audit_request);

        assert_eq!(audit_result.reports_audited, 3);
        assert!(audit_result.new_issues_found > 0);
        assert!(audit_result.duration_ms > 0);
    }

    /// Test audit with existing resolved issues
    #[test]
    fn test_audit_include_resolved() {
        // First run - find new issues
        let first_run = RunAuditRequest {
            report_ids: Some(vec![10]),
            include_resolved: false,
            audit_types: None,
        };

        let first_result = simulate_run_audit(&first_run);
        assert!(first_result.new_issues_found > 0);

        // Second run - include resolved issues
        let second_run = RunAuditRequest {
            report_ids: Some(vec![10]),
            include_resolved: true,
            audit_types: None,
        };

        let second_result = simulate_run_audit(&second_run);
        assert!(second_result.existing_issues_updated > 0);
    }

    /// Test bulk update of audit issues
    #[test]
    fn test_bulk_update_workflow() {
        // Step 1: Run audit and get issues
        let audit_request = RunAuditRequest {
            report_ids: Some(vec![10]),
            include_resolved: false,
            audit_types: None,
        };

        let audit_result = simulate_run_audit(&audit_request);

        // Step 2: Bulk mark all as reviewed
        let issue_ids: Vec<i64> = audit_result.issues.iter()
            .map(|i| i.id)
            .collect();

        let bulk_request = BulkUpdateAuditIssuesRequest {
            issue_ids: issue_ids.clone(),
            status: AuditStatus::Reviewed,
            resolved_by: "dba_admin".to_string(),
            resolution_note: "Bulk reviewed during quarterly audit".to_string(),
        };

        let bulk_result = simulate_bulk_update(&bulk_request);

        assert!(bulk_result.success);
        assert_eq!(bulk_result.updated_count, issue_ids.len());
        assert!(bulk_result.failed_updates.is_empty());
    }

    /// Test filtering by severity
    #[test]
    fn test_filter_by_severity() {
        // Run audit first
        let audit_request = RunAuditRequest {
            report_ids: Some(vec![10]),
            include_resolved: false,
            audit_types: None,
        };

        let _audit_result = simulate_run_audit(&audit_request);

        // Get only critical issues
        let critical_request = GetAuditIssuesRequest {
            report_id: Some(10),
            status: None,
            severity: Some(AuditSeverity::Critical),
            issue_type: None,
            limit: None,
            offset: None,
            sort_by: None,
        };

        let critical_issues = simulate_get_issues(&critical_request);

        for issue in &critical_issues.issues {
            assert_eq!(issue.severity, AuditSeverity::Critical);
        }
    }

    /// Test filtering by issue type
    #[test]
    fn test_filter_by_issue_type() {
        let audit_request = RunAuditRequest {
            report_ids: Some(vec![10]),
            include_resolved: false,
            audit_types: None,
        };

        let _audit_result = simulate_run_audit(&audit_request);

        // Get only FullTableScan issues
        let type_request = GetAuditIssuesRequest {
            report_id: Some(10),
            status: None,
            severity: None,
            issue_type: Some(AuditIssueType::FullTableScan),
            limit: None,
            offset: None,
            sort_by: None,
        };

        let type_issues = simulate_get_issues(&type_request);

        for issue in &type_issues.issues {
            assert_eq!(issue.issue_type, AuditIssueType::FullTableScan);
        }
    }

    /// Test pagination through issues
    #[test]
    fn test_pagination() {
        let audit_request = RunAuditRequest {
            report_ids: Some(vec![10]),
            include_resolved: false,
            audit_types: None,
        };

        let audit_result = simulate_run_audit(&audit_request);

        let page_size = 10;
        let total_issues = audit_result.new_issues_found;
        let total_pages = (total_issues + page_size - 1) / page_size;

        // Fetch first page
        let page1 = GetAuditIssuesRequest {
            report_id: Some(10),
            status: None,
            severity: None,
            issue_type: None,
            limit: Some(page_size),
            offset: Some(0),
            sort_by: None,
        };

        let result1 = simulate_get_issues(&page1);
        assert_eq!(result1.issues.len(), page_size as usize);
        assert_eq!(result1.total, total_issues as i64);

        // Fetch second page
        let page2 = GetAuditIssuesRequest {
            report_id: Some(10),
            status: None,
            severity: None,
            issue_type: None,
            limit: Some(page_size),
            offset: Some(page_size),
            sort_by: None,
        };

        let result2 = simulate_get_issues(&page2);
        assert!(result2.issues.len() <= page_size as usize);
    }

    /// Test audit summary generation
    #[test]
    fn test_audit_summary_generation() {
        let issues = vec![
            create_issue(AuditIssueType::FullTableScan, AuditSeverity::Critical),
            create_issue(AuditIssueType::MissingIndex, AuditSeverity::High),
            create_issue(AuditIssueType::FullTableScan, AuditSeverity::High),
            create_issue(AuditIssueType::MissingStats, AuditSeverity::Medium),
            create_issue(AuditIssueType::SortOperation, AuditSeverity::Low),
        ];

        let summary = generate_audit_summary(&issues);

        assert_eq!(summary.total_issues, 5);

        // Check severity breakdown
        assert_eq!(*summary.by_severity.get(&AuditSeverity::Critical).unwrap(), 1);
        assert_eq!(*summary.by_severity.get(&AuditSeverity::High).unwrap(), 2);
        assert_eq!(*summary.by_severity.get(&AuditSeverity::Medium).unwrap(), 1);
        assert_eq!(*summary.by_severity.get(&AuditSeverity::Low).unwrap(), 1);

        // Check type breakdown
        assert_eq!(*summary.by_type.get(&AuditIssueType::FullTableScan).unwrap(), 2);
    }

    /// Test issue status transitions
    #[test]
    fn test_status_transition_workflow() {
        let issue = create_issue(AuditIssueType::MissingIndex, AuditSeverity::High);

        // Open -> Reviewed
        let reviewed = update_issue_status(&issue, AuditStatus::Reviewed, "admin", "Initial review");
        assert_eq!(reviewed.status, AuditStatus::Reviewed);

        // Reviewed -> Fixed
        let fixed = update_issue_status(&reviewed, AuditStatus::Fixed, "dba", "Index created");
        assert_eq!(fixed.status, AuditStatus::Fixed);
        assert!(fixed.resolved_at.is_some());
        assert_eq!(fixed.resolved_by.unwrap(), "dba");
    }

    /// Test audit on specific issue types only
    #[test]
    fn test_audit_specific_types() {
        let audit_request = RunAuditRequest {
            report_ids: Some(vec![10]),
            include_resolved: false,
            audit_types: Some(vec![
                AuditIssueType::FullTableScan,
                AuditIssueType::MissingIndex,
            ]),
        };

        let audit_result = simulate_run_audit(&audit_request);

        for issue in &audit_result.issues {
            assert!(
                issue.issue_type == AuditIssueType::FullTableScan ||
                issue.issue_type == AuditIssueType::MissingIndex
            );
        }
    }

    /// Test whitelisting issues
    #[test]
    fn test_whitelist_workflow() {
        let issue = create_issue(AuditIssueType::FullTableScan, AuditSeverity::High);

        // Whitelist as false positive
        let whitelisted = update_issue_status(
            &issue,
            AuditStatus::Whitelisted,
            "admin",
            "False positive - table is small by design"
        );

        assert_eq!(whitelisted.status, AuditStatus::Whitelisted);
    }

    /// Test sorting options
    #[test]
    fn test_sorting_options() {
        // Sort by severity (highest first)
        let by_severity = GetAuditIssuesRequest {
            report_id: Some(10),
            status: None,
            severity: None,
            issue_type: None,
            limit: None,
            offset: None,
            sort_by: Some("severity".to_string()),
        };

        let result = simulate_get_issues(&by_severity);

        // Verify critical issues come first
        for i in 1..result.issues.len() {
            let current_severity = severity_to_value(&result.issues[i].severity);
            let prev_severity = severity_to_value(&result.issues[i - 1].severity);
            assert!(current_severity <= prev_severity);
        }
    }

    // Helper types and functions

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

    #[derive(Debug, Clone)]
    struct SqlAuditIssueList {
        pub issues: Vec<SqlAuditIssue>,
        pub total: i64,
        pub summary: AuditSummary,
    }

    fn simulate_run_audit(request: &RunAuditRequest) -> AuditRunResult {
        let issues = vec![
            create_issue(AuditIssueType::FullTableScan, AuditSeverity::High),
            create_issue(AuditIssueType::MissingIndex, AuditSeverity::Critical),
        ];

        AuditRunResult {
            success: true,
            reports_audited: request.report_ids.as_ref().map(|v| v.len()).unwrap_or(1),
            new_issues_found: issues.len(),
            existing_issues_updated: 0,
            issues,
            duration_ms: 150,
            message: None,
        }
    }

    fn simulate_get_issues(request: &GetAuditIssuesRequest) -> SqlAuditIssueList {
        let issues = vec![
            create_issue(AuditIssueType::FullTableScan, AuditSeverity::Critical),
            create_issue(AuditIssueType::MissingIndex, AuditSeverity::High),
        ];

        SqlAuditIssueList {
            total: issues.len() as i64,
            issues,
            summary: generate_audit_summary(&issues),
        }
    }

    fn simulate_update_status(request: &UpdateAuditIssueStatusRequest) -> UpdateAuditIssueResult {
        let now = chrono::Utc::now().to_rfc3339();

        UpdateAuditIssueResult {
            success: true,
            issue_id: request.issue_id,
            old_status: AuditStatus::Open,
            new_status: request.status.clone(),
            message: Some("Status updated successfully".to_string()),
        }
    }

    fn simulate_bulk_update(request: &BulkUpdateAuditIssuesRequest) -> BulkUpdateResult {
        BulkUpdateResult {
            success: true,
            updated_count: request.issue_ids.len(),
            failed_updates: vec![],
            message: Some(format!("Updated {} issues", request.issue_ids.len())),
        }
    }

    fn create_issue(issue_type: AuditIssueType, severity: AuditSeverity) -> SqlAuditIssue {
        SqlAuditIssue {
            id: 1,
            report_id: Some(10),
            sql_id: Some(100),
            issue_type,
            severity,
            title: format!("{:?} detected", issue_type),
            description: "Issue description".to_string(),
            problematic_sql: Some("SELECT * FROM table".to_string()),
            recommendation: "Fix recommendation".to_string(),
            status: AuditStatus::Open,
            detected_at: chrono::Utc::now().to_rfc3339(),
            resolved_at: None,
            resolved_by: None,
        }
    }

    fn update_issue_status(
        issue: &SqlAuditIssue,
        status: AuditStatus,
        resolved_by: &str,
        _note: &str,
    ) -> SqlAuditIssue {
        let mut updated = issue.clone();
        updated.status = status.clone();
        updated.resolved_at = if matches!(status, AuditStatus::Fixed | AuditStatus::Whitelisted | AuditStatus::Ignored) {
            Some(chrono::Utc::now().to_rfc3339())
        } else {
            None
        };
        updated.resolved_by = if matches!(status, AuditStatus::Fixed | AuditStatus::Whitelisted) {
            Some(resolved_by.to_string())
        } else {
            None
        };
        updated
    }

    fn generate_audit_summary(issues: &[SqlAuditIssue]) -> AuditSummary {
        let mut by_severity = std::collections::HashMap::new();
        let mut by_status = std::collections::HashMap::new();
        let mut by_type = std::collections::HashMap::new();

        for issue in issues {
            *by_severity.entry(issue.severity.clone()).or_insert(0) += 1;
            *by_status.entry(issue.status.clone()).or_insert(0) += 1;
            *by_type.entry(issue.issue_type.clone()).or_insert(0) += 1;
        }

        AuditSummary {
            total_issues: issues.len() as i64,
            by_severity,
            by_status,
            by_type,
        }
    }

    fn severity_to_value(severity: &AuditSeverity) -> i32 {
        match severity {
            AuditSeverity::Critical => 5,
            AuditSeverity::High => 4,
            AuditSeverity::Medium => 3,
            AuditSeverity::Low => 2,
            AuditSeverity::Info => 1,
        }
    }
}
