// Execution plan commands
// IPC commands for SQL execution plan visualization

use crate::database::{DatabaseOperations, DatabasePool};
use crate::models::execution_plan::*;
use crate::parsers::sql_parser::*;
use std::sync::Arc;

/// Get hot SQL queries from WDR reports
#[tauri::command]
pub async fn get_wdr_hot_sqls(
    report_id: Option<i64>,
    limit: Option<i32>,
    sort_by: Option<String>,
    pool: tauri::State<'_, Arc<DatabasePool>>,
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
    pool: tauri::State<'_, Arc<DatabasePool>>,
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
                // Generate a mock plan for demonstration
                generate_mock_plan_for_sql(&sql.sql_text)
            }
        }
        "UserProvided" => {
            let sql = sql_text.ok_or("sql_text required for UserProvided source")?;

            // Validate SQL syntax
            validate_sql_syntax(&sql)?;

            // Generate a mock plan for user-provided SQL
            generate_mock_plan_for_sql(&sql)
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

    Ok(ParsedPlan {
        success: true,
        plan_tree,
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
            estimated_benefit: "5-15% improvement".to_string(),
        });
    }

    // Determine optimization potential
    let optimization_potential = if score >= 80 {
        "Low - Minor optimizations available".to_string()
    } else if score >= 50 {
        "Medium - Several optimizations possible".to_string()
    } else {
        "High - Significant optimization opportunities".to_string()
    };

    let estimated_improvement = if score < 50 {
        Some(30 + (80 - score) as i32)
    } else if score < 80 {
        Some(10 + (80 - score) / 2)
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
    pool: tauri::State<'_, Arc<DatabasePool>>,
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

    let plan = crate::models::SqlExecutionPlan {
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
    pool: tauri::State<'_, Arc<DatabasePool>>,
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
    pool: tauri::State<'_, Arc<DatabasePool>>,
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
    pool: tauri::State<'_, Arc<DatabasePool>>,
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
    let warnings = Vec::new();

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

fn generate_mock_plan_for_sql(sql: &str) -> ExecutionPlanNode {
    // Extract table names for a more realistic mock
    let tables = extract_table_names(sql);
    let table_name = tables
        .first()
        .unwrap_or(&"unknown_table".to_string())
        .clone();

    ExecutionPlanNode {
        operation: "Seq Scan".to_string(),
        cost: 1000.0,
        rows: 50000,
        actual_rows: None,
        actual_time: None,
        width: Some(100),
        children: vec![],
        node_details: PlanNodeDetails {
            output: None,
            filter: Some("id = ?".to_string()),
            buffers: None,
            join_type: None,
            hash_keys: None,
            index_name: None,
            table_name: Some(table_name),
        },
        warnings: vec!["This is a mock execution plan".to_string()],
        suggestions: vec!["Consider creating an index on filter columns".to_string()],
    }
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
    if node.operation.contains("Seq Scan") {
        if let Some(table) = &node.node_details.table_name {
            if let Some(filter) = &node.node_details.filter {
                // Try to extract column from filter
                let column = filter
                    .split('=')
                    .next()
                    .and_then(|s| s.split_whitespace().last())
                    .unwrap_or("id");

                sql_statements.push(format!(
                    "CREATE INDEX idx_{}_{} ON {}({});",
                    table, column, table, column
                ));
                explanations.push(format!(
                    "Create index on {}({}) to improve filter performance",
                    table, column
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
    if let Some(table) = &node.node_details.table_name {
        sql_statements.push(format!("ANALYZE {};", table));
        explanations.push(format!(
            "Update statistics for table {} to improve query planning",
            table
        ));
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
    if node.operation.contains("Nested Loop") && node.cost > 1000.0 {
        sql_statements.push("-- Consider using INNER JOIN with appropriate indexes".to_string());
        explanations
            .push("Rewrite query to use explicit JOIN syntax with proper indexes".to_string());
    }

    if node.operation.contains("Sort") && node.rows > 10000 {
        sql_statements.push("-- Consider using index to avoid sorting".to_string());
        explanations.push("Add index covering ORDER BY columns to eliminate sort".to_string());
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
