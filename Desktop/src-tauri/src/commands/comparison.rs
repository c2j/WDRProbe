// Comparison commands
// IPC commands for WDR report comparison

use crate::database::DatabaseOperations;
use crate::database::DatabasePool;
use crate::models::comparison::*;
use crate::models::TopSql;
use std::collections::HashMap;
use tauri::State;

/// Get list of comparisons with pagination and sorting
#[tauri::command(rename_all = "camelCase")]
pub async fn get_comparisons(
    pool: State<'_, DatabasePool>,
    limit: Option<i32>,
    offset: Option<i32>,
    sort_by: Option<String>,
    sort_order: Option<String>,
) -> Result<ComparisonListResponse, String> {
    let pool_ref = pool.inner();

    let comparisons = DatabaseOperations::get_comparisons(
        pool_ref,
        limit,
        offset,
        sort_by.as_deref(),
        sort_order.as_deref(),
    )
    .map_err(|e| format!("Failed to retrieve comparisons: {}", e))?;

    let total = DatabaseOperations::count_comparisons(pool_ref)
        .map_err(|e| format!("Failed to count comparisons: {}", e))?;

    Ok(ComparisonListResponse { comparisons, total })
}

/// Get comparison summary with performance score and key findings
#[tauri::command(rename_all = "camelCase")]
pub async fn get_comparison_summary(
    pool: State<'_, DatabasePool>,
    comparison_id: i64,
) -> Result<ComparisonSummary, String> {
    let pool_ref = pool.inner();

    let summary = DatabaseOperations::get_comparison_summary(pool_ref, comparison_id)
        .map_err(|e| format!("Failed to retrieve comparison summary: {}", e))?
        .ok_or_else(|| format!("Comparison not found: {}", comparison_id))?;

    Ok(summary)
}

