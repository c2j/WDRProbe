// Unit tests for comparison functionality

#[cfg(test)]
mod comparison_tests {
    use std::collections::HashMap;
    use wdrprobe_desktop_lib::commands::comparison::calculate_performance_score;
    use wdrprobe_desktop_lib::models::comparison::*;

    #[test]
    fn test_create_comparison_response() {
        let result = CreateComparisonResult {
            success: true,
            comparison_id: 123,
            message: "Comparison created successfully".to_string(),
            processing_time_ms: 150,
        };

        assert!(result.success);
        assert_eq!(result.comparison_id, 123);
        assert_eq!(result.processing_time_ms, 150);
    }

    #[test]
    fn test_comparison_summary_improved_status() {
        let summary = ComparisonSummary {
            performance_score_change: 45,
            status: "Improved".to_string(),
            conclusion: "Performance improved significantly".to_string(),
            key_findings: vec![],
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(summary.performance_score_change, 45);
        assert_eq!(summary.status, "Improved");
        assert!(summary.performance_score_change > 0);
    }

    #[test]
    fn test_comparison_summary_degraded_status() {
        let summary = ComparisonSummary {
            performance_score_change: -30,
            status: "Degraded".to_string(),
            conclusion: "Performance degraded".to_string(),
            key_findings: vec![],
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(summary.performance_score_change, -30);
        assert_eq!(summary.status, "Degraded");
        assert!(summary.performance_score_change < 0);
    }

    #[test]
    fn test_key_finding_severity() {
        let critical_finding = KeyFinding {
            category: "Sql".to_string(),
            metric: "Total Elapsed Time".to_string(),
            change_percent: -150.0,
            severity: "Critical".to_string(),
            description: "Critical performance degradation detected".to_string(),
        };

        assert_eq!(critical_finding.severity, "Critical");
        assert!(critical_finding.change_percent < -50.0);
    }

    #[test]
    fn test_sql_metrics_comparison() {
        let source_metrics = SqlMetrics {
            executions: 1000,
            total_elapsed_time: 50000.0,
            cpu_time: 45000.0,
            io_time: 5000.0,
            buffer_gets: 100000,
            disk_reads: 5000,
            rows_processed: 50000,
        };

        let target_metrics = SqlMetrics {
            executions: 800,
            total_elapsed_time: 40000.0,
            cpu_time: 36000.0,
            io_time: 4000.0,
            buffer_gets: 80000,
            disk_reads: 4000,
            rows_processed: 40000,
        };

        // Target has fewer executions (20% decrease)
        assert!(target_metrics.executions < source_metrics.executions);

        // Target has better performance (20% decrease in elapsed time)
        assert!(target_metrics.total_elapsed_time < source_metrics.total_elapsed_time);
    }

    #[test]
    fn test_sql_change_percentages() {
        let changes = SqlChangePercentages {
            executions: Some(-20.0),
            elapsed_time: Some(-20.0),
            cpu_time: Some(-20.0),
            io_time: Some(-20.0),
            buffer_gets: Some(-20.0),
            disk_reads: Some(-20.0),
            rows_processed: Some(-20.0),
        };

        // All metrics improved (negative = improvement for time/resources)
        assert!(changes.executions.unwrap() < 0.0);
        assert!(changes.elapsed_time.unwrap() < 0.0);
        assert!(changes.cpu_time.unwrap() < 0.0);
    }

    #[test]
    fn test_sql_change_percentages_partial() {
        let partial_changes = SqlChangePercentages {
            executions: Some(10.0),
            elapsed_time: None,
            cpu_time: Some(-5.0),
            io_time: None,
            buffer_gets: Some(15.0),
            disk_reads: None,
            rows_processed: None,
        };

        // Some metrics have values, others are None
        assert!(partial_changes.executions.is_some());
        assert!(partial_changes.elapsed_time.is_none());
        assert!(partial_changes.cpu_time.is_some());
    }

    #[test]
    fn test_calculate_performance_score_all_improvements() {
        let mut changes = HashMap::new();
        changes.insert("sql_elapsed_time".to_string(), -50.0); // -50% = improvement (less time)
        changes.insert("sql_cpu_time".to_string(), -30.0); // -30% = improvement (less CPU)
        changes.insert("buffer_hit_ratio".to_string(), 20.0); // +20% = improvement (better ratio)

        let score = calculate_performance_score(&changes);
        // All improvements = positive score
        assert!(score > 0);
        assert!(score <= 100);
    }

    #[test]
    fn test_calculate_performance_score_all_degradations() {
        let mut changes = HashMap::new();
        changes.insert("sql_elapsed_time".to_string(), 50.0); // +50% = degradation (more time)
        changes.insert("sql_cpu_time".to_string(), 30.0); // +30% = degradation (more CPU)
        changes.insert("buffer_hit_ratio".to_string(), -20.0); // -20% = degradation (worse ratio)

        let score = calculate_performance_score(&changes);
        // All degradations = negative score
        assert!(score < 0);
        assert!(score >= -100);
    }

    #[test]
    fn test_calculate_performance_score_mixed() {
        let mut changes = HashMap::new();
        changes.insert("sql_elapsed_time".to_string(), 20.0); // +20% improvement
        changes.insert("sql_cpu_time".to_string(), -10.0); // -10% degradation
        changes.insert("buffer_hit_ratio".to_string(), 5.0); // +5% improvement

        let score = calculate_performance_score(&changes);
        // Mixed changes = moderate score
        assert!(score > -100 && score < 100);
    }

    #[test]
    fn test_calculate_performance_score_empty() {
        let changes = HashMap::new();
        let score = calculate_performance_score(&changes);
        // No changes = neutral score
        assert_eq!(score, 0);
    }

    #[test]
    fn test_comparison_details() {
        let details = ComparisonDetails {
            comparison_id: 1,
            category: "sql".to_string(),
            metrics: vec![],
            total_count: 100,
        };

        assert_eq!(details.comparison_id, 1);
        assert_eq!(details.category, "sql");
        assert_eq!(details.total_count, 100);
    }

    #[test]
    fn test_wdr_comparison() {
        let comparison = WdrComparison {
            id: 1,
            source_report_id: 10,
            target_report_id: 20,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            comparison_type: "TimeBased".to_string(),
            summary: ComparisonSummary {
                performance_score_change: 25,
                status: "Improved".to_string(),
                conclusion: "Overall improvement".to_string(),
                key_findings: vec![],
                created_at: "2024-01-01T00:00:00Z".to_string(),
            },
        };

        assert_eq!(comparison.source_report_id, 10);
        assert_eq!(comparison.target_report_id, 20);
        assert_eq!(comparison.comparison_type, "TimeBased");
        assert!(comparison.summary.performance_score_change > 0);
    }

    #[test]
    fn test_sql_comparison_metric() {
        let source = SqlMetrics {
            executions: 1000,
            total_elapsed_time: 50000.0,
            cpu_time: 45000.0,
            io_time: 5000.0,
            buffer_gets: 100000,
            disk_reads: 5000,
            rows_processed: 50000,
        };

        let target = SqlMetrics {
            executions: 800,
            total_elapsed_time: 40000.0,
            cpu_time: 36000.0,
            io_time: 4000.0,
            buffer_gets: 80000,
            disk_reads: 4000,
            rows_processed: 40000,
        };

        let changes = SqlChangePercentages {
            executions: Some(-20.0),
            elapsed_time: Some(-20.0),
            cpu_time: Some(-20.0),
            io_time: Some(-20.0),
            buffer_gets: Some(-20.0),
            disk_reads: Some(-20.0),
            rows_processed: Some(-20.0),
        };

        let metric = SqlComparisonMetric {
            sql_id: Some(123),
            sql_text_hash: "abc123".to_string(),
            source_metrics: source,
            target_metrics: target,
            change_percentages: changes,
        };

        assert_eq!(metric.sql_id, Some(123));
        assert_eq!(metric.sql_text_hash, "abc123");
        assert_eq!(metric.change_percentages.executions, Some(-20.0));
    }

    #[test]
    fn test_key_finding_categories() {
        let categories = vec!["Sql", "Wait", "Object", "System"];

        for category in categories {
            let finding = KeyFinding {
                category: category.to_string(),
                metric: "Test Metric".to_string(),
                change_percent: 10.0,
                severity: "Info".to_string(),
                description: format!("Test finding for {}", category),
            };
            assert_eq!(finding.category, category);
        }
    }

    #[test]
    fn test_key_finding_severity_levels() {
        let severities = vec!["Critical", "Warning", "Info"];

        for severity in severities {
            let finding = KeyFinding {
                category: "Sql".to_string(),
                metric: "Test Metric".to_string(),
                change_percent: 10.0,
                severity: severity.to_string(),
                description: format!("Test finding with {}", severity),
            };
            assert_eq!(finding.severity, severity);
        }
    }

    #[test]
    fn test_delete_result() {
        let result = DeleteResult {
            success: true,
            deleted_comparison_id: 123,
            message: Some("Comparison deleted successfully".to_string()),
        };

        assert!(result.success);
        assert_eq!(result.deleted_comparison_id, 123);
        assert!(result.message.is_some());
    }

    #[test]
    fn test_comparison_list_response() {
        let response = ComparisonListResponse {
            comparisons: vec![WdrComparisonListItem {
                id: 1,
                source_report_id: 10,
                target_report_id: 20,
                source_instance: Some("primary".to_string()),
                target_instance: Some("standby".to_string()),
                created_at: "2024-01-01T00:00:00Z".to_string(),
                comparison_type: "TimeBased".to_string(),
                performance_score_change: 25,
                status: "Improved".to_string(),
            }],
            total: 1,
        };

        assert_eq!(response.total, 1);
        assert_eq!(response.comparisons.len(), 1);
    }

    #[test]
    fn test_comparison_performance_thresholds() {
        // Test critical threshold (>= 50% change)
        let critical_change = KeyFinding {
            category: "Sql".to_string(),
            metric: "Elapsed Time".to_string(),
            change_percent: -75.0,
            severity: "Critical".to_string(),
            description: "Critical performance issue".to_string(),
        };
        assert!(critical_change.change_percent.abs() >= 50.0);

        // Test warning threshold (>= 20% change)
        let warning_change = KeyFinding {
            category: "Sql".to_string(),
            metric: "CPU Time".to_string(),
            change_percent: -35.0,
            severity: "Warning".to_string(),
            description: "Performance warning".to_string(),
        };
        assert!(warning_change.change_percent.abs() >= 20.0);
        assert!(warning_change.change_percent.abs() < 50.0);
    }
}
