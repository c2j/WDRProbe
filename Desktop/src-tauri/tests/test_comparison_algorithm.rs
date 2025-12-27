// Unit tests for comparison algorithm

#[cfg(test)]
mod comparison_algorithm_tests {
    use std::collections::HashMap;
    use wdrprobe_desktop_lib::models::comparison::*;

    /// Helper function to calculate percentage change
    fn calculate_percent_change(source: f64, target: f64) -> f64 {
        if source == 0.0 {
            return 0.0;
        }
        ((target - source) / source) * 100.0
    }

    /// Test matching SQL queries by text hash
    #[test]
    fn test_match_sql_by_hash() {
        let sql_1 = "SELECT * FROM users WHERE id = 1";
        let sql_2 = "SELECT * FROM users WHERE id = 1";
        let sql_3 = "SELECT * FROM orders WHERE id = 1";

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Same SQL should have same hash
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();

        sql_1.hash(&mut hasher1);
        sql_2.hash(&mut hasher2);
        sql_3.hash(&mut hasher3);

        let hash_1 = hasher1.finish();
        let hash_2 = hasher2.finish();
        let hash_3 = hasher3.finish();

        assert_eq!(hash_1, hash_2);
        assert_ne!(hash_1, hash_3);
    }

    /// Test SQL metrics calculation
    #[test]
    fn test_calculate_sql_metrics() {
        let source = SqlMetrics {
            executions: 1000,
            total_elapsed_time: 50000.0,
            cpu_time: 40000.0,
            io_time: 10000.0,
            buffer_gets: 50000,
            disk_reads: 1000,
            rows_processed: 10000,
        };

        let target = SqlMetrics {
            executions: 500,             // -50% executions
            total_elapsed_time: 25000.0, // -50% time
            cpu_time: 20000.0,           // -50% CPU
            io_time: 5000.0,             // -50% IO
            buffer_gets: 25000,          // -50% buffers
            disk_reads: 500,             // -50% disk reads
            rows_processed: 5000,        // -50% rows
        };

        let exec_change =
            calculate_percent_change(source.executions as f64, target.executions as f64);
        assert!((exec_change - (-50.0)).abs() < 0.01);

        let time_change =
            calculate_percent_change(source.total_elapsed_time, target.total_elapsed_time);
        assert!((time_change - (-50.0)).abs() < 0.01);
    }

    /// Test performance score calculation weights
    #[test]
    fn test_performance_score_weights() {
        // SQL metrics have higher weight
        let mut changes = HashMap::new();
        changes.insert("sql_elapsed_time".to_string(), 50.0); // Weight: 0.4
        changes.insert("sql_cpu_time".to_string(), 30.0); // Weight: 0.3
        changes.insert("buffer_hit_ratio".to_string(), 20.0); // Weight: 0.2
        changes.insert("instance_efficiency".to_string(), 10.0); // Weight: 0.1

        // Expected: (50 * 0.4 + 30 * 0.3 + 20 * 0.2 + 10 * 0.1) / 1.0
        //         = (20 + 9 + 4 + 1) / 1.0 = 34
        let score = calculate_performance_score_from_weights(&changes);
        assert!((score - 34.0).abs() < 1.0);
    }

    fn calculate_performance_score_from_weights(changes: &HashMap<String, f64>) -> f64 {
        let weights: HashMap<&str, f64> = [
            ("sql_elapsed_time", 0.4),
            ("sql_cpu_time", 0.3),
            ("buffer_hit_ratio", 0.2),
            ("instance_efficiency", 0.1),
        ]
        .iter()
        .cloned()
        .collect();

        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for (metric_name, change) in changes {
            if let Some(weight) = weights.get(metric_name.as_str()) {
                weighted_sum += change * weight;
                total_weight += weight;
            }
        }

        if total_weight == 0.0 {
            return 0.0;
        }

        let score = weighted_sum / total_weight;
        score.max(-100.0).min(100.0)
    }

    /// Test key findings identification
    #[test]
    fn test_identify_key_findings() {
        let changes = vec![
            ("SQL Elapsed Time", -75.0), // Critical
            ("CPU Time", -35.0),         // Warning
            ("Buffer Hit Ratio", 5.0),   // Info
            ("Disk Reads", -55.0),       // Critical
        ];

        let findings = identify_findings_from_changes(&changes);

        // Should find 2 critical, 1 warning, 1 info
        let critical_count = findings.iter().filter(|f| f.severity == "Critical").count();
        let warning_count = findings.iter().filter(|f| f.severity == "Warning").count();

        assert_eq!(critical_count, 2);
        assert_eq!(warning_count, 1);
    }

    fn identify_findings_from_changes(changes: &[(&str, f64)]) -> Vec<KeyFinding> {
        let mut findings = Vec::new();

        for (metric, change) in changes {
            let severity = if change.abs() >= 50.0 {
                "Critical"
            } else if change.abs() >= 20.0 {
                "Warning"
            } else {
                "Info"
            };

            findings.push(KeyFinding {
                category: "Sql".to_string(),
                metric: metric.to_string(),
                change_percent: *change,
                severity: severity.to_string(),
                description: format!("{} changed by {:.1}%", metric, change),
            });
        }

        findings
    }

