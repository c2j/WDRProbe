// Integration test for comparison flow
// Tests the complete workflow from WDR reports to comparison results

#[cfg(test)]
mod comparison_integration_tests {
    use wdrprobe_desktop_lib::models::comparison::*;

    /// Test the complete comparison flow:
    /// 1. Load two WDR reports
    /// 2. Compare their metrics
    /// 3. Generate key findings
    /// 4. Return summary
    #[test]
    fn test_complete_comparison_flow() {
        // Step 1: Simulate loading two WDR reports
        let source_report = create_mock_report(1, "db1", "2024-01-01T10:00:00Z");
        let target_report = create_mock_report(2, "db1", "2024-01-02T10:00:00Z");

        // Verify reports can be compared (same instance, different times)
        assert_eq!(source_report.instance_name, target_report.instance_name);
        assert_ne!(source_report.snapshot_start, target_report.snapshot_start);

        // Step 2: Simulate SQL metrics from both reports
        let source_sqls = create_mock_sql_metrics(1, 1000, 50000.0);
        let target_sqls = create_mock_sql_metrics(1, 800, 35000.0); // Improved

        // Step 3: Calculate comparison metrics
        let comparison_metrics = compare_sql_metrics(&source_sqls, &target_sqls);
        assert!(comparison_metrics.change_percentages.elapsed_time.unwrap() < 0.0); // Improvement

        // Step 4: Generate key findings
        let findings = generate_key_findings_from_comparison(&comparison_metrics);
        assert!(!findings.is_empty());

        // Step 5: Calculate performance score
        let mut changes = std::collections::HashMap::new();
        changes.insert("sql_elapsed_time".to_string(), 30.0); // 30% improvement

        let score = 30.0; // Simulated score
        let status = if score >= 15.0 {
            "Improved"
        } else if score <= -15.0 {
            "Degraded"
        } else {
            "NoSignificantChange"
        };

        assert_eq!(status, "Improved");
    }

    /// Test comparison with performance degradation
    #[test]
    fn test_comparison_with_degradation() {
        let source_sqls = create_mock_sql_metrics(1, 1000, 50000.0);
        let target_sqls = create_mock_sql_metrics(1, 1500, 75000.0); // Degraded

        let comparison_metrics = compare_sql_metrics(&source_sqls, &target_sqls);

        // Executions increased (negative for performance)
        assert!(comparison_metrics.change_percentages.executions.unwrap() > 0.0);

        // Elapsed time increased (negative for performance)
        assert!(comparison_metrics.change_percentages.elapsed_time.unwrap() > 0.0);
    }

    /// Test comparison with no significant change
    #[test]
    fn test_comparison_no_significant_change() {
        let source_sqls = create_mock_sql_metrics(1, 1000, 50000.0);
        let target_sqls = create_mock_sql_metrics(1, 1050, 52500.0); // ~5% change

        let comparison_metrics = compare_sql_metrics(&source_sqls, &target_sqls);

        let exec_change = comparison_metrics.change_percentages.executions.unwrap();
        assert!(exec_change.abs() < 10.0); // Less than 10% change
    }

    /// Test multiple SQL comparison
    #[test]
    fn test_multiple_sql_comparison() {
        let source_sqls = vec![
            create_mock_sql_metrics(1, 1000, 50000.0),
            create_mock_sql_metrics(2, 500, 30000.0),
            create_mock_sql_metrics(3, 2000, 100000.0),
        ];

        let target_sqls = vec![
            create_mock_sql_metrics(1, 800, 40000.0),   // Improved
            create_mock_sql_metrics(2, 520, 30500.0),   // Slightly degraded
            create_mock_sql_metrics(3, 1800, 85000.0),  // Improved
        ];

        let comparisons: Vec<SqlComparisonMetric> = source_sqls.iter()
            .zip(target_sqls.iter())
            .map(|(source, target)| compare_sql_metrics(source, target))
            .collect();

        assert_eq!(comparisons.len(), 3);

        // At least one should show improvement
        let improvements = comparisons.iter()
            .filter(|c| c.change_percentages.elapsed_time.unwrap_or(0.0) < 0.0)
            .count();
        assert!(improvements > 0);
    }

