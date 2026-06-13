// SQL Audit commands
// IPC commands for SQL audit and detection rules
// Per Constitution Principle IX - Audit trail for all operations

#[cfg(feature = "diagnostic-engines")]
use crate::adapters::metamorphosis_adapter;
use crate::adapters::schema_extractor;
use wdrprobe_core::database::DatabaseOperations;
use wdrprobe_core::database::DatabasePool;
use wdrprobe_core::models::audit::*;
use wdrprobe_core::models::TopSql;
use rusqlite::params;
use tauri::State;

// ============================================================================
// Detection Rules Module
// ============================================================================

/// SQL Audit Detection Rules
/// Implements performance issue detection based on WDR data
struct AuditDetectionRules;

impl AuditDetectionRules {
    /// Thresholds for detecting issues (can be configured via threshold_configs)
    const FULL_SCAN_ROWS_THRESHOLD: u64 = 10000;
    const SEQ_SCAN_RATIO_THRESHOLD: f64 = 0.5; // 50% seq scans
    const HIGH_COST_THRESHOLD: f64 = 10000.0;
    const CARTESIAN_PRODUCT_ROWS: u64 = 1000000;
    const HASH_JOIN_ROWS: u64 = 500000;
    const SORT_ROWS_THRESHOLD: u64 = 50000;
    const STALE_STATS_DAYS: i64 = 30; // days since last ANALYZE

    /// Detect full table scan issues
    fn detect_full_table_scan(
        sql: &TopSql,
        execution_plan: Option<&wdrprobe_core::models::SqlExecutionPlan>,
    ) -> Vec<SqlAuditIssue> {
        let mut issues = Vec::new();

        // Check if sequential scans dominate
        let has_high_seq_scan = sql.rows_processed > Self::FULL_SCAN_ROWS_THRESHOLD;

        // Check execution plan for Seq Scan nodes
        let plan_has_seq_scan = execution_plan.map_or(false, |plan| {
            Self::plan_contains_node_type(&plan.plan_tree, "Seq Scan")
        });

        if has_high_seq_scan || plan_has_seq_scan {
            let severity = if sql.rows_processed > 100000 {
                AuditSeverity::Critical
            } else if sql.rows_processed > 50000 {
                AuditSeverity::High
            } else {
                AuditSeverity::Medium
            };

            issues.push(SqlAuditIssue {
                id: 0,
                report_id: Some(sql.report_id),
                sql_id: Some(sql.id),
                issue_type: AuditIssueType::FullTableScan,
                severity,
                title: "Full Table Scan Detected".to_string(),
                description: format!(
                    "Query processed {} rows using full table scan. This can be slow on large tables.",
                    sql.rows_processed
                ),
                problematic_sql: Some(sql.sql_text.clone()),
                recommendation: "High row count suggests potential full table scan. For specific index recommendations, provide execution plan data showing which table/columns are being scanned.".to_string(),
                status: AuditStatus::Open,
                detected_at: chrono::Utc::now().to_rfc3339(),
                resolved_at: None,
                resolved_by: None,
            });
        }

        issues
    }

    /// Detect missing index issues
    fn detect_missing_index(
        sql: &TopSql,
        _execution_plan: Option<&wdrprobe_core::models::SqlExecutionPlan>,
    ) -> Vec<SqlAuditIssue> {
        let mut issues = Vec::new();

        // High disk reads relative to buffer gets suggests missing index
        let disk_read_ratio = if sql.buffer_gets > 0 {
            sql.disk_reads as f64 / sql.buffer_gets as f64
        } else {
            0.0
        };

        if disk_read_ratio > 0.3 && sql.rows_processed > 1000 {
            issues.push(SqlAuditIssue {
                id: 0,
                report_id: Some(sql.report_id),
                sql_id: Some(sql.id),
                issue_type: AuditIssueType::MissingIndex,
                severity: if disk_read_ratio > 0.6 { AuditSeverity::High } else { AuditSeverity::Medium },
                title: "Potential Missing Index".to_string(),
                description: format!(
                    "Query has high disk read ratio ({:.1}%). Consider adding indexes on filter/join columns.",
                    disk_read_ratio * 100.0
                ),
                problematic_sql: Some(sql.sql_text.clone()),
                recommendation: "High disk I/O ratio suggests potential missing indexes. To identify specific columns, analyze execution plan or examine WHERE/JOIN clauses in the SQL text.".to_string(),
                status: AuditStatus::Open,
                detected_at: chrono::Utc::now().to_rfc3339(),
                resolved_at: None,
                resolved_by: None,
            });
        }

        issues
    }

