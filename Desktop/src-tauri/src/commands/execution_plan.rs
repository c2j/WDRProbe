// Execution plan commands
// IPC commands for SQL execution plan visualization

use crate::adapters::ogexplain_adapter;
use wdrprobe_core::database::{DatabaseOperations, DatabasePool};
use wdrprobe_core::models::execution_plan::*;
use wdrprobe_core::parsers::sql_parser::*;


/// Get hot SQL queries from WDR reports
#[tauri::command]
pub async fn get_wdr_hot_sqls(
    report_id: Option<i64>,
    limit: Option<i32>,
    sort_by: Option<String>,
    pool: tauri::State<'_, DatabasePool>,
) -> Result<WdrHotSqlList, String> {
    println!(
        "Backend: get_wdr_hot_sqls called with report_id={:?}, limit={:?}",
        report_id, limit
    );

    let effective_limit = limit.unwrap_or(50).min(100) as i32;

    // Get hot SQLs from database
    let top_sqls = pool
        .get_hot_sqls(Some(effective_limit))
        .map_err(|e| format!("Failed to retrieve hot SQLs: {}", e))?;

    // Get report info for instance name and generation time
    let mut hot_sqls = Vec::new();
    for sql in top_sqls {
        // If report_id is specified, filter by it
        if let Some(rid) = report_id {
            if sql.report_id != rid {
                continue;
            }
        }

        // Get report details
        if let Ok(Some(report)) = pool.get_wdr_report(sql.report_id) {
            hot_sqls.push(WdrHotSql {
                id: sql.id,
                report_id: sql.report_id,
                sql_id: sql.sql_id,
                sql_text: sql.sql_text,
                executions: sql.executions,
                total_elapsed_time: sql.total_elapsed_time,
                cpu_time: sql.cpu_time,
                rank: sql.rank_by_time.unwrap_or(0),
                instance_name: report.instance_name.clone(),
                generation_time: report.generation_time.clone(),
            });
        }
    }

    let total = hot_sqls.len() as i64;

    // Sort by specified field
    match sort_by.as_deref() {
        Some("cpu_time") => {
            hot_sqls.sort_by(|a, b| b.cpu_time.partial_cmp(&a.cpu_time).unwrap());
        }
        Some("executions") => {
            hot_sqls.sort_by(|a, b| b.executions.cmp(&a.executions));
        }
        _ => {
            // Default: sort by elapsed_time
            hot_sqls.sort_by(|a, b| {
                b.total_elapsed_time
                    .partial_cmp(&a.total_elapsed_time)
                    .unwrap()
            });
        }
    }

    Ok(WdrHotSqlList { hot_sqls, total })
}

/// Get execution plan for a specific SQL query
#[tauri::command]
pub async fn get_execution_plan(
    sql_id: Option<i64>,
    sql_text: Option<String>,
    plan_source: String,
    report_id: Option<i64>,
    pool: tauri::State<'_, DatabasePool>,
) -> Result<ExecutionPlanResponse, String> {
    println!(
        "Backend: get_execution_plan called with sql_id={:?}, plan_source={}",
        sql_id, plan_source
    );

    let plan_tree = match plan_source.as_str() {
        "FromWdrReport" | "HotSql" => {
            // Get SQL from database
            let effective_sql_id = sql_id.ok_or("sql_id required for FromWdrReport source")?;
            let top_sqls = pool
                .get_top_sqls_by_report(
                    report_id.ok_or("report_id required for FromWdrReport source")?,
                )
                .map_err(|e| format!("Failed to retrieve SQLs: {}", e))?;

            let sql = top_sqls
                .iter()
                .find(|s| s.id == effective_sql_id)
                .ok_or("SQL not found")?;

            // Check if we have a saved execution plan
            if let Ok(Some(saved_plan)) = pool.get_execution_plan_by_sql(effective_sql_id) {
                saved_plan.plan_tree
            } else {
                // No execution plan available - analyze SQL text for basic optimization suggestions
                return analyze_sql_without_plan(&sql.sql_text);
            }
        }
        "UserProvided" => {
            let sql = sql_text.ok_or("sql_text required for UserProvided source")?;

            // Validate SQL syntax
            validate_sql_syntax(&sql)?;

            // Analyze SQL text for basic optimization suggestions without execution plan
            return analyze_sql_without_plan(&sql);
        }
        _ => {
            return Err(format!("Unknown plan source: {}", plan_source));
        }
    };

    // Analyze the plan
    Ok(analyze_execution_plan(&plan_tree))
}

