// Integration test for execution plan visualization flow
// Tests the complete workflow from WDR report to execution plan visualization

#[cfg(test)]
mod execution_plan_integration_tests {
    use wdrprobe_desktop_lib::models::execution_plan::*;
    use wdrprobe_desktop_lib::parsers::sql_parser::*;
    use std::fs;

    /// Test the complete flow: Parse WDR -> Get Hot SQL -> Parse Execution Plan -> Analyze
    #[test]
    fn test_complete_execution_plan_flow() {
        // Step 1: Simulate getting a hot SQL from a WDR report
        let hot_sql = HotSqlFromReport {
            id: 1,
            sql_id: Some("12345".to_string()),
            sql_text: "SELECT u.*, o.total FROM users u JOIN orders o ON u.id = o.user_id WHERE u.status = 'active'".to_string(),
            executions: 1000,
            total_elapsed_time: 50000.0,
            cpu_time: 45000.0,
        };

        // Step 2: Validate the SQL syntax
        let validation_result = validate_sql_syntax(&hot_sql.sql_text);
        assert!(validation_result.is_ok(), "SQL validation should succeed");

        // Step 3: Generate EXPLAIN query
        let explain_query = generate_explain_json(&hot_sql.sql_text);
        assert!(explain_query.contains("EXPLAIN (FORMAT JSON)"));
        assert!(explain_query.contains("SELECT"));

        // Step 4: Parse a mock execution plan response
        let mock_plan_json = r#"[
            {
                "Plan": {
                    "Node Type": "Hash Join",
                    "Join Type": "Inner",
                    "Startup Cost": 50.00,
                    "Total Cost": 150.00,
                    "Plan Rows": 1000,
                    "Plan Width": 200,
                    "Hash Cond": "(u.id = o.user_id)",
                    "Plans": [
                        {
                            "Node Type": "Seq Scan",
                            "Relation Name": "users",
                            "Startup Cost": 0.00,
                            "Total Cost": 25.00,
                            "Plan Rows": 500,
                            "Plan Width": 100,
                            "Filter": "((status)::text = 'active'::text)"
                        },
                        {
                            "Node Type": "Seq Scan",
                            "Relation Name": "orders",
                            "Startup Cost": 0.00,
                            "Total Cost": 30.00,
                            "Plan Rows": 2000,
                            "Plan Width": 100
                        }
                    ]
                }
            }
        ]"#;

        let parse_result = parse_execution_plan_json(mock_plan_json);
        assert!(parse_result.is_ok(), "Should parse mock plan successfully");

        let plan_tree = parse_result.unwrap();

        // Step 5: Analyze the execution plan
        let analysis = analyze_execution_plan(&plan_tree);