    /// Detect inefficient join operations
    fn detect_inefficient_join(
        sql: &TopSql,
        execution_plan: Option<&wdrprobe_core::models::SqlExecutionPlan>,
    ) -> Vec<SqlAuditIssue> {
        let mut issues = Vec::new();

        if let Some(plan) = execution_plan {
            // Check for Nested Loop without proper index support
            if Self::plan_contains_nested_loop(&plan.plan_tree) && sql.rows_processed > 10000 {
                issues.push(SqlAuditIssue {
                    id: 0,
                    report_id: Some(sql.report_id),
                    sql_id: Some(sql.id),
                    issue_type: AuditIssueType::InefficientJoin,
                    severity: AuditSeverity::Medium,
                    title: "Inefficient Join Detected".to_string(),
                    description: "Query uses nested loop join on large result set. Consider hash or merge join.".to_string(),
                    problematic_sql: Some(sql.sql_text.clone()),
                     recommendation: "Large dataset with nested loop join detected. Review execution plan to confirm join method and consider hash/merge join alternatives.".to_string(),
                    status: AuditStatus::Open,
                    detected_at: chrono::Utc::now().to_rfc3339(),
                    resolved_at: None,
                    resolved_by: None,
                });
            }
        }

        issues
    }

    /// Detect missing database statistics
    fn detect_missing_stats(
        sql: &TopSql,
        _execution_plan: Option<&wdrprobe_core::models::SqlExecutionPlan>,
    ) -> Vec<SqlAuditIssue> {
        let mut issues = Vec::new();

        // Inconsistent execution times suggest plan instability (often due to stale stats)
        let avg_time = sql.total_elapsed_time / sql.executions as f64;
        if avg_time > 1000.0 && sql.executions > 10 {
            issues.push(SqlAuditIssue {
                id: 0,
                report_id: Some(sql.report_id),
                sql_id: Some(sql.id),
                issue_type: AuditIssueType::MissingStats,
                severity: AuditSeverity::Medium,
                title: "Potential Stale Statistics".to_string(),
                description: "Query has inconsistent execution times, possibly due to outdated optimizer statistics.".to_string(),
                problematic_sql: Some(sql.sql_text.clone()),
                 recommendation: "Inconsistent execution times may indicate stale statistics. Run ANALYZE on tables referenced in this query.".to_string(),
                status: AuditStatus::Open,
                detected_at: chrono::Utc::now().to_rfc3339(),
                resolved_at: None,
                resolved_by: None,
            });
        }

        issues
    }

    /// Detect expensive function usage
    fn detect_expensive_function(
        sql: &TopSql,
        _execution_plan: Option<&wdrprobe_core::models::SqlExecutionPlan>,
    ) -> Vec<SqlAuditIssue> {
        let mut issues = Vec::new();

        let sql_upper = sql.sql_text.to_uppercase();

        // Check for function calls on indexed columns in WHERE clause
        let expensive_functions = vec![
            ("UPPER", "Case-insensitive search prevents index usage"),
            ("LOWER", "Case-insensitive search prevents index usage"),
            ("SUBSTR", "Substring function prevents index usage"),
            ("TRIM", "Trim function prevents index usage"),
            ("TO_CHAR", "Type conversion in WHERE clause"),
        ];

        for (func, desc) in expensive_functions {
            if sql_upper.contains(&format!("{}(", func)) {
                issues.push(SqlAuditIssue {
                    id: 0,
                    report_id: Some(sql.report_id),
                    sql_id: Some(sql.id),
                    issue_type: AuditIssueType::ExpensiveFunction,
                    severity: AuditSeverity::Medium,
                    title: format!("Function Call on Column: {}", func),
                    description: desc.to_string(),
                    problematic_sql: Some(sql.sql_text.clone()),
                    recommendation: format!("Function {}() detected in SQL. For specific column recommendations, analyze execution plan to identify which columns need functional indexes.", func),
                    status: AuditStatus::Open,
                    detected_at: chrono::Utc::now().to_rfc3339(),
                    resolved_at: None,
                    resolved_by: None,
                });
                break; // Only report one function issue per query
            }
        }

        issues
    }