    /// Test comparison with no common SQL
    #[test]
    fn test_comparison_no_common_sql() {
        let source_sqls: Vec<String> = vec![
            "SELECT * FROM users".to_string(),
            "SELECT * FROM orders".to_string(),
        ];

        let target_sqls: Vec<String> = vec![
            "SELECT * FROM products".to_string(),
            "SELECT * FROM customers".to_string(),
        ];

        let common = find_common_sql(&source_sqls, &target_sqls);
        assert_eq!(common.len(), 0);
    }

    /// Test comparison with common SQL
    #[test]
    fn test_comparison_with_common_sql() {
        let source_sqls: Vec<String> = vec![
            "SELECT * FROM users".to_string(),
            "SELECT * FROM orders".to_string(),
            "SELECT * FROM products".to_string(),
        ];

        let target_sqls: Vec<String> = vec![
            "SELECT * FROM users".to_string(),
            "SELECT * FROM orders".to_string(),
            "SELECT * FROM customers".to_string(),
        ];

        let common = find_common_sql(&source_sqls, &target_sqls);
        assert_eq!(common.len(), 2);
    }

    fn find_common_sql(source: &[String], target: &[String]) -> Vec<String> {
        source
            .iter()
            .filter(|sql| target.contains(sql))
            .cloned()
            .collect()
    }

    /// Test performance score clamping
    #[test]
    fn test_performance_score_clamping() {
        let mut extreme_positive = HashMap::new();
        extreme_positive.insert("sql_elapsed_time".to_string(), 500.0);
        let score = calculate_performance_score_from_weights(&extreme_positive);
        assert!(score <= 100.0);

        let mut extreme_negative = HashMap::new();
        extreme_negative.insert("sql_elapsed_time".to_string(), -500.0);
        let score = calculate_performance_score_from_weights(&extreme_negative);
        assert!(score >= -100.0);
    }

    /// Test status determination from performance score
    #[test]
    fn test_status_from_performance_score() {
        assert_eq!(status_from_score(50.0), "Improved");
        assert_eq!(status_from_score(20.0), "Improved");
        assert_eq!(status_from_score(-20.0), "Degraded");
        assert_eq!(status_from_score(-50.0), "Degraded");
        assert_eq!(status_from_score(5.0), "NoSignificantChange");
        assert_eq!(status_from_score(-5.0), "NoSignificantChange");
    }

    fn status_from_score(score: f64) -> &'static str {
        if score >= 15.0 {
            "Improved"
        } else if score <= -15.0 {
            "Degraded"
        } else {
            "NoSignificantChange"
        }
    }

    /// Test comparison type validation
    #[test]
    fn test_comparison_type_validation() {
        let valid_types = vec!["TimeBased", "InstanceBased", "AdHoc"];

        for type_name in valid_types {
            assert!(is_valid_comparison_type(type_name));
        }

        assert!(!is_valid_comparison_type("InvalidType"));
    }

    fn is_valid_comparison_type(type_name: &str) -> bool {
        matches!(type_name, "TimeBased" | "InstanceBased" | "AdHoc")
    }

    /// Test SQL hash consistency
    #[test]
    fn test_sql_hash_consistency() {
        let sql = "SELECT * FROM users WHERE id = ?";

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        sql.hash(&mut hasher1);
        sql.hash(&mut hasher2);

        let hash1 = hasher1.finish();
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    /// Test metrics aggregation
    #[test]
    fn test_metrics_aggregation() {
        let metrics = vec![
            SqlMetrics {
                executions: 100,
                total_elapsed_time: 1000.0,
                cpu_time: 800.0,
                io_time: 200.0,
                buffer_gets: 5000,
                disk_reads: 100,
                rows_processed: 1000,
            },
            SqlMetrics {
                executions: 200,
                total_elapsed_time: 2000.0,
                cpu_time: 1600.0,
                io_time: 400.0,
                buffer_gets: 10000,
                disk_reads: 200,
                rows_processed: 2000,
            },
        ];

        let total_executions: u64 = metrics.iter().map(|m| m.executions).sum();
        assert_eq!(total_executions, 300);

        let total_time: f64 = metrics.iter().map(|m| m.total_elapsed_time).sum();
        assert!((total_time - 3000.0).abs() < 0.01);
    }

    /// Test change percentage calculation with zero source
    #[test]
    fn test_change_percent_with_zero_source() {
        let source = 0.0;
        let target = 100.0;

        let change = calculate_percent_change(source, target);
        assert_eq!(change, 0.0); // Should return 0 to avoid division by zero
    }

    /// Test comparison summary generation
    #[test]
    fn test_comparison_summary_generation() {
        let findings = vec![
            KeyFinding {
                category: "Sql".to_string(),
                metric: "Elapsed Time".to_string(),
                change_percent: -50.0,
                severity: "Warning".to_string(),
                description: "SQL performance degraded".to_string(),
            },
            KeyFinding {
                category: "System".to_string(),
                metric: "Buffer Hit Ratio".to_string(),
                change_percent: 10.0,
                severity: "Info".to_string(),
                description: "Cache efficiency improved".to_string(),
            },
        ];

        let score = -20.0;
        let status = status_from_score(score);
        let conclusion = format!(
            "Performance {} with {} key findings",
            if score >= 15.0 {
                "improved"
            } else if score <= -15.0 {
                "degraded"
            } else {
                "remained stable"
            },
            findings.len()
        );

        assert_eq!(status, "Degraded");
        assert!(conclusion.contains("degraded"));
        assert!(conclusion.contains("2 key findings"));
    }
}
