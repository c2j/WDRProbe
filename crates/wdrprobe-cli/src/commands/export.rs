use std::fs::File;
use std::io::Write;
use wdrprobe_core::database::{get_connection, init_database, initialize_schema, DatabaseOperations};

use crate::output;

/// Run the export command: export a WDR report to JSON or CSV
pub fn run(db: String, report_id: i64, format: &str, output: Option<String>) -> anyhow::Result<()> {
    let fmt = output::OutputFormat::parse(format);

    let pool = init_database(&db)
        .map_err(|e| anyhow::anyhow!("Failed to init database: {}", e))?;
    let conn = get_connection(&pool)
        .map_err(|e| anyhow::anyhow!("Failed to get connection: {}", e))?;
    initialize_schema(&conn)
        .map_err(|e| anyhow::anyhow!("Failed to initialize schema: {}", e))?;
    drop(conn);

    // Fetch all report data
    let report = pool
        .get_wdr_report(report_id)
        .map_err(|e| anyhow::anyhow!("Failed to get report: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("Report #{} not found", report_id))?;

    let efficiency = pool
        .get_efficiency_metrics(report_id)
        .map_err(|e| anyhow::anyhow!("Failed to get efficiency metrics: {}", e))?;

    let load_profile = pool
        .get_load_profile(report_id)
        .map_err(|e| anyhow::anyhow!("Failed to get load profile: {}", e))?;

    let top_sqls = pool
        .get_top_sqls_by_report(report_id)
        .map_err(|e| anyhow::anyhow!("Failed to get top SQLs: {}", e))?;

    let database_stats = pool
        .get_database_stats(report_id)
        .map_err(|e| anyhow::anyhow!("Failed to get database stats: {}", e))?;

    let cache_io_stats = pool
        .get_cache_io_stats(report_id)
        .map_err(|e| anyhow::anyhow!("Failed to get cache I/O stats: {}", e))?;

    let object_stats = pool
        .get_object_stats(report_id)
        .map_err(|e| anyhow::anyhow!("Failed to get object stats: {}", e))?;

    match fmt {
        output::OutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct ExportData {
                report: wdrprobe_core::models::WdrReport,
                efficiency: Option<wdrprobe_core::models::EfficiencyMetrics>,
                load_profile: Option<wdrprobe_core::models::LoadProfile>,
                top_sqls: Vec<wdrprobe_core::models::TopSql>,
                database_stats: Vec<wdrprobe_core::models::DatabaseStats>,
                cache_io_stats: Vec<wdrprobe_core::models::CacheIoStats>,
                object_stats: Vec<wdrprobe_core::models::ObjectStats>,
            }

            let data = ExportData {
                report,
                efficiency,
                load_profile,
                top_sqls,
                database_stats,
                cache_io_stats,
                object_stats,
            };

            let json = serde_json::to_string_pretty(&data)?;
            write_output(json, output)?;
        }
        output::OutputFormat::Text => {
            // Text output defaults to CSV-like format for top SQLs
            let mut csv = String::from(
                "Rank,SQL_ID,SQL_Text,Executions,Total_Elapsed_Time_ms,CPU_Time_ms,IO_Time_ms,Buffer_Gets,Disk_Reads,Rows_Processed\n",
            );
            for (i, sql) in top_sqls.iter().enumerate() {
                let sql_text = escape_csv(&sql.sql_text);
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{}\n",
                    i + 1,
                    sql.sql_id.as_deref().unwrap_or("-"),
                    sql_text,
                    sql.executions,
                    sql.total_elapsed_time,
                    sql.cpu_time,
                    sql.io_time,
                    sql.buffer_gets,
                    sql.disk_reads,
                    sql.rows_processed,
                ));
            }
            write_output(csv, output)?;
        }
    }

    Ok(())
}

/// Write content to a file or stdout
fn write_output(content: String, path: Option<String>) -> anyhow::Result<()> {
    match path {
        Some(p) => {
            let mut file = File::create(&p)?;
            file.write_all(content.as_bytes())?;
            println!("Exported to {}", p);
        }
        None => {
            print!("{}", content);
        }
    }
    Ok(())
}

/// Escape a string for CSV (wrap in quotes if contains comma, quote, or newline)
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