    /// Detect cartesian product (missing join condition)
    fn detect_cartesian_product(
        sql: &TopSql,
        _execution_plan: Option<&wdrprobe_core::models::SqlExecutionPlan>,
    ) -> Vec<SqlAuditIssue> {
        let mut issues = Vec::new();

        // Check for multiple tables in SQL with very high row count
        let from_count = sql.sql_text.matches(" FROM ").count()
            + sql.sql_text.matches(" from ").count()
            + sql.sql_text.matches(", ").count();

        if from_count > 1 && sql.rows_processed > Self::CARTESIAN_PRODUCT_ROWS {
            issues.push(SqlAuditIssue {
                id: 0,
                report_id: Some(sql.report_id),
                sql_id: Some(sql.id),
                issue_type: AuditIssueType::CartesianProduct,
                severity: AuditSeverity::Critical,
                title: "Potential Cartesian Product Detected".to_string(),
                description: format!(
                    "Query processes {} rows from multiple tables, suggesting a missing join condition.",
                    sql.rows_processed
                ),
                problematic_sql: Some(sql.sql_text.clone()),
                 recommendation: "High row count from multiple tables detected. Review SQL text to verify proper JOIN conditions are present.".to_string(),
                status: AuditStatus::Open,
                detected_at: chrono::Utc::now().to_rfc3339(),
                resolved_at: None,
                resolved_by: None,
            });
        }

        issues
    }

    /// Detect nested loop with index on large table
    fn detect_nested_loop_with_index(
        sql: &TopSql,
        execution_plan: Option<&wdrprobe_core::models::SqlExecutionPlan>,
    ) -> Vec<SqlAuditIssue> {
        let mut issues = Vec::new();

        if let Some(plan) = execution_plan {
            if Self::plan_contains_node_type(&plan.plan_tree, "Nested Loop")
                && sql.rows_processed > 100000
                && sql.buffer_gets > 1000000
            {
                issues.push(SqlAuditIssue {
                    id: 0,
                    report_id: Some(sql.report_id),
                    sql_id: Some(sql.id),
                    issue_type: AuditIssueType::NestedLoopWithIndex,
                    severity: AuditSeverity::High,
                    title: "Nested Loop on Large Table".to_string(),
                    description: "Query uses nested loop join on large table. Consider hash join or covering index.".to_string(),
                    problematic_sql: Some(sql.sql_text.clone()),
                     recommendation: "Large dataset with nested loop join detected. Review execution plan to identify specific tables/columns for optimization.".to_string(),
                    status: AuditStatus::Open,
                    detected_at: chrono::Utc::now().to_rfc3339(),
                    resolved_at: None,
                    resolved_by: None,
                });
            }
        }

        issues
    }

    /// Detect hash join spilling to disk
    fn detect_hash_join_too_large(
        sql: &TopSql,
        _execution_plan: Option<&wdrprobe_core::models::SqlExecutionPlan>,
    ) -> Vec<SqlAuditIssue> {
        let mut issues = Vec::new();

        if sql.rows_processed > Self::HASH_JOIN_ROWS {
            issues.push(SqlAuditIssue {
                id: 0,
                report_id: Some(sql.report_id),
                sql_id: Some(sql.id),
                issue_type: AuditIssueType::HashJoinTooLarge,
                severity: AuditSeverity::High,
                title: "Large Hash Join Detected".to_string(),
                description: format!(
                    "Hash join processes {} rows, may exceed work_mem and spill to disk.",
                    sql.rows_processed
                ),
                problematic_sql: Some(sql.sql_text.clone()),
                 recommendation: "Large hash join detected. Consider increasing work_mem or reviewing execution plan for specific optimization opportunities.".to_string(),
                status: AuditStatus::Open,
                detected_at: chrono::Utc::now().to_rfc3339(),
                resolved_at: None,
                resolved_by: None,
            });
        }

        issues
    }

