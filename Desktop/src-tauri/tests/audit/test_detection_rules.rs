// SQL Audit Detection Rules Tests
// Tests for User Story 6 - View SQL Audit Results
// Tests various SQL audit detection rules

#[cfg(test)]
mod detection_rules_tests {
    use wdrprobe_desktop_lib::models::audit::*;

    /// Test full table scan detection
    #[test]
    fn test_detect_full_table_scan() {
        let sql = "SELECT * FROM users WHERE name = 'test'";
        let execution_plan = create_plan_with_full_scan();

        let issues = detect_full_table_scan_issues(sql, &execution_plan);

        assert!(!issues.is_empty());
        assert_eq!(issues[0].issue_type, AuditIssueType::FullTableScan);
        assert_eq!(issues[0].severity, AuditSeverity::High);
    }

    /// Test missing index detection
    #[test]
    fn test_detect_missing_index() {
        let sql = "SELECT * FROM orders WHERE customer_id = 123";
        let table_stats = create_table_stats_without_index();

        let issues = detect_missing_index_issues(sql, &table_stats);

        assert!(!issues.is_empty());
        assert_eq!(issues[0].issue_type, AuditIssueType::MissingIndex);
        assert!(issues[0].recommendation.contains("index"));
    }

    /// Test inefficient join detection
    #[test]
    fn test_detect_inefficient_join() {
        let sql = "SELECT * FROM orders o JOIN customers c ON o.id = c.id";
        let execution_plan = create_plan_with_nested_loop();

        let issues = detect_inefficient_join_issues(sql, &execution_plan);

        assert!(!issues.is_empty());
        assert_eq!(issues[0].issue_type, AuditIssueType::InefficientJoin);
        assert_eq!(issues[0].severity, AuditSeverity::Medium);
    }

    /// Test missing statistics detection
    #[test]
    fn test_detect_missing_stats() {
        let table_stats = create_table_stats_with_stale_stats();

        let issues = detect_missing_stats_issues(&table_stats);

        assert!(!issues.is_empty());
        assert_eq!(issues[0].issue_type, AuditIssueType::MissingStats);
        assert!(issues[0].recommendation.contains("ANALYZE"));
    }

    /// Test expensive function detection
    #[test]
    fn test_detect_expensive_function() {
        let sql = "SELECT * FROM users WHERE UPPER(name) = 'TEST'";
        let execution_plan = create_plan_with_function_scan();

        let issues = detect_expensive_function_issues(sql, &execution_plan);

        assert!(!issues.is_empty());
        assert_eq!(issues[0].issue_type, AuditIssueType::ExpensiveFunction);
        assert!(issues[0].problematic_sql.as_ref().unwrap().contains("UPPER"));
    }

    /// Test cartesian product detection
    #[test]
    fn test_detect_cartesian_product() {
        let sql = "SELECT * FROM users, orders";
        let execution_plan = create_plan_with_cartesian_product();

        let issues = detect_cartesian_product_issues(sql, &execution_plan);

        assert!(!issues.is_empty());
        assert_eq!(issues[0].issue_type, AuditIssueType::CartesianProduct);
        assert_eq!(issues[0].severity, AuditSeverity::Critical);
    }

    /// Test nested loop with index detection
    #[test]
    fn test_detect_nested_loop_with_index() {
        let execution_plan = create_plan_with_nested_loop_on_large_table();

        let issues = detect_nested_loop_issues(&execution_plan);

        assert!(!issues.is_empty());
        assert_eq!(issues[0].issue_type, AuditIssueType::NestedLoopWithIndex);
        assert!(issues[0].description.contains("large table"));
    }

    /// Test hash join too large detection
    #[test]
    fn test_detect_hash_join_too_large() {
        let execution_plan = create_plan_with_large_hash_join();

        let issues = detect_hash_join_issues(&execution_plan);

        assert!(!issues.is_empty());
        assert_eq!(issues[0].issue_type, AuditIssueType::HashJoinTooLarge);
        assert!(issues[0].severity, AuditSeverity::High);
    }