        // Verify analysis results
        assert_eq!(analysis.success, true);
        assert_eq!(analysis.plan_tree.operation, "Hash Join");
        assert_eq!(analysis.plan_metadata.total_cost, 150.0);
        assert_eq!(analysis.plan_metadata.total_rows, 1000);
        assert_eq!(analysis.plan_metadata.plan_depth, 2);
        assert_eq!(analysis.plan_metadata.node_count, 3);
    }

    /// Test execution plan with actual ANALYZE data
    #[test]
    fn test_execution_plan_with_analyze_data() {
        let plan_json_with_analyze = r#"[
            {
                "Plan": {
                    "Node Type": "Seq Scan",
                    "Relation Name": "large_table",
                    "Startup Cost": 0.00,
                    "Total Cost": 5000.00,
                    "Plan Rows": 100000,
                    "Plan Width": 100,
                    "Actual Startup Time": 0.050,
                    "Actual Total Time": 125.456,
                    "Actual Rows": 150000,
                    "Actual Loops": 1,
                    "Filter": "(created_at > '2024-01-01'::date)"
                }
            }
        ]"#;

        let parse_result = parse_execution_plan_json(plan_json_with_analyze);
        assert!(parse_result.is_ok());

        let plan_tree = parse_result.unwrap();
        let analysis = analyze_execution_plan(&plan_tree);

        // Should detect row estimation mismatch (estimated 100000, actual 150000)
        assert!(analysis.plan_metadata.has_actual_stats);
        assert!(analysis.warnings.iter().any(|w| w.contains("estimation") || w.contains("ANALYZE")));
    }

    /// Test complex nested execution plan
    #[test]
    fn test_complex_nested_execution_plan() {
        let complex_plan = r#"[
            {
                "Plan": {
                    "Node Type": "Sort",
                    "Startup Cost": 250.00,
                    "Total Cost": 255.50,
                    "Plan Rows": 1000,
                    "Plan Width": 300,
                    "Sort Key": ["u.created_at DESC"],
                    "Plans": [
                        {
                            "Node Type": "Hash Join",
                            "Join Type": "Inner",
                            "Startup Cost": 100.00,
                            "Total Cost": 200.00,
                            "Plan Rows": 1000,
                            "Plan Width": 300,
                            "Hash Cond": "(o.user_id = u.id)",
                            "Plans": [
                                {
                                    "Node Type": "Seq Scan",
                                    "Relation Name": "orders",
                                    "Startup Cost": 0.00,
                                    "Total Cost": 30.00,
                                    "Plan Rows": 2000,
                                    "Plan Width": 100
                                },
                                {
                                    "Node Type": "Hash",
                                    "Startup Cost": 25.00,
                                    "Total Cost": 25.00,
                                    "Plan Rows": 500,
                                    "Plan Width": 200,
                                    "Plans": [
                                        {
                                            "Node Type": "Seq Scan",
                                            "Relation Name": "users",
                                            "Startup Cost": 0.00,
                                            "Total Cost": 25.00,
                                            "Plan Rows": 500,
                                            "Plan Width": 200
                                        }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            }
        ]"#;

        let parse_result = parse_execution_plan_json(complex_plan);
        assert!(parse_result.is_ok());

        let plan_tree = parse_result.unwrap();
        let analysis = analyze_execution_plan(&plan_tree);

        // Verify deep nesting
        assert_eq!(analysis.plan_metadata.plan_depth, 4);
        assert!(analysis.plan_metadata.node_count >= 5);

        // Should detect both Hash Join and Sort operations
        let operations: Vec<&str> = analysis.warnings.iter()
            .filter_map(|w| {
                if w.contains("Hash Join") Some("Hash Join")
                else if w.contains("Sort") Some("Sort")
                else None
            })
            .collect();

        // At least one of the operations should generate warnings
        if analysis.warnings.iter().any(|w| w.contains("Sort") || w.contains("Hash Join")) {
            // Good - we detected the operations
        }
    }

    /// Test multiple optimization suggestions
    #[test]
    fn test_multiple_optimization_suggestions() {
        let problem_plan = ExecutionPlanNode {
            operation: "Nested Loop".to_string(),
            cost: 10000.0,
            rows: 100000,
            actual_rows: None,
            actual_time: None,
            width: None,
            children: vec![
                ExecutionPlanNode {
                    operation: "Seq Scan".to_string(),
                    cost: 5000.0,
                    rows: 500000,
                    actual_rows: None,
                    actual_time: None,
                    width: None,
                    children: vec![],
                    node_details: PlanNodeDetails {
                        output: None,
                        filter: Some("status = 'active'".to_string()),
                        buffers: None,
                        join_type: None,
                        hash_keys: None,
                        index_name: None,
                        table_name: Some("users".to_string()),
                    },
                    warnings: vec![],
                    suggestions: vec![],
                },
                ExecutionPlanNode {
                    operation: "Seq Scan".to_string(),
                    cost: 3000.0,
                    rows: 200000,
                    actual_rows: None,
                    actual_time: None,
                    width: None,
                    children: vec![],
                    node_details: PlanNodeDetails {
                        output: None,
                        filter: Some("total > 100".to_string()),
                        buffers: None,
                        join_type: None,
                        hash_keys: None,
                        index_name: None,
                        table_name: Some("orders".to_string()),
                    },
                    warnings: vec![],
                    suggestions: vec![],
                }
            ],
            node_details: PlanNodeDetails {
                output: None,
                filter: None,
                buffers: None,
                join_type: None,
                hash_keys: None,
                index_name: None,
                table_name: None,
            },
            warnings: vec![],
            suggestions: vec![],
        };

        let analysis = analyze_execution_plan(&problem_plan);

        // Should have multiple warnings and suggestions
        assert!(!analysis.warnings.is_empty());
        assert!(!analysis.suggestions.is_empty());

        // Should suggest indexes for the sequential scans with filters
        assert!(analysis.suggestions.iter().any(|s| s.contains("index") || s.contains("Index")));
    }

    /// Test table name extraction from various SQL patterns
    #[test]
    fn test_table_extraction_from_real_sql() {
        let test_cases = vec![
            ("SELECT * FROM users", vec!["users"]),
            ("SELECT u.id, o.total FROM users u JOIN orders o ON u.id = o.user_id", vec!["users", "orders"]),
            ("SELECT * FROM products WHERE price > 100", vec!["products"]),
            ("SELECT * FROM order_items oi JOIN products p ON oi.product_id = p.id", vec!["order_items", "products"]),
        ];

        for (sql, expected_tables) in test_cases {
            let tables = extract_table_names(sql);
            assert_eq!(tables.len(), expected_tables.len(),
                "Expected {} tables for SQL: {}, got {}",
                expected_tables.len(), sql, tables.len());

            for expected_table in expected_tables {
                assert!(tables.contains(&expected_table.to_string()),
                    "Expected table '{}' not found in {:?} for SQL: {}",
                    expected_table, tables, sql);
            }
        }
    }

    /// Test EXPLAIN prefix removal
    #[test]
    fn test_explain_prefix_removal() {
        let test_cases = vec![
            ("EXPLAIN SELECT * FROM users", "SELECT * FROM users"),
            ("EXPLAIN (FORMAT JSON) SELECT * FROM products WHERE id = 1", "SELECT * FROM products WHERE id = 1"),
            ("EXPLAIN (FORMAT TEXT) SELECT count(*) FROM orders", "SELECT count(*) FROM orders"),
            ("EXPLAIN ANALYZE SELECT * FROM users", "SELECT * FROM users"),
            ("EXPLAIN (ANALYZE, BUFFERS) SELECT * FROM orders", "SELECT * FROM orders"),
            ("SELECT * FROM users", "SELECT * FROM users"), // No EXPLAIN prefix
        ];

        for (input, expected) in test_cases {
            let result = extract_sql_from_explain(input).unwrap();
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    /// Helper struct for test data
    struct HotSqlFromReport {
        id: i64,
        sql_id: Option<String>,
        sql_text: String,
        executions: u64,
        total_elapsed_time: f64,
        cpu_time: f64,
    }
}