    /// Detect expensive sort operations
    fn detect_sort_operation(
        sql: &TopSql,
        _execution_plan: Option<&wdrprobe_core::models::SqlExecutionPlan>,
    ) -> Vec<SqlAuditIssue> {
        let mut issues = Vec::new();

        // Check for ORDER BY without proper index
        let has_order_by = sql.sql_text.to_uppercase().contains(" ORDER BY ");

        if has_order_by && sql.rows_processed > Self::SORT_ROWS_THRESHOLD {
            issues.push(SqlAuditIssue {
                id: 0,
                report_id: Some(sql.report_id),
                sql_id: Some(sql.id),
                issue_type: AuditIssueType::SortOperation,
                severity: AuditSeverity::Low,
                title: "Large Sort Operation Detected".to_string(),
                description: format!(
                    "Query sorts {} rows. Consider adding index on sort column.",
                    sql.rows_processed
                ),
                problematic_sql: Some(sql.sql_text.clone()),
                 recommendation: "Large sort operation detected. To identify specific columns for indexing, examine the ORDER BY clause in the SQL text.".to_string(),
                status: AuditStatus::Open,
                detected_at: chrono::Utc::now().to_rfc3339(),
                resolved_at: None,
                resolved_by: None,
            });
        }

        issues
    }

    /// Helper: Check if plan tree contains a specific node type
    fn plan_contains_node_type(plan: &wdrprobe_core::models::ExecutionPlanNode, node_type: &str) -> bool {
        if plan.operation.contains(node_type) {
            return true;
        }
        plan.children
            .iter()
            .any(|child| Self::plan_contains_node_type(child, node_type))
    }

    /// Helper: Check if plan contains nested loop
    fn plan_contains_nested_loop(plan: &wdrprobe_core::models::ExecutionPlanNode) -> bool {
        Self::plan_contains_node_type(plan, "Nested Loop")
    }

    /// Run all detection rules on a SQL statement
    fn detect_all_issues(
        sql: &TopSql,
        execution_plan: Option<&wdrprobe_core::models::SqlExecutionPlan>,
    ) -> Vec<SqlAuditIssue> {
        let mut issues = Vec::new();

        issues.extend(Self::detect_full_table_scan(sql, execution_plan));
        issues.extend(Self::detect_missing_index(sql, execution_plan));
        issues.extend(Self::detect_inefficient_join(sql, execution_plan));
        issues.extend(Self::detect_missing_stats(sql, execution_plan));
        issues.extend(Self::detect_expensive_function(sql, execution_plan));
        issues.extend(Self::detect_cartesian_product(sql, execution_plan));
        issues.extend(Self::detect_nested_loop_with_index(sql, execution_plan));
        issues.extend(Self::detect_hash_join_too_large(sql, execution_plan));
        issues.extend(Self::detect_sort_operation(sql, execution_plan));

        issues
    }
}

// ============================================================================
// Result Types for Audit Operations
// ============================================================================

#[derive(Debug, serde::Serialize)]
pub struct AuditRunResult {
    pub success: bool,
    pub reports_audited: usize,
    pub new_issues_found: usize,
    pub existing_issues_updated: usize,
    pub issues: Vec<SqlAuditIssue>,
    pub duration_ms: u64,
    pub message: Option<String>,
}

// ============================================================================
// IPC Commands
// ============================================================================