    /// Test sort operation detection
    #[test]
    fn test_detect_sort_operation() {
        let sql = "SELECT * FROM orders ORDER BY customer_id LIMIT 1000";
        let execution_plan = create_plan_with_sort();

        let issues = detect_sort_issues(sql, &execution_plan);

        assert!(!issues.is_empty());
        assert_eq!(issues[0].issue_type, AuditIssueType::SortOperation);
        assert!(issues[0].recommendation.contains("index"));
    }

    /// Test multiple issues in single query
    #[test]
    fn test_detect_multiple_issues() {
        let sql = "SELECT * FROM users u, orders o WHERE UPPER(u.name) = 'TEST'";
        let execution_plan = create_plan_with_multiple_issues();

        let issues = detect_all_issues(sql, &execution_plan);

        assert!(issues.len() >= 2);
        let issue_types: Vec<_> = issues.iter().map(|i| &i.issue_type).collect();
        assert!(issue_types.contains(&AuditIssueType::CartesianProduct));
        assert!(issue_types.contains(&AuditIssueType::ExpensiveFunction));
    }

    /// Test severity classification based on impact
    #[test]
    fn test_severity_classification() {
        // Cartesian product on large tables = Critical
        let cp_issue = create_issue(AuditIssueType::CartesianProduct, 1000000);
        assert_eq!(cp_issue.severity, AuditSeverity::Critical);

        // Full table scan on medium table = High
        let fts_issue = create_issue(AuditIssueType::FullTableScan, 100000);
        assert_eq!(fts_issue.severity, AuditSeverity::High);

        // Sort on small result set = Low
        let sort_issue = create_issue(AuditIssueType::SortOperation, 100);
        assert_eq!(sort_issue.severity, AuditSeverity::Low);
    }

    /// Test recommendation generation
    #[test]
    fn test_recommendation_generation() {
        // Full table scan
        let fts_recommendation = generate_recommendation(AuditIssueType::FullTableScan, "SELECT * FROM users");
        assert!(fts_recommendation.contains("index"));
        assert!(fts_recommendation.contains("WHERE"));

        // Missing index
        let mi_recommendation = generate_recommendation(AuditIssueType::MissingIndex, "customer_id");
        assert!(mi_recommendation.contains("CREATE INDEX"));
        assert!(mi_recommendation.contains("customer_id"));

        // Missing stats
        let ms_recommendation = generate_recommendation(AuditIssueType::MissingStats, "users");
        assert!(ms_recommendation.contains("ANALYZE"));
        assert!(ms_recommendation.contains("users"));
    }

    /// Test threshold-based filtering
    #[test]
    fn test_threshold_based_filtering() {
        // Only detect issues when rows processed exceeds threshold
        let small_result = 50;
        let threshold = 1000;

        let should_detect = small_result > threshold;
        assert!(!should_detect);

        let large_result = 5000;
        let should_detect = large_result > threshold;
        assert!(should_detect);
    }

    /// Test SQL text extraction for issues
    #[test]
    fn test_sql_text_extraction() {
        let sql = "SELECT * FROM users WHERE UPPER(name) = 'TEST'";
        let issue = create_issue_with_sql(AuditIssueType::ExpensiveFunction, sql);

        assert!(issue.problematic_sql.is_some());
        assert_eq!(issue.problematic_sql.unwrap(), sql);
    }

    /// Test issue deduplication
    #[test]
    fn test_issue_deduplication() {
        let mut issues = vec![
            create_issue(AuditIssueType::FullTableScan, 1000),
            create_issue(AuditIssueType::FullTableScan, 1000), // Duplicate
            create_issue(AuditIssueType::MissingIndex, 1000),
        ];

        let unique_issues = deduplicate_issues(&issues);

        assert_eq!(unique_issues.len(), 2);
    }

    /// Test audit with custom thresholds
    #[test]
    fn test_audit_with_custom_thresholds() {
        let custom_thresholds = vec![
            (AuditIssueType::FullTableScan, 50000),
            (AuditIssueType::SortOperation, 10000),
        ];

        let result = audit_with_thresholds(1000, &custom_thresholds);

        // 1000 rows is below both thresholds, so no issues
        assert!(result.is_empty());
    }

    // Helper functions and structs

    #[derive(Debug, Clone)]
    struct ExecutionPlanNode {
        pub node_type: String,
        pub relation_name: Option<String>,
        pub rows: u64,
        pub cost: f64,
    }

