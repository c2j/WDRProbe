// Test parsing real WDR file from opengauss_v1.html
#[cfg(test)]
mod parse_real_wdr_tests {
    use wdrprobe_desktop_lib::parsers::complete_wdr_parser::parse_complete_wdr_report;

    #[tokio::test]
    async fn test_parse_real_wdr_file() {
        let file_path = "/Users/c2j/Desktop/Desktop_Projects/DB/WDRProbe/example/opengauss_v1.html";
        let instance_name = "gaussdb".to_string();

        println!("\n=== Testing real WDR file parsing ===");
        println!("File: {}", file_path);

        // Parse the complete WDR report
        let result = parse_complete_wdr_report(file_path, instance_name);

        match result {
            Ok(complete_report) => {
                println!("\n✓ Report parsed successfully!");
                println!("  Report ID: {}", complete_report.report.id);
                println!("  Instance: {}", complete_report.report.instance_name);
                println!(
                    "  Generation Time: {}",
                    complete_report.report.generation_time
                );
                println!(
                    "  Snapshot: {} to {}",
                    complete_report.report.snapshot_start, complete_report.report.snapshot_end
                );

                println!("\n📊 Data Summary:");
                println!(
                    "  - Database stats: {}",
                    complete_report.database_stats.len()
                );
                println!("  - Top SQLs: {}", complete_report.top_sql.len());
                println!(
                    "  - Cache IO stats: {}",
                    complete_report.cache_io_stats.len()
                );
                println!("  - Object stats: {}", complete_report.object_stats.len());

                // Print database stats
                if !complete_report.database_stats.is_empty() {
                    println!("\n🏗️  Database Stats:");
                    for (i, db) in complete_report.database_stats.iter().take(3).enumerate() {
                        println!(
                            "    {}. {} - Backends: {}, Xact Commit: {}",
                            i + 1,
                            db.db_name,
                            db.backends,
                            db.xact_commit
                        );
                    }
                    if complete_report.database_stats.len() > 3 {
                        println!(
                            "    ... and {} more",
                            complete_report.database_stats.len() - 3
                        );
                    }
                }

                // Print SQL examples
                if !complete_report.top_sql.is_empty() {
                    println!(
                        "\n🔍 Top SQL Statements (showing first {}):",
                        complete_report.top_sql.len().min(5)
                    );
                    for (i, sql) in complete_report.top_sql.iter().take(5).enumerate() {
                        let sql_text = if sql.sql_text.len() > 80 {
                            format!("{}...", &sql.sql_text[..80])
                        } else {
                            sql.sql_text.clone()
                        };
                        println!(
                            "    {}. ID: {}",
                            i + 1,
                            sql.sql_id.as_ref().unwrap_or(&"N/A".to_string())
                        );
                        println!(
                            "       Calls: {}, Elapsed: {:.2}ms, CPU: {:.2}ms",
                            sql.executions,
                            sql.total_elapsed_time / 1000.0,
                            sql.cpu_time / 1000.0
                        );
                        println!("       SQL: {}", sql_text);
                        println!();
                    }
                } else {
                    println!("\n⚠️  No SQL statements found! This is a problem.");
                    assert!(false, "No SQL statements were parsed from the file");
                }

                // Print object stats
                if !complete_report.object_stats.is_empty() {
                    println!(
                        "📦 Object Stats (first {}):",
                        complete_report.object_stats.len().min(3)
                    );
                    for (i, obj) in complete_report.object_stats.iter().take(3).enumerate() {
                        println!(
                            "    {}. {}.{} ({})",
                            i + 1,
                            obj.schema_name,
                            obj.object_name,
                            obj.object_type
                        );
                    }
                }

                // Assertions
                assert!(
                    complete_report.database_stats.len() > 0,
                    "Should have database stats"
                );
                assert!(
                    complete_report.top_sql.len() > 0,
                    "Should have SQL statements"
                );
                assert!(
                    complete_report.object_stats.len() > 0,
                    "Should have object stats"
                );

                println!("\n✅ All assertions passed!");
            }
            Err(e) => {
                println!("\n❌ Failed to parse WDR file: {}", e);
                assert!(false, "Failed to parse WDR file: {}", e);
            }
        }
    }
}