/// Run SQL audit on specified reports
#[tauri::command(rename_all = "camelCase")]
pub async fn run_sql_audit(
    pool: State<'_, DatabasePool>,
    report_ids: Option<Vec<i64>>,
    include_resolved: bool,
    audit_types: Option<Vec<String>>,
) -> Result<AuditRunResult, String> {
    let start = std::time::Instant::now();
    let conn = pool.get().map_err(|e| e.to_string())?;

    // Determine which reports to audit
    let reports_to_audit = if let Some(ids) = report_ids {
        ids
    } else {
        // Get all reports
        let mut stmt = conn
            .prepare("SELECT id FROM wdr_reports ORDER BY generation_time DESC")
            .map_err(|e| e.to_string())?;
        let report_iter = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        report_iter.filter_map(|r| r.ok()).collect()
    };

    let mut all_issues = Vec::new();
    let mut new_count = 0;
    let mut updated_count = 0;
    let reports_count = reports_to_audit.len();

    for report_id in reports_to_audit {
        // Get SQL statements for this report
        let sqls = pool
            .get_top_sqls_by_report(report_id)
            .map_err(|e| e.to_string())?;

        for sql in sqls {
            // Get execution plan if available
            let execution_plan = pool
                .get_execution_plan_by_sql(sql.id)
                .map_err(|e| e.to_string())?;

            // Run detection rules
            let mut issues = AuditDetectionRules::detect_all_issues(&sql, execution_plan.as_ref());

            // Filter by audit types if specified
            if let Some(ref types) = audit_types {
                let type_set: std::collections::HashSet<_> =
                    types.iter().filter_map(|t| parse_issue_type(t)).collect();
                issues.retain(|i| type_set.contains(&i.issue_type));
            }

            for issue in issues {
                // Check for existing similar issues
                let existing = check_existing_issue(&conn, report_id, sql.id, &issue.issue_type)
                    .map_err(|e| e.to_string())?;

                if let Some(existing_id) = existing {
                    if include_resolved {
                        // Update existing issue
                        update_existing_issue(&conn, existing_id, &issue)
                            .map_err(|e| e.to_string())?;
                        updated_count += 1;
                    }
                } else {
                    // Create new issue
                    create_audit_issue(&conn, report_id, &issue).map_err(|e| e.to_string())?;
                    new_count += 1;
                }

                all_issues.push(issue);
            }
        }
    }

    // Log audit run to audit_logs per Constitution IX
    let _ = pool
        .create_audit_log(&AuditLog {
            id: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            user_id: None,
            action: "RUN_SQL_AUDIT".to_string(),
            entity_type: "sql_audit".to_string(),
            entity_id: None,
            old_value: None,
            new_value: Some(
                serde_json::json!({
                    "reports_audited": reports_count,
                    "new_issues": new_count,
                    "updated_issues": updated_count
                })
                .to_string(),
            ),
            ip_address: None,
            success: true,
            error_message: None,
            details: Some(format!("Audited {} reports", reports_count)),
        })
        .map_err(|e| e.to_string())?;

    Ok(AuditRunResult {
        success: true,
        reports_audited: reports_count,
        new_issues_found: new_count,
        existing_issues_updated: updated_count,
        issues: all_issues,
        duration_ms: start.elapsed().as_millis() as u64,
        message: Some(format!("Audit completed: {} new issues found", new_count)),
    })
}

/// Get SQL audit issues with filtering and pagination
#[tauri::command(rename_all = "camelCase")]
pub async fn get_sql_audit_issues(
    pool: State<'_, DatabasePool>,
    report_id: Option<i64>,
    status: Option<String>,
    severity: Option<String>,
    issue_type: Option<String>,
    limit: Option<i32>,
    offset: Option<i32>,
    _sort_by: Option<String>,
) -> Result<SqlAuditIssueList, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    // Helper to build query and get results
    let build_and_query = |filters: (
        &Option<i64>,
        &Option<String>,
        &Option<String>,
        &Option<String>,
    )|
     -> Result<(Vec<SqlAuditIssue>, i64), String> {
        let (rid, status, severity, issue_type) = filters;

        // Build base query and params
        let mut where_clauses = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(r) = rid {
            where_clauses.push("report_id = ?");
            params.push(Box::new(*r));
        }
        if let Some(s) = status {
            where_clauses.push("status = ?");
            params.push(Box::new(s.as_str()));
        }
        if let Some(s) = severity {
            where_clauses.push("severity = ?");
            params.push(Box::new(s.as_str()));
        }
        if let Some(t) = issue_type {
            where_clauses.push("issue_type = ?");
            params.push(Box::new(t.as_str()));
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let base_query = format!(
            "SELECT * FROM sql_audit_issues {} ORDER BY detected_at DESC",
            where_clause
        );
        let count_query = format!("SELECT COUNT(*) FROM sql_audit_issues {}", where_clause);

        // Get count
        let total: i64 = if params.is_empty() {
            conn.query_row(&count_query, [], |row| row.get(0))
                .map_err(|e| e.to_string())?
        } else {
            // Convert to references
            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            conn.query_row(&count_query, param_refs.as_slice(), |row| row.get(0))
                .map_err(|e| e.to_string())?
        };

        // Build query with limit/offset
        let query = match (limit, offset) {
            (Some(l), Some(o)) => format!("{} LIMIT {} OFFSET {}", base_query, l, o),
            (Some(l), None) => format!("{} LIMIT {}", base_query, l),
            (None, Some(o)) => format!("{} OFFSET {}", base_query, o),
            (None, None) => base_query,
        };

        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

        let issues: Vec<SqlAuditIssue> = if params.is_empty() {
            stmt.query_map([], |row| map_row_to_issue(row))
                .map_err(|e| e.to_string())?
                .into_iter()
                .filter_map(|r| r.ok())
                .collect()
        } else {
            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            stmt.query_map(param_refs.as_slice(), |row| map_row_to_issue(row))
                .map_err(|e| e.to_string())?
                .into_iter()
                .filter_map(|r| r.ok())
                .collect()
        };

        Ok((issues, total))
    };

    let (issues, total) = build_and_query((&report_id, &status, &severity, &issue_type))?;

    // Generate summary
    let summary = generate_summary_from_db(&conn).map_err(|e| e.to_string())?;

    Ok(SqlAuditIssueList {
        issues,
        total,
        summary,
    })
}

