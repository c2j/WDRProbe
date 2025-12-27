// Reports commands
// IPC commands for WDR report import, retrieval, and management

use crate::database::DatabaseOperations;
use crate::database::DatabasePool;
use crate::models::{
    EfficiencyMetrics, LoadProfile, WdrReport, WdrReportDetail, WdrReportListResponse,
};
use crate::parsers::complete_wdr_parser::parse_complete_wdr_report;
use tauri::State;

/// Import WDR report from file
#[tauri::command(rename_all = "camelCase")]
pub async fn import_wdr_report(
    pool: State<'_, DatabasePool>,
    file_path: String,
    instance_name: String,
    _description: Option<String>, // Reserved for future use
) -> Result<WdrReport, String> {
    let pool_ref = pool.inner();

    // Validate file exists
    let path = std::path::Path::new(&file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    // Parse complete WDR report with all sections
    let complete_report = parse_complete_wdr_report(&file_path, instance_name)
        .map_err(|e| format!("Failed to parse WDR report: {}", e))?;

    let report = complete_report.report;

    // Save to database
    let report_id = DatabaseOperations::create_wdr_report(pool_ref, &report)
        .map_err(|e| format!("Failed to save report: {}", e))?;

    // Update report_id in dependent tables
    println!(
        "Backend: Saving all parsed data for report {}...",
        report_id
    );

    // Save database stats
    for (i, mut stat) in complete_report.database_stats.into_iter().enumerate() {
        stat.report_id = report_id;
        match DatabaseOperations::create_database_stats(pool_ref, &stat) {
            Ok(id) => println!("Backend: Saved database stat {} with ID: {}", i, id),
            Err(e) => println!("Backend: Failed to save database stat {}: {}", i, e),
        }
    }

    // Save object stats
    for (i, mut obj) in complete_report.object_stats.into_iter().enumerate() {
        obj.report_id = report_id;
        match DatabaseOperations::create_object_stats(pool_ref, &obj) {
            Ok(id) => println!("Backend: Saved object stat {} with ID: {}", i, id),
            Err(e) => println!("Backend: Failed to save object stat {}: {}", i, e),
        }
    }

    // Save cache IO stats
    for (i, mut io) in complete_report.cache_io_stats.into_iter().enumerate() {
        io.report_id = report_id;
        match DatabaseOperations::create_cache_io_stats(pool_ref, &io) {
            Ok(id) => println!("Backend: Saved cache IO stat {} with ID: {}", i, id),
            Err(e) => println!("Backend: Failed to save cache IO stat {}: {}", i, e),
        }
    }

    // Save top SQLs
    for (i, mut sql) in complete_report.top_sql.into_iter().enumerate() {
        sql.report_id = report_id;
        match DatabaseOperations::create_top_sql(pool_ref, &sql) {
            Ok(id) => println!("Backend: Saved top SQL {} with ID: {}", i, id),
            Err(e) => println!("Backend: Failed to save top SQL {}: {}", i, e),
        }
    }

    // Save efficiency metrics
    let mut efficiency = complete_report.efficiency;
    efficiency.report_id = report_id;
    match DatabaseOperations::create_efficiency_metrics(pool_ref, &efficiency) {
        Ok(id) => println!("Backend: Saved efficiency metrics with ID: {}", id),
        Err(e) => println!("Backend: Failed to save efficiency metrics: {}", e),
    }

    // Save load profile
    let mut load_profile = complete_report.load_profile;
    load_profile.report_id = report_id;
    match DatabaseOperations::create_load_profile(pool_ref, &load_profile) {
        Ok(id) => println!("Backend: Saved load profile with ID: {}", id),
        Err(e) => println!("Backend: Failed to save load profile: {}", e),
    }

    // Retrieve the saved report with ID
    let saved_report = DatabaseOperations::get_wdr_report(pool_ref, report_id)
        .map_err(|e| format!("Failed to retrieve report: {}", e))?
        .ok_or_else(|| "Report was not saved correctly".to_string())?;

    println!(
        "Backend: Successfully imported report {} with all sections",
        report_id
    );

    Ok(saved_report)
}

/// Get list of WDR reports with pagination
#[tauri::command]
pub async fn get_wdr_reports(
    pool: State<'_, DatabasePool>,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<WdrReportListResponse, String> {
    let pool_ref = pool.inner();

    let reports = DatabaseOperations::list_wdr_reports(pool_ref, limit, offset)
        .map_err(|e| format!("Failed to retrieve reports: {}", e))?;

    let total = DatabaseOperations::list_wdr_reports(pool_ref, None, None)
        .map_err(|e| format!("Failed to count reports: {}", e))?
        .len() as i64;

    Ok(WdrReportListResponse { reports, total })
}

/// Get detailed WDR report with SQLs and metrics
#[tauri::command(rename_all = "camelCase")]
pub async fn get_wdr_report_detail(
    pool: State<'_, DatabasePool>,
    report_id: i64,
) -> Result<WdrReportDetail, String> {
    let pool_ref = pool.inner();

    println!("Backend: Querying report detail for ID: {}", report_id);

    // Get report
    let report = DatabaseOperations::get_wdr_report(pool_ref, report_id)
        .map_err(|e| format!("Failed to retrieve report: {}", e))?
        .ok_or_else(|| format!("Report not found: {}", report_id))?;

    println!("Backend: Found report: {:?}", report);

    // Get all related data for the report
    let sqls = DatabaseOperations::get_top_sqls_by_report(pool_ref, report_id)
        .map_err(|e| format!("Failed to retrieve SQLs: {}", e))?;

    let object_stats = DatabaseOperations::get_object_stats(pool_ref, report_id)
        .map_err(|e| format!("Failed to retrieve object stats: {}", e))?;

    let cache_io_stats = DatabaseOperations::get_cache_io_stats(pool_ref, report_id)
        .map_err(|e| format!("Failed to retrieve cache IO stats: {}", e))?;

    let database_stats = DatabaseOperations::get_database_stats(pool_ref, report_id)
        .map_err(|e| format!("Failed to retrieve database stats: {}", e))?;

    // Get efficiency metrics and load profile from database
    let efficiency = DatabaseOperations::get_efficiency_metrics(pool_ref, report_id)
        .map_err(|e| format!("Failed to retrieve efficiency metrics: {}", e))?
        .unwrap_or_else(|| EfficiencyMetrics {
            report_id: report.id,
            buffer_hit_percent: 0.0,
            cpu_efficiency_percent: 0.0,
            soft_parse_rate_percent: 0.0,
            hard_parse_rate_percent: 0.0,
            execution_efficiency_percent: 0.0,
        });

    let load_profile = DatabaseOperations::get_load_profile(pool_ref, report_id)
        .map_err(|e| format!("Failed to retrieve load profile: {}", e))?
        .unwrap_or_else(|| LoadProfile {
            report_id: report.id,
            db_time_per_sec: 0.0,
            cpu_time_per_sec: 0.0,
            io_requests_per_sec: 0.0,
            total_transactions: 0,
            commits_per_sec: 0.0,
            rollbacks_per_sec: 0.0,
        });

    println!(
        "Backend: Retrieved {} SQLs, {} object stats, {} cache IO stats, {} database stats",
        sqls.len(),
        object_stats.len(),
        cache_io_stats.len(),
        database_stats.len()
    );
    println!(
        "Backend: Efficiency metrics - Buffer Hit: {:.2}%, CPU: {:.2}%",
        efficiency.buffer_hit_percent, efficiency.cpu_efficiency_percent
    );
    println!(
        "Backend: Load Profile - DB Time: {:.2}/s, CPU Time: {:.2}/s",
        load_profile.db_time_per_sec, load_profile.cpu_time_per_sec
    );

    // Return the data
    let detail = WdrReportDetail {
        id: report.id,
        instance_name: report.instance_name,
        generation_time: report.generation_time,
        snapshot_start: report.snapshot_start,
        snapshot_end: report.snapshot_end,
        status: report.status,
        efficiency,
        load_profile,
        top_sql: sqls,
        object_stats,
    };

    println!(
        "Backend: Returning detail with {} SQLs and {} object stats",
        detail.top_sql.len(),
        detail.object_stats.len()
    );
    Ok(detail)
}

/// Delete WDR report
#[tauri::command]
pub async fn delete_wdr_report(
    pool: State<'_, DatabasePool>,
    report_id: i64,
) -> Result<(), String> {
    let pool_ref = pool.inner();

    // Verify report exists
    let _report = DatabaseOperations::get_wdr_report(pool_ref, report_id)
        .map_err(|e| format!("Failed to retrieve report: {}", e))?
        .ok_or_else(|| format!("Report not found: {}", report_id))?;

    // Delete the report (SQLs will be cascade deleted via foreign key)
    DatabaseOperations::delete_wdr_report(pool_ref, report_id)
        .map_err(|e| format!("Failed to delete report: {}", e))?;

    Ok(())
}

/// Get Hot SQLs across all reports
#[tauri::command]
pub async fn get_hot_sqls(
    pool: State<'_, DatabasePool>,
    limit: Option<i32>,
) -> Result<Vec<crate::models::TopSql>, String> {
    let pool_ref = pool.inner();

    let sqls = DatabaseOperations::get_hot_sqls(pool_ref, limit)
        .map_err(|e| format!("Failed to retrieve hot SQLs: {}", e))?;

    Ok(sqls)
}