/// Parse execution plan from text or JSON
#[tauri::command]
pub async fn parse_execution_plan(
    plan_text: String,
    format: String,
    source: String,
) -> Result<ParsedPlan, String> {
    println!(
        "Backend: parse_execution_plan called with format={}, source={}",
        format, source
    );

    let plan_tree = match format.as_str() {
        "json" => parse_execution_plan_json(&plan_text)?,
        "text" => parse_execution_plan_text(&plan_text)?,
        "sql-plan" => parse_sql_plan_format(&plan_text)?,
        _ => {
            return Err(format!("Unsupported format: {}", format));
        }
    };

    let parse_warnings = if source != "gaussdb" {
        vec![format!(
            "Source is '{}' not 'gaussdb' - some operators may not be recognized",
            source
        )]
    } else {
        Vec::new()
    };

    // Extract SQL from the plan if it exists
    let sql = plan_tree.node_details.output.as_ref().and_then(|output| {
        output.iter().find_map(|line| {
            if line.starts_with("SQL: ") {
                Some(line[5..].to_string()) // Remove "SQL: " prefix
            } else {
                None
            }
        })
    });

    Ok(ParsedPlan {
        success: true,
        plan_tree,
        sql,
        parse_warnings,
        parsed_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Analyze execution plan for performance issues
#[tauri::command]
pub async fn analyze_execution_plan_command(
    plan: ExecutionPlanNode,
    thresholds: Option<ThresholdOverrides>,
) -> Result<PlanAnalysis, String> {
    println!("Backend: analyze_execution_plan called");

    let response = analyze_execution_plan(&plan);

    // Calculate performance score (0-100)
    let base_score = 100i32;
    let warning_penalty = response.warnings.len() as i32 * 5;
    let mut score = base_score.saturating_sub(warning_penalty);

    // Apply custom thresholds if provided
    if let Some(thresholds) = thresholds {
        if let Some(cost_threshold) = thresholds.cost_threshold {
            if response.plan_metadata.total_cost > cost_threshold {
                score = score.saturating_sub(10);
            }
        }
        if let Some(rows_threshold) = thresholds.rows_threshold {
            if response.plan_metadata.total_rows > rows_threshold {
                score = score.saturating_sub(10);
            }
        }
    }

    // Ensure score is in range 0-100
    score = score.max(0).min(100);

    // Generate issues from warnings
    let mut issues = Vec::new();
    let mut severity_counts = std::collections::HashMap::new();

    for (i, warning) in response.warnings.iter().enumerate() {
        let severity = if warning.contains("Critical") || warning.contains("High") {
            AuditSeverity::Critical
        } else if warning.contains("may") || warning.contains("consider") {
            AuditSeverity::Medium
        } else {
            AuditSeverity::Low
        };

        *severity_counts.entry(severity.clone()).or_insert(0) += 1;

        issues.push(PlanIssue {
            node_path: format!("node_{}", i),
            issue_type: classify_issue(warning),
            severity,
            description: warning.clone(),
            affected_rows: response.plan_metadata.total_rows,
            cost_impact: (response.plan_metadata.total_cost * 0.1) as i32,
        });
    }

        // Generate recommendations
        let mut recommendations = Vec::new();
        for suggestion in &response.suggestions {
            recommendations.push(PlanRecommendation {
                priority: if suggestion.contains("index") || suggestion.contains("Index") {
                    RecommendationPriority::High
                } else {
                    RecommendationPriority::Medium
                },
                action: "Optimize".to_string(),
                description: suggestion.clone(),
                sql_example: None,
                estimated_benefit: "Variable - depends on data distribution and query patterns".to_string(),
            });
        }

    // Determine optimization potential - be more conservative and honest about estimates
    let optimization_potential = if score >= 80 {
        "Low - Minor optimizations may be available".to_string()
    } else if score >= 50 {
        "Medium - Some optimization opportunities possible".to_string()
    } else {
        "High - Multiple optimization areas identified (requires execution plan for accuracy)".to_string()
    };

    // Be much more conservative with improvement estimates
    let estimated_improvement = if score < 50 {
        Some(15 + (80 - score) / 3) // Reduced from 30+ to 15+ 
    } else if score < 80 {
        Some(5 + (80 - score) / 4)  // Reduced from 10+ to 5+
    } else {
        None
    };

    Ok(PlanAnalysis {
        score,
        issues,
        recommendations,
        optimization_potential,
        estimated_improvement,
    })
}

/// Save execution plan for future reference
#[tauri::command]
pub async fn save_execution_plan(
    sql_id: Option<i64>,
    sql_text: Option<String>,
    plan_tree: ExecutionPlanNode,
    plan_source: String,
    report_id: Option<i64>,
    _name: Option<String>,
    pool: tauri::State<'_, DatabasePool>,
) -> Result<SavePlanResult, String> {
    println!("Backend: save_execution_plan called");

    // Determine effective SQL ID
    let effective_sql_id = if let Some(sid) = sql_id {
        sid
    } else if let Some(_) = sql_text {
        // For user-provided SQL, we'd need to create a TopSql entry first
        // For now, return an error
        return Err("User-provided SQL with plan saving not yet implemented".to_string());
    } else {
        return Err("Either sql_id or sql_text must be provided".to_string());
    };

    let source = match plan_source.as_str() {
        "FromWdrReport" => {
            format!("FromWdrReport({})", report_id.ok_or("report_id required")?)
        }
        "HotSql" => {
            format!("HotSql({})", report_id.ok_or("report_id required")?)
        }
        "UserProvided" => "UserProvided".to_string(),
        _ => return Err(format!("Unknown plan source: {}", plan_source)),
    };

    let plan = wdrprobe_core::models::SqlExecutionPlan {
        id: 0, // Will be assigned by database
        sql_id: Some(effective_sql_id),
        plan_tree,
        created_at: chrono::Utc::now().to_rfc3339(),
        source,
    };

    let plan_id = pool
        .create_execution_plan(&plan)
        .map_err(|e| format!("Failed to save execution plan: {}", e))?;

    Ok(SavePlanResult {
        success: true,
        plan_id,
        message: Some(format!("Execution plan saved with ID: {}", plan_id)),
    })
}

/// Get list of saved execution plans
#[tauri::command]
pub async fn get_saved_plans(
    sql_id: Option<i64>,
    report_id: Option<i64>,
    limit: Option<i32>,
    offset: Option<i32>,
    pool: tauri::State<'_, DatabasePool>,
) -> Result<SavedPlansResponse, String> {
    println!("Backend: get_saved_plans called");

    // Get plans by report or SQL
    let plans = if let Some(rid) = report_id {
        pool.get_execution_plans_by_report(rid)
            .map_err(|e| format!("Failed to retrieve plans: {}", e))?
    } else if let Some(sid) = sql_id {
        if let Some(plan) = pool
            .get_execution_plan_by_sql(sid)
            .map_err(|e| format!("Failed to retrieve plans: {}", e))?
        {
            vec![plan]
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let total = plans.len() as i64;

    // Apply pagination
    let start = offset.unwrap_or(0) as usize;
    let end = if let Some(limit) = limit {
        (start + limit as usize).min(plans.len())
    } else {
        plans.len()
    };

    let plans: Vec<SavedPlan> = plans[start..end]
        .iter()
        .map(|plan| {
            // Calculate metadata
            let (total_cost, node_count, _) = calculate_plan_metadata(&plan.plan_tree);

            SavedPlan {
                id: plan.id,
                sql_id: plan.sql_id,
                sql_text: None,
                source: plan.source.clone(),
                created_at: plan.created_at.clone(),
                total_cost,
                node_count,
                name: None,
            }
        })
        .collect();

    Ok(SavedPlansResponse { plans, total })
}

/// Delete a saved execution plan
#[tauri::command]
pub async fn delete_execution_plan(
    plan_id: i64,
    confirm: bool,
    pool: tauri::State<'_, DatabasePool>,
) -> Result<DeleteResult, String> {
    println!(
        "Backend: delete_execution_plan called for plan_id={}",
        plan_id
    );

    if !confirm {
        return Err("Confirmation required to delete execution plan".to_string());
    }

    pool.delete_execution_plan(plan_id)
        .map_err(|e| format!("Failed to delete execution plan: {}", e))?;

    Ok(DeleteResult {
        success: true,
        deleted_plan_id: plan_id,
        message: Some(format!("Execution plan {} deleted successfully", plan_id)),
    })
}

/// Generate optimization SQL (CREATE INDEX statements, etc.)
#[tauri::command]
pub async fn generate_optimization_sql(
    plan_id: i64,
    optimization_type: String,
    pool: tauri::State<'_, DatabasePool>,
) -> Result<OptimizationSql, String> {
    println!(
        "Backend: generate_optimization_sql called for plan_id={}, type={}",
        plan_id, optimization_type
    );

    // Get the plan
    let plan = pool
        .get_execution_plan_by_sql(plan_id)
        .map_err(|e| format!("Failed to retrieve plan: {}", e))?
        .ok_or("Plan not found")?;

    let mut sql_statements = Vec::new();
    let mut explanations = Vec::new();
    let mut warnings = Vec::new();

    // Check if this is a SQL analysis without real execution plan
    if plan.plan_tree.operation == "SQL Analysis" {
        warnings.push("Optimization SQL generation requires actual execution plan data. These suggestions are based on SQL text analysis only.".to_string());
        warnings.push("For accurate index and statistics recommendations, provide actual EXPLAIN output or execution plan data.".to_string());
        
        // Return empty SQL statements with warnings instead of generating potentially incorrect SQL
        return Ok(OptimizationSql {
            sql_statements,
            explanations,
            warnings,
            confidence: OptimizationConfidence::Low,
        });
    }

    match optimization_type.as_str() {
        "index" => {
            // Analyze plan for index opportunities
            analyze_for_indexes(&plan.plan_tree, &mut sql_statements, &mut explanations);
        }
        "statistics" => {
            // Generate ANALYZE statements
            analyze_for_statistics(&plan.plan_tree, &mut sql_statements, &mut explanations);
        }
        "rewrite" => {
            // Suggest query rewrites
            analyze_for_rewrite(&plan.plan_tree, &mut sql_statements, &mut explanations);
        }
        _ => {
            return Err(format!("Unknown optimization type: {}", optimization_type));
        }
    }

    let confidence = if !sql_statements.is_empty() {
        OptimizationConfidence::High
    } else {
        OptimizationConfidence::Low
    };

    Ok(OptimizationSql {
        sql_statements,
        explanations,
        warnings,
        confidence,
    })
}

/// Analyze SQL text without execution plan to provide basic optimization suggestions
pub fn analyze_sql_without_plan(sql: &str) -> Result<ExecutionPlanResponse, String> {
    println!("Analyzing SQL text without execution plan: {}", sql.chars().take(50).collect::<String>());
    
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();
    let mut plan_metadata = PlanMetadata {
        total_cost: 0.0, // No cost data available without execution plan
        total_rows: 0,   // No row estimates available without execution plan
        plan_depth: 0,
        node_count: 0,
        optimization_warnings: 0,
        estimated_time_ms: 0.0, // No timing data available
        gaussdb_format: false,    // This is not a real GaussDB format plan
        has_actual_stats: false,
    };

    // Convert SQL to uppercase for analysis
    let sql_upper = sql.to_uppercase();
    let sql_clean = sql_upper.trim();

    // Analyze SELECT statements
    if sql_clean.starts_with("SELECT") {
        analyze_select_statement(sql, &mut warnings, &mut suggestions);
    }
    // Analyze INSERT statements  
    else if sql_clean.starts_with("INSERT") {
        analyze_insert_statement(sql, &mut warnings, &mut suggestions);
    }
    // Analyze UPDATE statements
    else if sql_clean.starts_with("UPDATE") {
        analyze_update_statement(sql, &mut warnings, &mut suggestions);
    }
    // Analyze DELETE statements
    else if sql_clean.starts_with("DELETE") {
        analyze_delete_statement(sql, &mut warnings, &mut suggestions);
    }

    // Count optimization warnings
    plan_metadata.optimization_warnings = warnings.len() as u32;

    // Create a minimal plan tree for the response
    let plan_tree = ExecutionPlanNode {
        operation: "SQL_TEXT_ANALYSIS_ONLY".to_string(),
        cost: 0.0, // No cost data available
        rows: 0,   // No row estimates available
        actual_rows: None,
        actual_time: None,
        width: None, // No width data available
        children: vec![],
        node_details: PlanNodeDetails {
            output: Some(vec![
                "=== SQL TEXT ANALYSIS ONLY ===".to_string(),
                "No execution plan data available".to_string(),
                "Analysis based on SQL text patterns only".to_string(),
                "For accurate optimization, provide EXPLAIN output".to_string()
            ]),
            filter: None,
            buffers: None,
            join_type: None,
            hash_keys: None,
            index_name: None,
            table_name: None, // Don't extract table names to avoid confusion
        },
        warnings: vec![
            "WARNING: Analysis performed without execution plan data".to_string(),
            "Suggestions are based on SQL text patterns only".to_string(),
            "For accurate index/cost analysis, provide actual execution plan".to_string()
        ],
        suggestions: vec![
            "Upload EXPLAIN (FORMAT JSON) output for detailed analysis".to_string(),
            "Provide execution plan data for accurate optimization recommendations".to_string()
        ],
    };

    Ok(ExecutionPlanResponse {
        success: true,
        plan_tree,
        plan_metadata,
        warnings,
        suggestions,
    })
}

/// Analyze SELECT statements for optimization opportunities
fn analyze_select_statement(sql: &str, warnings: &mut Vec<String>, suggestions: &mut Vec<String>) {
    let sql_upper = sql.to_uppercase();

    // Check for SELECT *
    if sql_upper.contains("SELECT *") {
        warnings.push("SELECT * retrieves all columns which may be inefficient for large tables".to_string());
        suggestions.push("Specify only required columns instead of SELECT *".to_string());
    }

    // Check for missing WHERE clause on large tables
    if !sql_upper.contains("WHERE") && !sql_upper.contains("LIMIT") {
        warnings.push("Query lacks WHERE clause - may scan entire table".to_string());
        suggestions.push("Add appropriate WHERE clause to filter rows".to_string());
    }

    // Check for OR conditions that might prevent index usage
    if sql_upper.contains(" OR ") {
        warnings.push("OR conditions may prevent efficient index usage".to_string());
        suggestions.push("Consider using UNION or rewriting with appropriate indexes".to_string());
    }

    // Check for LIKE with leading wildcard
    if sql_upper.contains("LIKE '%") {
        warnings.push("LIKE with leading wildcard prevents index usage".to_string());
        suggestions.push("Consider full-text search or restructuring the query".to_string());
    }

    // Check for NOT IN which might be inefficient
    if sql_upper.contains("NOT IN") {
        warnings.push("NOT IN can be inefficient with large lists".to_string());
        suggestions.push("Consider using NOT EXISTS or LEFT JOIN with NULL check".to_string());
    }

    // Check for subqueries in SELECT clause
    if sql_upper.contains("SELECT (") {
        warnings.push("Subqueries in SELECT clause may execute for each row".to_string());
        suggestions.push("Consider using JOINs or CTEs instead".to_string());
    }

    // Check for DISTINCT which might indicate missing indexes
    if sql_upper.contains("DISTINCT") {
        warnings.push("DISTINCT requires sorting which can be expensive".to_string());
        suggestions.push("Consider adding appropriate indexes or restructuring query".to_string());
    }

    // Check for ORDER BY without LIMIT on large result sets
    if sql_upper.contains("ORDER BY") && !sql_upper.contains("LIMIT") {
        warnings.push("ORDER BY without LIMIT may sort large result sets".to_string());
        suggestions.push("Add LIMIT clause or ensure appropriate covering index".to_string());
    }

    // Check for JOIN without proper syntax
    if sql_upper.contains("JOIN") && !sql_upper.contains("ON") && !sql_upper.contains("USING") {
        warnings.push("JOIN without ON or USING clause detected".to_string());
        suggestions.push("Specify proper JOIN conditions with ON or USING clauses".to_string());
    }

    // Add specific warnings about SQL analysis limitations
    warnings.push("SQL TEXT ANALYSIS: Cannot determine actual table sizes or data distribution".to_string());
    warnings.push("Index recommendations require schema knowledge and execution plan data".to_string());
}

/// Analyze INSERT statements for optimization opportunities
fn analyze_insert_statement(sql: &str, warnings: &mut Vec<String>, suggestions: &mut Vec<String>) {
    let sql_upper = sql.to_uppercase();

    // Check for INSERT without column specification
    // Look for INSERT INTO table_name VALUES pattern (no column list)
    if sql_upper.contains("INSERT INTO") && sql_upper.contains("VALUES") {
        // Find position of INSERT INTO and VALUES
        if let (Some(insert_pos), Some(values_pos)) = (sql_upper.find("INSERT INTO"), sql_upper.find("VALUES")) {
            // Check if there's no opening parenthesis between INSERT INTO and VALUES
            let between_insert_and_values = &sql_upper[insert_pos + 11..values_pos];
            if !between_insert_and_values.contains('(') {
                warnings.push("INSERT without column specification may cause issues with schema changes".to_string());
                suggestions.push("Explicitly specify column names in INSERT statement".to_string());
            }
        }
    }

    // Check for large batch inserts
    if sql_upper.matches("VALUES").count() > 10 {
        warnings.push("Large batch insert detected".to_string());
        suggestions.push("Consider using COPY or batching in smaller chunks".to_string());
    }
}

/// Analyze UPDATE statements for optimization opportunities
fn analyze_update_statement(sql: &str, warnings: &mut Vec<String>, suggestions: &mut Vec<String>) {
    let sql_upper = sql.to_uppercase();

    // Check for UPDATE without WHERE clause
    if !sql_upper.contains("WHERE") {
        warnings.push("UPDATE without WHERE clause will affect all rows".to_string());
        suggestions.push("Add appropriate WHERE clause to limit affected rows".to_string());
    }

    // Check for UPDATE on indexed columns
    // This is a basic check - in real scenarios you'd need schema information
    if sql_upper.contains("SET") && sql_upper.contains("WHERE") {
        warnings.push("UPDATE on columns with indexes may require index maintenance".to_string());
        suggestions.push("Consider impact on indexes and potential fragmentation".to_string());
    }
}

/// Analyze DELETE statements for optimization opportunities
fn analyze_delete_statement(sql: &str, warnings: &mut Vec<String>, suggestions: &mut Vec<String>) {
    let sql_upper = sql.to_uppercase();

    // Check for DELETE without WHERE clause
    if !sql_upper.contains("WHERE") {
        warnings.push("DELETE without WHERE clause will remove all rows".to_string());
        suggestions.push("Add appropriate WHERE clause to limit affected rows".to_string());
    }

    // Check for large DELETE operations
    if sql_upper.contains("WHERE") && !sql_upper.contains("LIMIT") {
        warnings.push("Large DELETE operations may lock tables for extended periods".to_string());
        suggestions.push("Consider batching DELETE operations or using TRUNCATE for full table".to_string());
    }
}

// ===== Helper Functions =====

fn calculate_plan_metadata(node: &ExecutionPlanNode) -> (f64, u32, u32) {
    let mut total_cost = node.cost;
    let mut node_count = 1u32;
    let mut max_depth = 0u32;

    for child in &node.children {
        let (child_cost, child_count, child_depth) = calculate_plan_metadata(child);
        total_cost = total_cost.max(child_cost);
        node_count += child_count;
        max_depth = max_depth.max(child_depth);
    }

    (total_cost, node_count, max_depth + 1)
}



fn classify_issue(warning: &str) -> IssueType {
    if warning.contains("scan") || warning.contains("Scan") {
        IssueType::FullTableScan
    } else if warning.contains("index") || warning.contains("Index") {
        IssueType::MissingIndex
    } else if warning.contains("join") || warning.contains("Join") {
        IssueType::InefficientJoin
    } else if warning.contains("sort") || warning.contains("Sort") {
        IssueType::SortOperation
    } else if warning.contains("statistics") || warning.contains("ANALYZE") {
        IssueType::MissingStatistics
    } else {
        IssueType::MissingIndex
    }
}

fn analyze_for_indexes(
    node: &ExecutionPlanNode,
    sql_statements: &mut Vec<String>,
    explanations: &mut Vec<String>,
) {
    // Skip analysis if this is SQL text analysis only
    if node.operation == "SQL_TEXT_ANALYSIS_ONLY" {
        explanations.push("Index analysis requires actual execution plan data".to_string());
        explanations.push("SQL text analysis cannot determine optimal index strategies".to_string());
        return;
    }

    if node.operation.contains("Seq Scan") {
        if let Some(table) = &node.node_details.table_name {
            if let Some(filter) = &node.node_details.filter {
                // Try to extract column from filter - but be more cautious
                let column = filter
                    .split('=')
                    .next()
                    .and_then(|s| s.split_whitespace().last())
                    .unwrap_or("unknown_column");

                // Only suggest index if we have reasonable confidence in the column name
                if column != "unknown_column" && !column.contains('(') && !column.contains(')') {
                    sql_statements.push(format!(
                        "-- Consider: CREATE INDEX idx_{}_{} ON {}({});",
                        table, column, table, column
                    ));
                    explanations.push(format!(
                        "Sequential scan on {} with filter '{}' - consider index on {}",
                        table, filter, column
                    ));
                    explanations.push("Note: Verify column selectivity and query frequency before creating index".to_string());
                } else {
                    explanations.push(format!(
                        "Sequential scan on {} with complex filter '{}' - manual analysis needed",
                        table, filter
                    ));
                }
            } else {
                explanations.push(format!(
                    "Sequential scan on {} without filter - check if index needed",
                    table
                ));
            }
        }
    }

    for child in &node.children {
        analyze_for_indexes(child, sql_statements, explanations);
    }
}

fn analyze_for_statistics(
    node: &ExecutionPlanNode,
    sql_statements: &mut Vec<String>,
    explanations: &mut Vec<String>,
) {
    // Skip analysis if this is SQL text analysis only
    if node.operation == "SQL_TEXT_ANALYSIS_ONLY" {
        explanations.push("Statistics analysis requires actual execution plan data".to_string());
        return;
    }

    if let Some(table) = &node.node_details.table_name {
        sql_statements.push(format!("-- Consider: ANALYZE {};", table));
        explanations.push(format!(
            "Table {} may benefit from updated statistics for better query planning",
            table
        ));
        explanations.push("Note: Run ANALYZE after significant data changes or periodically".to_string());
    }

    for child in &node.children {
        analyze_for_statistics(child, sql_statements, explanations);
    }
}

fn analyze_for_rewrite(
    node: &ExecutionPlanNode,
    sql_statements: &mut Vec<String>,
    explanations: &mut Vec<String>,
) {
    // Skip analysis if this is SQL text analysis only
    if node.operation == "SQL_TEXT_ANALYSIS_ONLY" {
        explanations.push("Query rewrite suggestions require actual execution plan data".to_string());
        return;
    }

    if node.operation.contains("Nested Loop") && node.cost > 1000.0 {
        sql_statements.push("-- Consider: Rewrite with explicit JOIN syntax and appropriate indexes".to_string());
        explanations.push("High-cost nested loop join detected - consider hash join or better indexes".to_string());
        explanations.push("Note: Requires analysis of data distribution and available indexes".to_string());
    }

    if node.operation.contains("Sort") && node.rows > 10000 {
        sql_statements.push("-- Consider: Add covering index to avoid sorting".to_string());
        explanations.push("Large sort operation detected - consider index on ORDER BY columns".to_string());
        explanations.push("Note: Index overhead vs. sort cost trade-off needs evaluation".to_string());
    }

    for child in &node.children {
        analyze_for_rewrite(child, sql_statements, explanations);
    }
}

// ===== Additional Types for Commands =====

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ThresholdOverrides {
    pub cost_threshold: Option<f64>,
    pub rows_threshold: Option<u64>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ParsedPlan {
    pub success: bool,
    pub plan_tree: ExecutionPlanNode,
    pub sql: Option<String>,
    pub parse_warnings: Vec<String>,
    pub parsed_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PlanAnalysis {
    pub score: i32,
    pub issues: Vec<PlanIssue>,
    pub recommendations: Vec<PlanRecommendation>,
    pub optimization_potential: String,
    pub estimated_improvement: Option<i32>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct PlanIssue {
    pub node_path: String,
    pub issue_type: IssueType,
    pub severity: AuditSeverity,
    pub description: String,
    pub affected_rows: u64,
    pub cost_impact: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum IssueType {
    FullTableScan,
    MissingIndex,
    InefficientJoin,
    NestedLoopWithIndex,
    HashJoinTooLarge,
    SortOperation,
    ExpensiveFunction,
    CartesianProduct,
    MissingStatistics,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum AuditSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct PlanRecommendation {
    pub priority: RecommendationPriority,
    pub action: String,
    pub description: String,
    pub sql_example: Option<String>,
    pub estimated_benefit: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SavePlanResult {
    pub success: bool,
    pub plan_id: i64,
    pub message: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct SavedPlan {
    pub id: i64,
    pub sql_id: Option<i64>,
    pub sql_text: Option<String>,
    pub source: String,
    pub created_at: String,
    pub total_cost: f64,
    pub node_count: u32,
    pub name: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SavedPlansResponse {
    pub plans: Vec<SavedPlan>,
    pub total: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DeleteResult {
    pub success: bool,
    pub deleted_plan_id: i64,
    pub message: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OptimizationSql {
    pub sql_statements: Vec<String>,
    pub explanations: Vec<String>,
    pub warnings: Vec<String>,
    pub confidence: OptimizationConfidence,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum OptimizationConfidence {
    High,
    Medium,
    Low,
}

// ===== ogexplain-core Commands =====

/// Parse execution plan using ogexplain-core (new parser, coexists with existing)
#[tauri::command]
pub async fn parse_explain_with_ogexplain(
    plan_text: String,
) -> Result<wdrprobe_core::models::execution_plan::ExecutionPlanNode, String> {
    let plan = ogexplain_core::parse(&plan_text)
        .map_err(|e| format!("Parse error: {}", e))?;
    Ok(ogexplain_adapter::convert_plan_node(&plan.root))
}

/// Diagnose execution plan using ogexplain-core 25 rules
#[tauri::command]
pub async fn diagnose_explain_plan(
    plan_text: String,
) -> Result<ogexplain_adapter::DiagnosticReportResponse, String> {
    let plan = ogexplain_core::parse(&plan_text)
        .map_err(|e| format!("Parse error: {}", e))?;
    let report = ogexplain_core::analyze(&plan);
    Ok(ogexplain_adapter::convert_diagnostic_report(&report, &plan))
}

/// Generate cost-actual deviation heatmap (requires EXPLAIN ANALYZE data)
#[tauri::command]
pub async fn get_explain_heatmap(
    plan_text: String,
) -> Result<Option<ogexplain_adapter::HeatmapData>, String> {
    let plan = ogexplain_core::parse(&plan_text)
        .map_err(|e| format!("Parse error: {}", e))?;
    Ok(ogexplain_core::heatmap(&plan).map(|h| ogexplain_adapter::convert_heatmap(&h)))
}

/// Generate resource waterfall chart (requires EXPLAIN ANALYZE data)
#[tauri::command]
pub async fn get_explain_waterfall(
    plan_text: String,
) -> Result<Option<ogexplain_adapter::WaterfallData>, String> {
    let plan = ogexplain_core::parse(&plan_text)
        .map_err(|e| format!("Parse error: {}", e))?;
    Ok(ogexplain_core::waterfall(&plan).map(|w| ogexplain_adapter::convert_waterfall(&w)))
}

/// List all 25 diagnostic rules with metadata
#[tauri::command]
pub async fn list_diagnostic_rules() -> Result<Vec<ogexplain_adapter::RuleInfo>, String> {
    // Static rule catalog - matches ogexplain-core's 25 rules
    Ok(vec![
        ogexplain_adapter::RuleInfo { rule_id: "SCAN-001".into(), category: "Scan".into(), title: "Large table full scan".into(), description: "Seq Scan on table exceeding row threshold".into(), severity: "Warning".into() },
        ogexplain_adapter::RuleInfo { rule_id: "SCAN-004".into(), category: "Scan".into(), title: "Filter without index".into(), description: "Filter removing many rows without index support".into(), severity: "Warning".into() },
        ogexplain_adapter::RuleInfo { rule_id: "JOIN-001".into(), category: "Join".into(), title: "Nested loop on large tables".into(), description: "Nested loop join with high row counts".into(), severity: "Warning".into() },
        ogexplain_adapter::RuleInfo { rule_id: "JOIN-002".into(), category: "Join".into(), title: "Hash join spill to disk".into(), description: "Hash join exceeding work_mem".into(), severity: "Critical".into() },
        ogexplain_adapter::RuleInfo { rule_id: "MEM-001".into(), category: "Memory".into(), title: "Sort spill to disk".into(), description: "External merge sort including VectorSort".into(), severity: "Critical".into() },
        ogexplain_adapter::RuleInfo { rule_id: "MEM-004".into(), category: "Memory".into(), title: "High peak memory".into(), description: "Highest-memory node in subtree".into(), severity: "Warning".into() },
        ogexplain_adapter::RuleInfo { rule_id: "SORT-003".into(), category: "Sort".into(), title: "Duplicate sort".into(), description: "Recursive subtree duplicate Sort Keys".into(), severity: "Info".into() },
        ogexplain_adapter::RuleInfo { rule_id: "NET-001".into(), category: "Network".into(), title: "Broadcast large data".into(), description: "Broadcasting excessive rows".into(), severity: "Warning".into() },
        ogexplain_adapter::RuleInfo { rule_id: "EST-001".into(), category: "Estimation".into(), title: "Severe row estimation error".into(), description: "Actual rows far exceed/fall below estimate".into(), severity: "Warning".into() },
        ogexplain_adapter::RuleInfo { rule_id: "EST-004".into(), category: "Estimation".into(), title: "Nested loop from underestimation".into(), description: "Nested Loop caused by row underestimation".into(), severity: "Warning".into() },
        ogexplain_adapter::RuleInfo { rule_id: "PUSH-001".into(), category: "Pushdown".into(), title: "Query not pushed down".into(), description: "FQS failure with signal accumulation".into(), severity: "Critical".into() },
        ogexplain_adapter::RuleInfo { rule_id: "PUSH-002".into(), category: "Pushdown".into(), title: "Multi-layer streaming".into(), description: "Streaming layer chain detected".into(), severity: "Warning".into() },
        ogexplain_adapter::RuleInfo { rule_id: "TYPE-001".into(), category: "Type".into(), title: "Implicit type coercion".into(), description: "TypeMismatch with fix suggestions".into(), severity: "Warning".into() },
        ogexplain_adapter::RuleInfo { rule_id: "TYPE-004".into(), category: "Type".into(), title: "LIKE with leading wildcard".into(), description: "Leading wildcard prevents index usage".into(), severity: "Info".into() },
        ogexplain_adapter::RuleInfo { rule_id: "VEC-001".into(), category: "Vectorization".into(), title: "Mixed row/vector engines".into(), description: "Row↔Vector adapter boundaries".into(), severity: "Info".into() },
        ogexplain_adapter::RuleInfo { rule_id: "GEN-001".into(), category: "General".into(), title: "Plan too deep".into(), description: "Plan depth exceeds threshold".into(), severity: "Info".into() },
        ogexplain_adapter::RuleInfo { rule_id: "SUBQ-001".into(), category: "Subquery".into(), title: "Subquery not pulled up".into(), description: "SubqueryScan nodes detected".into(), severity: "Warning".into() },
        ogexplain_adapter::RuleInfo { rule_id: "REW-001".into(), category: "Subquery".into(), title: "Large IN list not rewritten".into(), description: "IN lists with many values".into(), severity: "Info".into() },
        ogexplain_adapter::RuleInfo { rule_id: "SUBQ-006".into(), category: "Subquery".into(), title: "Correlated subquery self-update".into(), description: "Self-referencing correlated subqueries".into(), severity: "Critical".into() },
        ogexplain_adapter::RuleInfo { rule_id: "AGG-001".into(), category: "Aggregate".into(), title: "Group aggregate should be hash".into(), description: "Suggest Hash Aggregate for large GROUP BY".into(), severity: "Info".into() },
        ogexplain_adapter::RuleInfo { rule_id: "AGG-002".into(), category: "Aggregate".into(), title: "Hash aggregate spill to disk".into(), description: "Hash Aggregate exceeding work_mem".into(), severity: "Critical".into() },
        ogexplain_adapter::RuleInfo { rule_id: "SKEW-001".into(), category: "Distribution".into(), title: "Data skew detected".into(), description: "Uneven row distribution across datanodes".into(), severity: "Warning".into() },
        ogexplain_adapter::RuleInfo { rule_id: "DIST-001".into(), category: "Distribution".into(), title: "Distribution column mismatch".into(), description: "Join columns don't match distribution columns".into(), severity: "Warning".into() },
        ogexplain_adapter::RuleInfo { rule_id: "STATS-001".into(), category: "Statistics".into(), title: "Stats not collected".into(), description: "Tables with missing or stale statistics".into(), severity: "Warning".into() },
        ogexplain_adapter::RuleInfo { rule_id: "PART-001".into(), category: "Partition".into(), title: "Partition pruning failure".into(), description: "Full partition scan when pruning should help".into(), severity: "Warning".into() },
    ])
}