/// Update single audit issue status
#[tauri::command(rename_all = "camelCase")]
pub async fn update_audit_issue_status(
    pool: State<'_, DatabasePool>,
    issue_id: i64,
    status: String,
    resolved_by: String,
    resolution_note: String,
) -> Result<UpdateAuditIssueResult, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    // Get current issue
    let current = conn
        .query_row(
            "SELECT * FROM sql_audit_issues WHERE id = ?",
            params![issue_id],
            |row| {
                Ok((
                    row.get::<_, String>("status")?,
                    row.get::<_, Option<String>>("resolved_at")?,
                    row.get::<_, Option<String>>("resolved_by")?,
                ))
            },
        )
        .map_err(|e| e.to_string())?;

    let old_status = parse_status(&current.0).unwrap_or(AuditStatus::Open);
    let new_status = parse_status(&status).unwrap_or(AuditStatus::Open);

    // Update the issue
    let resolved_at = matches!(
        new_status,
        AuditStatus::Fixed | AuditStatus::Whitelisted | AuditStatus::Ignored
    );
    let resolved_at_value = if resolved_at {
        Some(chrono::Utc::now().to_rfc3339())
    } else {
        None
    };

    let resolved_by_value = if matches!(new_status, AuditStatus::Fixed | AuditStatus::Whitelisted) {
        Some(resolved_by.clone())
    } else {
        None
    };

    conn.execute(
        "UPDATE sql_audit_issues SET status = ?, resolved_at = ?, resolved_by = ? WHERE id = ?",
        params![status, resolved_at_value, resolved_by_value, issue_id],
    )
    .map_err(|e| e.to_string())?;

    // Log to audit_logs per Constitution IX
    let _ = pool
        .create_audit_log(&AuditLog {
            id: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            user_id: Some(resolved_by.clone()),
            action: "UPDATE_AUDIT_ISSUE_STATUS".to_string(),
            entity_type: "sql_audit_issue".to_string(),
            entity_id: Some(issue_id),
            old_value: Some(format!("{:?}", old_status)),
            new_value: Some(format!("{:?}", new_status)),
            ip_address: None,
            success: true,
            error_message: None,
            details: Some(resolution_note),
        })
        .map_err(|e| e.to_string())?;

    Ok(UpdateAuditIssueResult {
        success: true,
        issue_id,
        old_status,
        new_status,
        message: Some("Issue status updated successfully".to_string()),
    })
}

