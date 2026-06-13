use anyhow::Context;
use wdrprobe_core::database::{get_connection, init_database, initialize_schema, DatabaseOperations};
use wdrprobe_core::database::schema::{initialize_default_thresholds, initialize_sample_audit_issues};
use wdrprobe_core::parsers::complete_wdr_parser::parse_complete_wdr_report;

/// Run the import command: parse a WDR HTML file and store it in the database
pub fn run(file: String, db: String, instance: String) -> anyhow::Result<()> {
    // 1. Initialize database
    let pool = init_database(&db)
        .map_err(|e| anyhow::anyhow!("Failed to init database: {}", e))?;
    let conn = get_connection(&pool)
        .map_err(|e| anyhow::anyhow!("Failed to get connection: {}", e))?;
    initialize_schema(&conn)
        .map_err(|e| anyhow::anyhow!("Failed to initialize schema: {}", e))?;
    initialize_default_thresholds(&conn)
        .map_err(|e| anyhow::anyhow!("Failed to initialize thresholds: {}", e))?;
    initialize_sample_audit_issues(&conn)
        .map_err(|e| anyhow::anyhow!("Failed to initialize audit issues: {}", e))?;

    // Drop connection explicitly so pool can be used for operations
    drop(conn);

    // 2. Parse the WDR HTML file
    eprintln!("Parsing WDR report: {}", file);
    let parsed = parse_complete_wdr_report(&file, instance)
        .map_err(|e| anyhow::anyhow!("Failed to parse WDR report: {}", e))?;

    // 3. Store the report metadata
    let report_id = pool
        .create_wdr_report(&parsed.report)
        .context("Failed to create WDR report")?;
    eprintln!("Created WDR report with ID: {}", report_id);

    // 4. Store efficiency metrics
    let mut eff = parsed.efficiency.clone();
    eff.report_id = report_id;
    pool.create_efficiency_metrics(&eff)
        .context("Failed to store efficiency metrics")?;

    // 5. Store load profile
    let mut lp = parsed.load_profile.clone();
    lp.report_id = report_id;
    pool.create_load_profile(&lp)
        .context("Failed to store load profile")?;

    // 6. Store database stats
    for mut s in parsed.database_stats.clone() {
        s.report_id = report_id;
        pool.create_database_stats(&s)
            .context("Failed to store database stats")?;
    }

    // 7. Store top SQLs
    for mut s in parsed.top_sql.clone() {
        s.report_id = report_id;
        pool.create_top_sql(&s)
            .context("Failed to store top SQL")?;
    }

    // 8. Store cache I/O stats
    for mut s in parsed.cache_io_stats.clone() {
        s.report_id = report_id;
        pool.create_cache_io_stats(&s)
            .context("Failed to store cache I/O stats")?;
    }

    // 9. Store object stats
    for mut s in parsed.object_stats.clone() {
        s.report_id = report_id;
        pool.create_object_stats(&s)
            .context("Failed to store object stats")?;
    }

    println!(
        "Successfully imported WDR report #{} ({} databases, {} SQLs, {} cache I/O, {} objects)",
        report_id,
        parsed.database_stats.len(),
        parsed.top_sql.len(),
        parsed.cache_io_stats.len(),
        parsed.object_stats.len(),
    );

    Ok(())
}