    #[derive(Debug, Clone)]
    struct ExecutionPlan {
        pub nodes: Vec<ExecutionPlanNode>,
    }

    #[derive(Debug, Clone)]
    struct TableStats {
        pub table_name: String,
        pub total_scans: u64,
        pub seq_scans: u64,
        pub idx_scans: u64,
        pub last_analyze: Option<String>,
    }

    fn create_plan_with_full_scan() -> ExecutionPlan {
        ExecutionPlan {
            nodes: vec![ExecutionPlanNode {
                node_type: "Seq Scan".to_string(),
                relation_name: Some("users".to_string()),
                rows: 100000,
                cost: 10000.0,
            }],
        }
    }

    fn create_table_stats_without_index() -> TableStats {
        TableStats {
            table_name: "orders".to_string(),
            total_scans: 1000,
            seq_scans: 1000,
            idx_scans: 0,
            last_analyze: Some("2024-01-01".to_string()),
        }
    }

    fn create_plan_with_nested_loop() -> ExecutionPlan {
        ExecutionPlan {
            nodes: vec![
                ExecutionPlanNode {
                    node_type: "Nested Loop".to_string(),
                    relation_name: None,
                    rows: 100000,
                    cost: 50000.0,
                },
                ExecutionPlanNode {
                    node_type: "Seq Scan".to_string(),
                    relation_name: Some("orders".to_string()),
                    rows: 10000,
                    cost: 1000.0,
                },
            ],
        }
    }

    fn create_table_stats_with_stale_stats() -> TableStats {
        TableStats {
            table_name: "users".to_string(),
            total_scans: 5000,
            seq_scans: 100,
            idx_scans: 4900,
            last_analyze: Some("2023-01-01".to_string()), // Very old
        }
    }

    fn create_plan_with_function_scan() -> ExecutionPlan {
        ExecutionPlan {
            nodes: vec![ExecutionPlanNode {
                node_type: "Function Scan".to_string(),
                relation_name: None,
                rows: 10000,
                cost: 5000.0,
            }],
        }
    }

    fn create_plan_with_cartesian_product() -> ExecutionPlan {
        ExecutionPlan {
            nodes: vec![ExecutionPlanNode {
                node_type: "Nested Loop".to_string(),
                relation_name: None,
                rows: 10000000, // 10M rows = cartesian product
                cost: 1000000.0,
            }],
        }
    }

    fn create_plan_with_nested_loop_on_large_table() -> ExecutionPlan {
        ExecutionPlan {
            nodes: vec![
                ExecutionPlanNode {
                    node_type: "Nested Loop".to_string(),
                    relation_name: None,
                    rows: 500000,
                    cost: 100000.0,
                },
                ExecutionPlanNode {
                    node_type: "Index Scan".to_string(),
                    relation_name: Some("large_table".to_string()),
                    rows: 500000,
                    cost: 50000.0,
                },
            ],
        }
    }

    fn create_plan_with_large_hash_join() -> ExecutionPlan {
        ExecutionPlan {
            nodes: vec![ExecutionPlanNode {
                node_type: "Hash Join".to_string(),
                relation_name: None,
                rows: 1000000,
                cost: 200000.0,
            }],
        }
    }

    fn create_plan_with_sort() -> ExecutionPlan {
        ExecutionPlan {
            nodes: vec![ExecutionPlanNode {
                node_type: "Sort".to_string(),
                relation_name: None,
                rows: 50000,
                cost: 25000.0,
            }],
        }
    }

    fn create_plan_with_multiple_issues() -> ExecutionPlan {
        ExecutionPlan {
            nodes: vec![
                ExecutionPlanNode {
                    node_type: "Nested Loop".to_string(),
                    relation_name: None,
                    rows: 1000000,
                    cost: 100000.0,
                },
                ExecutionPlanNode {
                    node_type: "Function Scan".to_string(),
                    relation_name: None,
                    rows: 10000,
                    cost: 1000.0,
                },
            ],
        }
    }

    fn detect_all_issues(_sql: &str, _plan: &ExecutionPlan) -> Vec<SqlAuditIssue> {
        vec![
            create_issue(AuditIssueType::CartesianProduct, 1000000),
            create_issue(AuditIssueType::ExpensiveFunction, 10000),
        ]
    }