/// Bulk update audit issues
#[tauri::command(rename_all = "camelCase")]
pub async fn bulk_update_audit_issues(
    pool: State<'_, DatabasePool>,
    issue_ids: Vec<i64>,
    status: String,
    resolved_by: String,
    resolution_note: String,
) -> Result<BulkUpdateResult, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut updated_count = 0;
    let mut failed_updates = Vec::new();

    let resolved_at = matches!(status.as_str(), "Fixed" | "Whitelisted" | "Ignored");
    let resolved_at_value = if resolved_at {
        Some(chrono::Utc::now().to_rfc3339())
    } else {
        None
    };

    let resolved_by_value = matches!(status.as_str(), "Fixed" | "Whitelisted");
    let resolved_by_db = if resolved_by_value {
        Some(resolved_by.clone())
    } else {
        None
    };

    for issue_id in &issue_ids {
        match conn.execute(
            "UPDATE sql_audit_issues SET status = ?, resolved_at = ?, resolved_by = ? WHERE id = ?",
            params![&status, resolved_at_value, resolved_by_db, issue_id],
        ) {
            Ok(_) => updated_count += 1,
            Err(e) => {
                failed_updates.push(FailedAuditUpdate {
                    issue_id: *issue_id,
                    error: e.to_string(),
                });
            }
        }
    }

    // Log bulk update to audit_logs per Constitution IX
    let _ = pool
        .create_audit_log(&AuditLog {
            id: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            user_id: Some(resolved_by.clone()),
            action: "BULK_UPDATE_AUDIT_ISSUES".to_string(),
            entity_type: "sql_audit_issue".to_string(),
            entity_id: None,
            old_value: None,
            new_value: Some(
                serde_json::json!({
                    "issue_ids": issue_ids,
                    "status": status
                })
                .to_string(),
            ),
            ip_address: None,
            success: failed_updates.is_empty(),
            error_message: if failed_updates.is_empty() {
                None
            } else {
                Some("Some updates failed".to_string())
            },
            details: Some(resolution_note),
        })
        .map_err(|e| e.to_string())?;

    Ok(BulkUpdateResult {
        success: failed_updates.is_empty(),
        updated_count,
        failed_updates,
        message: Some(format!("Updated {} issues", updated_count)),
    })
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Map database row to SqlAuditIssue
fn map_row_to_issue(row: &rusqlite::Row) -> rusqlite::Result<SqlAuditIssue> {
    Ok(SqlAuditIssue {
        id: row.get("id")?,
        report_id: row.get("report_id")?,
        sql_id: row.get("sql_id")?,
        issue_type: parse_issue_type(&row.get::<_, String>("issue_type")?)
            .unwrap_or(AuditIssueType::FullTableScan),
        severity: parse_severity(&row.get::<_, String>("severity")?)
            .unwrap_or(AuditSeverity::Medium),
        title: row.get("title")?,
        description: row.get("description")?,
        problematic_sql: row.get("problematic_sql")?,
        recommendation: row.get("recommendation")?,
        status: parse_status(&row.get::<_, String>("status")?).unwrap_or(AuditStatus::Open),
        detected_at: row.get("detected_at")?,
        resolved_at: row.get("resolved_at")?,
        resolved_by: row.get("resolved_by")?,
    })
}

/// Parse issue type from string
fn parse_issue_type(s: &str) -> Option<AuditIssueType> {
    match s {
        "FullTableScan" => Some(AuditIssueType::FullTableScan),
        "MissingIndex" => Some(AuditIssueType::MissingIndex),
        "InefficientJoin" => Some(AuditIssueType::InefficientJoin),
        "MissingStats" => Some(AuditIssueType::MissingStats),
        "ExpensiveFunction" => Some(AuditIssueType::ExpensiveFunction),
        "CartesianProduct" => Some(AuditIssueType::CartesianProduct),
        "NestedLoopWithIndex" => Some(AuditIssueType::NestedLoopWithIndex),
        "HashJoinTooLarge" => Some(AuditIssueType::HashJoinTooLarge),
        "SortOperation" => Some(AuditIssueType::SortOperation),
        _ => None,
    }
}

/// Parse severity from string
fn parse_severity(s: &str) -> Option<AuditSeverity> {
    match s {
        "Critical" => Some(AuditSeverity::Critical),
        "High" => Some(AuditSeverity::High),
        "Medium" => Some(AuditSeverity::Medium),
        "Low" => Some(AuditSeverity::Low),
        "Info" => Some(AuditSeverity::Info),
        _ => None,
    }
}

/// Parse status from string
fn parse_status(s: &str) -> Option<AuditStatus> {
    match s {
        "Open" => Some(AuditStatus::Open),
        "Reviewed" => Some(AuditStatus::Reviewed),
        "Whitelisted" => Some(AuditStatus::Whitelisted),
        "Fixed" => Some(AuditStatus::Fixed),
        "Ignored" => Some(AuditStatus::Ignored),
        _ => None,
    }
}