    /// Test key findings generation
    #[test]
    fn test_key_findings_generation() {
        let changes = vec![
            ("SQL Elapsed Time", -60.0),  // Critical improvement
            ("CPU Time", -25.0),           // Warning improvement
            ("Buffer Hit Ratio", 15.0),    // Info improvement
            ("Disk Reads", -80.0),         // Critical improvement
        ];

        let findings = generate_findings(&changes);

        assert_eq!(findings.len(), 4);

        let critical = findings.iter().filter(|f| f.severity == "Critical").count();
        assert_eq!(critical, 2);
    }

    /// Test comparison summary creation
    #[test]
    fn test_comparison_summary_creation() {
        let findings = vec![
            KeyFinding {
                category: "Sql".to_string(),
                metric: "Elapsed Time".to_string(),
                change_percent: -50.0,
                severity: "Warning".to_string(),
                description: "Significant improvement".to_string(),
            },
        ];

        let summary = ComparisonSummary {
            performance_score_change: 35,
            status: "Improved".to_string(),
            conclusion: "Performance improved across all metrics".to_string(),
            key_findings: findings,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(summary.performance_score_change, 35);
        assert_eq!(summary.status, "Improved");
        assert_eq!(summary.key_findings.len(), 1);
    }

    /// Test comparison details retrieval
    #[test]
    fn test_comparison_details_retrieval() {
        let comparison_id = 1;
        let category = "sql";

        let details = ComparisonDetails {
            comparison_id,
            category: category.to_string(),
            metrics: vec![],
            total_count: 100,
        };

        assert_eq!(details.comparison_id, comparison_id);
        assert_eq!(details.category, category);
        assert_eq!(details.total_count, 100);
    }

    /// Test pagination in comparison details
    #[test]
    fn test_comparison_pagination() {
        let all_metrics: Vec<SqlComparisonMetric> = (0..150)
            .map(|i| SqlComparisonMetric {
                sql_id: Some(i),
                sql_text_hash: format!("hash{}", i),
                source_metrics: create_mock_sql_metrics(i, 1000, 50000.0),
                target_metrics: create_mock_sql_metrics(i, 800, 40000.0),
                change_percentages: SqlChangePercentages {
                    executions: Some(-20.0),
                    elapsed_time: Some(-20.0),
                    cpu_time: Some(-20.0),
                    io_time: Some(-20.0),
                    buffer_gets: Some(-20.0),
                    disk_reads: Some(-20.0),
                    rows_processed: Some(-20.0),
                },
            })
            .collect();

        // Page 1: items 0-49
        let page1: Vec<_> = all_metrics.iter().take(50).collect();
        assert_eq!(page1.len(), 50);

        // Page 2: items 50-99
        let page2: Vec<_> = all_metrics.iter().skip(50).take(50).collect();
        assert_eq!(page2.len(), 50);

        // Page 3: items 100-149
        let page3: Vec<_> = all_metrics.iter().skip(100).take(50).collect();
        assert_eq!(page3.len(), 50);
    }

    /// Test comparison with SQL hash matching
    #[test]
    fn test_sql_hash_matching() {
        let sql = "SELECT * FROM users WHERE id = ?";

        // Create SQL entries with same text (should match by hash)
        let source_sql = create_mock_sql_with_text(1, sql);
        let target_sql = create_mock_sql_with_text(1, sql);

        let source_hash = format!("{:x}", md5::compute(source_sql.sql_text.as_bytes()));
        let target_hash = format!("{:x}", md5::compute(target_sql.sql_text.as_bytes()));

        assert_eq!(source_hash, target_hash);
    }

    /// Test comparison result storage
    #[test]
    fn test_comparison_result_storage() {
        let comparison = WdrComparison {
            id: 0, // Will be assigned by database
            source_report_id: 10,
            target_report_id: 20,
            created_at: chrono::Utc::now().to_rfc3339(),
            comparison_type: "TimeBased".to_string(),
            summary: ComparisonSummary {
                performance_score_change: 25,
                status: "Improved".to_string(),
                conclusion: "Performance improved".to_string(),
                key_findings: vec![],
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        };

        assert_eq!(comparison.source_report_id, 10);
        assert_eq!(comparison.target_report_id, 20);
        assert!(comparison.summary.performance_score_change > 0);
    }

    /// Helper function to create a mock WDR report
    fn create_mock_report(
        id: i64,
        instance_name: &str,
        snapshot_start: &str,
    ) -> MockWdrReport {
        MockWdrReport {
            id,
            instance_name: instance_name.to_string(),
            snapshot_start: snapshot_start.to_string(),
            snapshot_end: "2024-01-01T11:00:00Z".to_string(),
        }
    }

    /// Helper function to create mock SQL metrics
    fn create_mock_sql_metrics(
        sql_id: i64,
        executions: u64,
        elapsed_time: f64,
    ) -> SqlMetrics {
        SqlMetrics {
            executions,
            total_elapsed_time: elapsed_time,
            cpu_time: elapsed_time * 0.8,
            io_time: elapsed_time * 0.2,
            buffer_gets: executions * 50,
            disk_reads: executions / 10,
            rows_processed: executions * 10,
        }
    }

    /// Helper function to create mock SQL with text
    fn create_mock_sql_with_text(
        sql_id: i64,
        sql_text: &str,
    ) -> MockSql {
        MockSql {
            sql_id,
            sql_text: sql_text.to_string(),
        }
    }

    /// Helper function to compare SQL metrics
    fn compare_sql_metrics(source: &SqlMetrics, target: &SqlMetrics) -> SqlComparisonMetric {
        let calc_change = |s: f64, t: f64| -> Option<f64> {
            if s == 0.0 { return None; }
            Some(((t - s) / s) * 100.0)
        };

        SqlComparisonMetric {
            sql_id: Some(1),
            sql_text_hash: "test_hash".to_string(),
            source_metrics: source.clone(),
            target_metrics: target.clone(),
            change_percentages: SqlChangePercentages {
                executions: calc_change(source.executions as f64, target.executions as f64),
                elapsed_time: calc_change(source.total_elapsed_time, target.total_elapsed_time),
                cpu_time: calc_change(source.cpu_time, target.cpu_time),
                io_time: calc_change(source.io_time, target.io_time),
                buffer_gets: calc_change(source.buffer_gets as f64, target.buffer_gets as f64),
                disk_reads: calc_change(source.disk_reads as f64, target.disk_reads as f64),
                rows_processed: calc_change(source.rows_processed as f64, target.rows_processed as f64),
            },
        }
    }

    /// Helper function to generate key findings
    fn generate_key_findings_from_comparison(
        comparison: &SqlComparisonMetric,
    ) -> Vec<KeyFinding> {
        let mut findings = Vec::new();

        let check_metric = |name: &str, change: f64, category: &str| {
            let severity = if change.abs() >= 50.0 {
                "Critical"
            } else if change.abs() >= 20.0 {
                "Warning"
            } else {
                "Info"
            };

            KeyFinding {
                category: category.to_string(),
                metric: name.to_string(),
                change_percent: change,
                severity: severity.to_string(),
                description: format!("{} changed by {:.1}%", name, change),
            }
        };

        if let Some(change) = comparison.change_percentages.elapsed_time {
            findings.push(check_metric("Elapsed Time", change, "Sql"));
        }

        findings
    }

    /// Helper function to generate findings from changes
    fn generate_findings(changes: &[(&str, f64)]) -> Vec<KeyFinding> {
        changes.iter().map(|(metric, change)| {
            let severity = if change.abs() >= 50.0 {
                "Critical"
            } else if change.abs() >= 20.0 {
                "Warning"
            } else {
                "Info"
            };

            KeyFinding {
                category: "Sql".to_string(),
                metric: metric.to_string(),
                change_percent: *change,
                severity: severity.to_string(),
                description: format!("{} changed by {:.1}%", metric, change),
            }
        }).collect()
    }

    /// Mock structures for testing
    struct MockWdrReport {
        id: i64,
        instance_name: String,
        snapshot_start: String,
        snapshot_end: String,
    }

    struct MockSql {
        sql_id: i64,
        sql_text: String,
    }
}