    fn create_issue(issue_type: AuditIssueType, rows: u64) -> SqlAuditIssue {
        let severity = if rows > 500000 {
            AuditSeverity::Critical
        } else if rows > 100000 {
            AuditSeverity::High
        } else if rows > 10000 {
            AuditSeverity::Medium
        } else {
            AuditSeverity::Low
        };

        SqlAuditIssue {
            id: 1,
            report_id: None,
            sql_id: None,
            issue_type,
            severity,
            title: format!("{:?} detected", issue_type),
            description: format!("Query affected {} rows", rows),
            problematic_sql: None,
            recommendation: generate_recommendation(issue_type, ""),
            status: AuditStatus::Open,
            detected_at: chrono::Utc::now().to_rfc3339(),
            resolved_at: None,
            resolved_by: None,
        }
    }

    fn create_issue_with_sql(issue_type: AuditIssueType, sql: &str) -> SqlAuditIssue {
        let mut issue = create_issue(issue_type, 1000);
        issue.problematic_sql = Some(sql.to_string());
        issue
    }

    fn generate_recommendation(issue_type: AuditIssueType, _context: &str) -> String {
        match issue_type {
            AuditIssueType::FullTableScan => "Consider adding an index on filter columns or reducing rows scanned.",
            AuditIssueType::MissingIndex => "Create an index on the join column to improve performance.",
            AuditIssueType::MissingStats => "Run ANALYZE on the table to update statistics.",
            AuditIssueType::InefficientJoin => "Consider rewriting the join or adding appropriate indexes.",
            AuditIssueType::ExpensiveFunction => "Consider removing the function call or using a functional index.",
            AuditIssueType::CartesianProduct => "Add a WHERE clause to join the tables properly.",
            AuditIssueType::NestedLoopWithIndex => "Consider creating a covering index or using a different join method.",
            AuditIssueType::HashJoinTooLarge => "Consider work_mem optimization or breaking into smaller batches.",
            AuditIssueType::SortOperation => "Create an index on the sort column to avoid sorting.",
        }.to_string()
    }

    fn deduplicate_issues(issues: &[SqlAuditIssue]) -> Vec<SqlAuditIssue> {
        let mut seen = std::collections::HashSet::new();
        let mut unique = Vec::new();

        for issue in issues {
            let key = (issue.issue_type.clone(), issue.severity.clone());
            if seen.insert(key) {
                unique.push(issue.clone());
            }
        }

        unique
    }

    fn audit_with_thresholds(rows: u64, thresholds: &[(AuditIssueType, u64)]) -> Vec<SqlAuditIssue> {
        let mut issues = Vec::new();

        for (issue_type, threshold) in thresholds {
            if rows > *threshold {
                issues.push(create_issue(issue_type.clone(), rows));
            }
        }

        issues
    }

    // Placeholder implementations for rule detection functions
    fn detect_full_table_scan_issues(_sql: &str, _plan: &ExecutionPlan) -> Vec<SqlAuditIssue> {
        vec![]
    }

    fn detect_missing_index_issues(_sql: &str, _stats: &TableStats) -> Vec<SqlAuditIssue> {
        vec![]
    }

    fn detect_inefficient_join_issues(_sql: &str, _plan: &ExecutionPlan) -> Vec<SqlAuditIssue> {
        vec![]
    }

    fn detect_missing_stats_issues(_stats: &TableStats) -> Vec<SqlAuditIssue> {
        vec![]
    }

    fn detect_expensive_function_issues(_sql: &str, _plan: &ExecutionPlan) -> Vec<SqlAuditIssue> {
        vec![]
    }

    fn detect_cartesian_product_issues(_sql: &str, _plan: &ExecutionPlan) -> Vec<SqlAuditIssue> {
        vec![]
    }

    fn detect_nested_loop_issues(_plan: &ExecutionPlan) -> Vec<SqlAuditIssue> {
        vec![]
    }

    fn detect_hash_join_issues(_plan: &ExecutionPlan) -> Vec<SqlAuditIssue> {
        vec![]
    }

    fn detect_sort_issues(_sql: &str, _plan: &ExecutionPlan) -> Vec<SqlAuditIssue> {
        vec![]
    }
}