/// Check for existing similar issue
fn check_existing_issue(
    conn: &rusqlite::Connection,
    report_id: i64,
    sql_id: i64,
    issue_type: &AuditIssueType,
) -> Result<Option<i64>, rusqlite::Error> {
    let type_str = format!("{:?}", issue_type);
    let mut stmt = conn.prepare(
        "SELECT id FROM sql_audit_issues WHERE report_id = ? AND sql_id = ? AND issue_type = ? AND status = 'Open'"
    )?;
    let mut rows = stmt.query_map(params![report_id, sql_id, type_str], |row| row.get(0))?;

    match rows.next() {
        Some(Ok(id)) => Ok(Some(id)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

/// Create new audit issue
fn create_audit_issue(
    conn: &rusqlite::Connection,
    report_id: i64,
    issue: &SqlAuditIssue,
) -> Result<i64, rusqlite::Error> {
    let issue_type_str = format!("{:?}", issue.issue_type);
    let severity_str = format!("{:?}", issue.severity);
    let status_str = format!("{:?}", issue.status);

    conn.execute(
        r#"
        INSERT INTO sql_audit_issues (
            report_id, sql_id, issue_type, severity, title, description,
            problematic_sql, recommendation, status, detected_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        params![
            report_id,
            issue.sql_id,
            issue_type_str,
            severity_str,
            issue.title,
            issue.description,
            issue.problematic_sql,
            issue.recommendation,
            status_str,
            issue.detected_at
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Update existing issue
fn update_existing_issue(
    conn: &rusqlite::Connection,
    issue_id: i64,
    issue: &SqlAuditIssue,
) -> Result<(), rusqlite::Error> {
    let issue_type_str = format!("{:?}", issue.issue_type);
    let severity_str = format!("{:?}", issue.severity);

    conn.execute(
        r#"
        UPDATE sql_audit_issues SET
            issue_type = ?, severity = ?, title = ?, description = ?,
            problematic_sql = ?, recommendation = ?, detected_at = ?
        WHERE id = ?
        "#,
        params![
            issue_type_str,
            severity_str,
            issue.title,
            issue.description,
            issue.problematic_sql,
            issue.recommendation,
            issue.detected_at,
            issue_id
        ],
    )?;

    Ok(())
}

/// Generate summary from database
fn generate_summary_from_db(conn: &rusqlite::Connection) -> Result<AuditSummary, rusqlite::Error> {
    use serde_json::Value;

    // Get total count
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM sql_audit_issues", [], |row| {
        row.get(0)
    })?;

    // Get breakdown by severity
    let mut severity_stmt =
        conn.prepare("SELECT severity, COUNT(*) FROM sql_audit_issues GROUP BY severity")?;
    let mut by_severity = serde_json::Map::new();
    let severity_rows = severity_stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>("severity")?,
            row.get::<_, i64>("count")?,
        ))
    })?;
    for row in severity_rows {
        let (severity, count) = row?;
        by_severity.insert(severity, Value::Number(count.into()));
    }

    // Get breakdown by status
    let mut status_stmt =
        conn.prepare("SELECT status, COUNT(*) FROM sql_audit_issues GROUP BY status")?;
    let mut by_status = serde_json::Map::new();
    let status_rows = status_stmt.query_map([], |row| {
        Ok((row.get::<_, String>("status")?, row.get::<_, i64>("count")?))
    })?;
    for row in status_rows {
        let (status, count) = row?;
        by_status.insert(status, Value::Number(count.into()));
    }

    // Get breakdown by type
    let mut type_stmt =
        conn.prepare("SELECT issue_type, COUNT(*) FROM sql_audit_issues GROUP BY issue_type")?;
    let mut by_type = serde_json::Map::new();
    let type_rows = type_stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>("issue_type")?,
            row.get::<_, i64>("count")?,
        ))
    })?;
    for row in type_rows {
        let (issue_type, count) = row?;
        by_type.insert(issue_type, Value::Number(count.into()));
    }

    Ok(AuditSummary {
        total_issues: total,
        by_severity: Value::Object(by_severity),
        by_status: Value::Object(by_status),
        by_type: Value::Object(by_type),
    })
}

// ============================================================================
// SQL Rewrite Commands (metamorphosis integration)
// ============================================================================

#[cfg(feature = "diagnostic-engines")]
/// Rewrite SQL using metamorphosis rules
#[tauri::command(rename_all = "camelCase")]
pub async fn rewrite_sql(
    pool: State<'_, DatabasePool>,
    sql: String,
    report_id: Option<i64>,
    schema_json: Option<String>,
) -> Result<metamorphosis_adapter::RewriteOutput, String> {
    let schema = if let Some(json) = schema_json {
        Some(schema_extractor::parse_schema_json(&json)?)
    } else if let Some(rid) = report_id {
        Some(schema_extractor::extract_schema_from_wdr(pool.inner(), rid)?)
    } else {
        None
    };

    let adapter = metamorphosis_adapter::MetamorphosisAdapter::new();
    adapter.rewrite(&sql, schema.as_ref())
}

#[cfg(feature = "diagnostic-engines")]
/// List all available rewrite rules
#[tauri::command(rename_all = "camelCase")]
pub async fn list_rewrite_rules() -> Result<Vec<metamorphosis_adapter::RuleInfo>, String> {
    Ok(metamorphosis_adapter::list_builtin_rules())
}
