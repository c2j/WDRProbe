use wdrprobe_core::database::{get_connection, init_database, initialize_schema, DatabaseOperations};

use crate::output;

/// Run the list command: list all imported WDR reports
pub fn run(db: String, format: String, limit: Option<i32>) -> anyhow::Result<()> {
    let fmt = output::OutputFormat::parse(&format);

    let pool = init_database(&db)
        .map_err(|e| anyhow::anyhow!("Failed to init database: {}", e))?;
    let conn = get_connection(&pool)
        .map_err(|e| anyhow::anyhow!("Failed to get connection: {}", e))?;
    initialize_schema(&conn)
        .map_err(|e| anyhow::anyhow!("Failed to initialize schema: {}", e))?;
    drop(conn);

    let reports = pool
        .list_wdr_reports(limit, None)
        .map_err(|e| anyhow::anyhow!("Failed to list reports: {}", e))?;

    match fmt {
        output::OutputFormat::Json => {
            output::print_json(&reports)?;
        }
        output::OutputFormat::Text => {
            if reports.is_empty() {
                println!("No WDR reports found in database.");
                return Ok(());
            }

            let mut table = output::Table::new(&["ID", "Instance", "Snapshot Start", "Snapshot End", "Status"]);
            for r in &reports {
                table.add_row(&[
                    r.id.to_string(),
                    r.instance_name.clone(),
                    r.snapshot_start.clone(),
                    r.snapshot_end.clone(),
                    r.status.clone(),
                ]);
            }
            table.print();
        }
    }

    Ok(())
}