/// Get detailed comparison metrics by category
#[tauri::command(rename_all = "camelCase")]
pub async fn get_comparison_details(
    pool: State<'_, DatabasePool>,
    comparison_id: i64,
    category: String,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<ComparisonDetails, String> {
    let pool_ref = pool.inner();

    let details = DatabaseOperations::get_comparison_details(
        pool_ref,
        comparison_id,
        &category,
        limit,
        offset,
    )
    .map_err(|e| format!("Failed to retrieve comparison details: {}", e))?;

    Ok(details)
}

/// Create a new comparison between two WDR reports
#[tauri::command(rename_all = "camelCase")]
pub async fn create_comparison(
    pool: State<'_, DatabasePool>,
    source_report_id: i64,
    target_report_id: i64,
    comparison_type: Option<String>,
    _custom_name: Option<String>,
) -> Result<CreateComparisonResult, String> {
    let start = std::time::Instant::now();
    let pool_ref = pool.inner();

    // Validate both reports exist
    let source_report = DatabaseOperations::get_wdr_report(pool_ref, source_report_id)
        .map_err(|e| format!("Failed to retrieve source report: {}", e))?
        .ok_or_else(|| format!("Source report not found: {}", source_report_id))?;

    let target_report = DatabaseOperations::get_wdr_report(pool_ref, target_report_id)
        .map_err(|e| format!("Failed to retrieve target report: {}", e))?
        .ok_or_else(|| format!("Target report not found: {}", target_report_id))?;

    // Determine comparison type if not specified
    let comp_type = comparison_type.unwrap_or_else(|| {
        if source_report.instance_name == target_report.instance_name {
            "TimeBased".to_string()
        } else {
            "InstanceBased".to_string()
        }
    });

    // Perform the comparison
    let comparison_result = perform_comparison(pool_ref, source_report_id, target_report_id)?;

    // Save comparison to database
    let comparison_id = DatabaseOperations::create_comparison(
        pool_ref,
        source_report_id,
        target_report_id,
        &comp_type,
        &comparison_result.summary,
    )
    .map_err(|e| format!("Failed to save comparison: {}", e))?;

    // Save SQL comparison metrics
    for metric in &comparison_result.sql_metrics {
        DatabaseOperations::create_sql_comparison_metric(pool_ref, comparison_id, metric)
            .map_err(|e| format!("Failed to save SQL metric: {}", e))?;
    }

    let processing_time = start.elapsed().as_millis() as u64;

    Ok(CreateComparisonResult {
        success: true,
        comparison_id,
        message: format!(
            "Comparison created successfully with {} key findings",
            comparison_result.summary.key_findings.len()
        ),
        processing_time_ms: processing_time,
    })
}

/// Delete a comparison
#[tauri::command(rename_all = "camelCase")]
pub async fn delete_comparison(
    pool: State<'_, DatabasePool>,
    comparison_id: i64,
    confirm: bool,
) -> Result<DeleteResult, String> {
    if !confirm {
        return Err("Deletion must be confirmed".to_string());
    }

    let pool_ref = pool.inner();

    // Verify comparison exists
    let _comparison = DatabaseOperations::get_comparison_summary(pool_ref, comparison_id)
        .map_err(|e| format!("Failed to retrieve comparison: {}", e))?
        .ok_or_else(|| format!("Comparison not found: {}", comparison_id))?;

    // Delete the comparison
    DatabaseOperations::delete_comparison(pool_ref, comparison_id)
        .map_err(|e| format!("Failed to delete comparison: {}", e))?;

    Ok(DeleteResult {
        success: true,
        deleted_comparison_id: comparison_id,
        message: Some("Comparison deleted successfully".to_string()),
    })
}

/// Get chart data for comparison visualization
#[tauri::command(rename_all = "camelCase")]
pub async fn get_comparison_chart_data(
    pool: State<'_, DatabasePool>,
    comparison_id: i64,
    chart_type: String,
) -> Result<ChartData, String> {
    let pool_ref = pool.inner();

    let chart_data =
        DatabaseOperations::get_comparison_chart_data(pool_ref, comparison_id, &chart_type)
            .map_err(|e| format!("Failed to retrieve chart data: {}", e))?;

    Ok(chart_data)
}

/// Calculate performance score from metric changes
pub fn calculate_performance_score(changes: &HashMap<String, f64>) -> i32 {
    // Weights for different metric categories
    let weights: HashMap<&str, f64> = [
        ("sql_elapsed_time", 0.35),
        ("sql_cpu_time", 0.25),
        ("sql_io_time", 0.15),
        ("buffer_hit_ratio", 0.10),
        ("disk_reads", 0.10),
        ("executions", 0.05),
    ]
    .iter()
    .cloned()
    .collect();

    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;

    for (metric_name, change) in changes {
        if let Some(weight) = weights.get(metric_name.as_str()) {
            // For time-based metrics, negative change is improvement (good)
            // For ratio-based metrics, positive change is improvement (good)
            let normalized_change = if metric_name.contains("time") || metric_name.contains("reads")
            {
                -change // Flip sign so improvement is positive
            } else {
                *change
            };

            weighted_sum += normalized_change * weight;
            total_weight += weight;
        }
    }

    if total_weight == 0.0 {
        return 0;
    }

    let score = weighted_sum / total_weight;
    score.max(-100.0).min(100.0) as i32
}

/// Determine status from performance score
fn get_status_from_score(score: i32) -> String {
    if score >= 15 {
        "Improved".to_string()
    } else if score <= -15 {
        "Degraded".to_string()
    } else {
        "NoSignificantChange".to_string()
    }
}

/// Generate key findings from comparison metrics
fn generate_key_findings(
    sql_metrics: &[SqlComparisonMetric],
    _performance_score: i32,
) -> Vec<KeyFinding> {
    let mut findings = Vec::new();

    // Analyze SQL metrics for significant changes
    for metric in sql_metrics {
        // Check elapsed time change
        if let Some(change) = metric.change_percentages.elapsed_time {
            let severity = if change.abs() >= 50.0 {
                "Critical"
            } else if change.abs() >= 20.0 {
                "Warning"
            } else {
                "Info"
            };

            if change.abs() >= 15.0 {
                findings.push(KeyFinding {
                    category: "Sql".to_string(),
                    metric: "Elapsed Time".to_string(),
                    change_percent: change,
                    severity: severity.to_string(),
                    description: format!("SQL elapsed time changed by {:.1}%", change),
                });
            }
        }

        // Check CPU time change
        if let Some(change) = metric.change_percentages.cpu_time {
            if change.abs() >= 20.0 {
                let severity = if change.abs() >= 50.0 {
                    "Critical"
                } else {
                    "Warning"
                };
                findings.push(KeyFinding {
                    category: "Sql".to_string(),
                    metric: "CPU Time".to_string(),
                    change_percent: change,
                    severity: severity.to_string(),
                    description: format!("SQL CPU time changed by {:.1}%", change),
                });
            }
        }

        // Check disk reads change
        if let Some(change) = metric.change_percentages.disk_reads {
            if change.abs() >= 30.0 {
                let severity = if change.abs() >= 60.0 {
                    "Critical"
                } else {
                    "Warning"
                };
                findings.push(KeyFinding {
                    category: "Sql".to_string(),
                    metric: "Disk Reads".to_string(),
                    change_percent: change,
                    severity: severity.to_string(),
                    description: format!("Disk reads changed by {:.1}%", change),
                });
            }
        }
    }

    // Sort findings by severity and magnitude
    findings.sort_by(|a, b| {
        let severity_order = |s: &str| -> i32 {
            match s {
                "Critical" => 0,
                "Warning" => 1,
                "Info" => 2,
                _ => 3,
            }
        };
        severity_order(&a.severity)
            .cmp(&severity_order(&b.severity))
            .then_with(|| {
                b.change_percent
                    .abs()
                    .partial_cmp(&a.change_percent.abs())
                    .unwrap()
            })
    });

    // Limit to top 10 findings
    findings.truncate(10);

    findings
}

/// Result of comparison processing
struct ComparisonResult {
    summary: ComparisonSummary,
    sql_metrics: Vec<SqlComparisonMetric>,
}

/// Perform comparison between two WDR reports
fn perform_comparison(
    pool: &DatabasePool,
    source_report_id: i64,
    target_report_id: i64,
) -> Result<ComparisonResult, String> {
    // Get SQL metrics from both reports
    let source_sqls = DatabaseOperations::get_top_sqls_by_report(pool, source_report_id)
        .map_err(|e| format!("Failed to get source SQLs: {}", e))?;

    let target_sqls = DatabaseOperations::get_top_sqls_by_report(pool, target_report_id)
        .map_err(|e| format!("Failed to get target SQLs: {}", e))?;

    println!(
        "[Comparison] Source report has {} SQLs, Target report has {} SQLs",
        source_sqls.len(),
        target_sqls.len()
    );

    // Create hash maps for SQL matching
    let source_map: HashMap<String, &TopSql> = source_sqls
        .iter()
        .map(|sql| (sql.sql_text_hash(), sql))
        .collect();

    let target_map: HashMap<String, &TopSql> = target_sqls
        .iter()
        .map(|sql| (sql.sql_text_hash(), sql))
        .collect();

    let mut sql_metrics = Vec::new();
    let mut all_changes = HashMap::new();

    // Compare common SQLs
    let mut common_count = 0;
    for (hash, source_sql) in &source_map {
        if let Some(target_sql) = target_map.get(hash) {
            common_count += 1;
            let metric = compare_sql_metrics(source_sql, target_sql);
            all_changes.insert(
                format!("sql_elapsed_time_{}", hash),
                metric.change_percentages.elapsed_time.unwrap_or(0.0),
            );
            all_changes.insert(
                format!("sql_cpu_time_{}", hash),
                metric.change_percentages.cpu_time.unwrap_or(0.0),
            );
            sql_metrics.push(metric);
        }
    }

    println!("[Comparison] Found {} common SQLs to compare", common_count);

    // Calculate aggregated changes for performance score
    let mut aggregated_changes = HashMap::new();
    if !sql_metrics.is_empty() {
        let avg_elapsed_change: f64 = sql_metrics
            .iter()
            .filter_map(|m| m.change_percentages.elapsed_time)
            .sum::<f64>()
            / sql_metrics.len() as f64;
        let avg_cpu_change: f64 = sql_metrics
            .iter()
            .filter_map(|m| m.change_percentages.cpu_time)
            .sum::<f64>()
            / sql_metrics.len() as f64;
        let avg_disk_change: f64 = sql_metrics
            .iter()
            .filter_map(|m| m.change_percentages.disk_reads)
            .sum::<f64>()
            / sql_metrics.len() as f64;

        aggregated_changes.insert("sql_elapsed_time".to_string(), avg_elapsed_change);
        aggregated_changes.insert("sql_cpu_time".to_string(), avg_cpu_change);
        aggregated_changes.insert("disk_reads".to_string(), avg_disk_change);
    }

    // Calculate performance score
    let performance_score = calculate_performance_score(&aggregated_changes);
    let status = get_status_from_score(performance_score);

    // Generate key findings
    let key_findings = generate_key_findings(&sql_metrics, performance_score);

    // Generate conclusion
    let conclusion = if performance_score >= 15 {
        format!(
            "Performance improved by {} points with {} significant changes",
            performance_score,
            key_findings.len()
        )
    } else if performance_score <= -15 {
        format!(
            "Performance degraded by {} points with {} significant changes",
            performance_score.abs(),
            key_findings.len()
        )
    } else {
        format!(
            "Performance remained stable with {} notable changes",
            key_findings.len()
        )
    };

    let now = chrono::Utc::now().to_rfc3339();

    Ok(ComparisonResult {
        summary: ComparisonSummary {
            performance_score_change: performance_score,
            status,
            conclusion,
            key_findings,
            created_at: now.clone(),
        },
        sql_metrics,
    })
}

/// Compare metrics between two SQL entries
fn compare_sql_metrics(source: &TopSql, target: &TopSql) -> SqlComparisonMetric {
    let calc_change = |s: f64, t: f64| -> Option<f64> {
        if s == 0.0 {
            return None;
        }
        Some(((t - s) / s) * 100.0)
    };

    let source_metrics = SqlMetrics {
        executions: source.executions,
        total_elapsed_time: source.total_elapsed_time,
        cpu_time: source.cpu_time,
        io_time: source.io_time,
        buffer_gets: source.buffer_gets,
        disk_reads: source.disk_reads,
        rows_processed: source.rows_processed,
    };

    let target_metrics = SqlMetrics {
        executions: target.executions,
        total_elapsed_time: target.total_elapsed_time,
        cpu_time: target.cpu_time,
        io_time: target.io_time,
        buffer_gets: target.buffer_gets,
        disk_reads: target.disk_reads,
        rows_processed: target.rows_processed,
    };

    // Convert sql_id from Option<String> to Option<i64>
    let sql_id_i64 = source.sql_id.as_ref().and_then(|s| s.parse::<i64>().ok());

    SqlComparisonMetric {
        sql_id: sql_id_i64,
        sql_text_hash: source.sql_text_hash(),
        source_metrics,
        target_metrics,
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
