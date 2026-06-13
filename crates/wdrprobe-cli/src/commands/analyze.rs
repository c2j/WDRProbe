use wdrprobe_core::database::{get_connection, init_database, initialize_schema, DatabaseOperations};

use crate::output;
use crate::truncate_sql;

/// Run the analyze command: show detailed analysis of a WDR report
pub fn run(db: String, report_id: i64, format: String) -> anyhow::Result<()> {
    let fmt = output::OutputFormat::parse(&format);

    let pool = init_database(&db)
        .map_err(|e| anyhow::anyhow!("Failed to init database: {}", e))?;
    let conn = get_connection(&pool)
        .map_err(|e| anyhow::anyhow!("Failed to get connection: {}", e))?;
    initialize_schema(&conn)
        .map_err(|e| anyhow::anyhow!("Failed to initialize schema: {}", e))?;
    drop(conn);

    // Fetch report
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

    match fmt {
        output::OutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct AnalyzeOutput {
                report: wdrprobe_core::models::WdrReport,
                efficiency: Option<wdrprobe_core::models::EfficiencyMetrics>,
                load_profile: Option<wdrprobe_core::models::LoadProfile>,
                top_sqls: Vec<wdrprobe_core::models::TopSql>,
            }
            let out = AnalyzeOutput {
                report,
                efficiency,
                load_profile,
                top_sqls,
            };
            output::print_json(&out)?;
        }
        output::OutputFormat::Text => {
            // Report metadata
            println!("=== Report #{}: {} ===", report.id, report.instance_name);
            println!(
                "Snapshot: {}  ->  {}",
                report.snapshot_start, report.snapshot_end
            );
            println!("Status: {} | Created: {}", report.status, report.created_at);
            println!();

            // Efficiency metrics
            if let Some(eff) = &efficiency {
                println!("--- Efficiency Metrics ---");
                println!("  Buffer Hit Rate:           {:.2}%", eff.buffer_hit_percent);
                println!("  CPU Efficiency:             {:.2}%", eff.cpu_efficiency_percent);
                println!("  Soft Parse Rate:            {:.2}%", eff.soft_parse_rate_percent);
                println!(
                    "  Hard Parse Rate:            {:.2}%",
                    eff.hard_parse_rate_percent
                );
                println!(
                    "  Execution Efficiency:       {:.2}%",
                    eff.execution_efficiency_percent
                );
                println!();
            }

            // Load profile
            if let Some(lp) = &load_profile {
                println!("--- Load Profile ---");
                println!("  DB Time/sec:                {:.2}", lp.db_time_per_sec);
                println!("  CPU Time/sec:               {:.2}", lp.cpu_time_per_sec);
                println!("  IO Requests/sec:            {:.2}", lp.io_requests_per_sec);
                println!("  Total Transactions:         {}", lp.total_transactions);
                println!("  Commits/sec:                {:.2}", lp.commits_per_sec);
                println!("  Rollbacks/sec:              {:.2}", lp.rollbacks_per_sec);
                println!();
            }

            // Top SQL
            if top_sqls.is_empty() {
                println!("--- Top SQL ---");
                println!("  (none)");
            } else {
                println!("--- Top SQL (by elapsed time) ---");
                for (i, sql) in top_sqls.iter().enumerate() {
                    let sql_preview = truncate_sql(&sql.sql_text, 80);
                    println!(
                        "  #{}  SQL ID: {}   Elapsed: {:.0}ms  CPU: {:.0}ms  Execs: {}",
                        i + 1,
                        sql.sql_id.as_deref().unwrap_or("-"),
                        sql.total_elapsed_time,
                        sql.cpu_time,
                        sql.executions,
                    );
                    println!("      {}", sql_preview);
                }
                println!();
            }
        }
    }

    Ok(())
}
