// Complete WDR Report Parser
// Parses all sections of WDR reports including Database Stats, Cache IO Stats, and Object Stats

use crate::models::report::{
    CacheIoStats, CompleteWdrReport, DatabaseStats, EfficiencyMetrics, LoadProfile, ObjectStats,
    TopSql, WdrReport,
};
use crate::utils::error::WdrProbeError;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::Read;

/// Parse complete WDR report with all sections
pub fn parse_complete_wdr_report(
    file_path: &str,
    instance_name: String,
) -> Result<CompleteWdrReport, WdrProbeError> {
    println!(
        "CompleteWDRParser: Starting comprehensive parse of {}",
        file_path
    );

    let mut file = File::open(file_path).map_err(|e| WdrProbeError::Io(e))?;
    let mut html_content = String::new();
    file.read_to_string(&mut html_content)
        .map_err(|e| WdrProbeError::Io(e))?;

    let document = Html::parse_document(&html_content);

    // Parse all sections
    let report = parse_report_metadata(&document, &instance_name)?;
    let efficiency = parse_efficiency_metrics(&document, report.id);
    let load_profile = parse_load_profile(&document, report.id);
    let database_stats = parse_database_stats(&document, report.id);
    let top_sql = parse_top_sqls(&document, report.id);
    let cache_io_stats = parse_cache_io_stats(&document, report.id);
    let object_stats = parse_object_stats(&document, report.id);

    println!("CompleteWDRParser: Successfully parsed all sections");
    println!("  - {} databases", database_stats.len());
    println!("  - {} SQLs", top_sql.len());
    println!("  - {} cache IO stats", cache_io_stats.len());
    println!("  - {} object stats", object_stats.len());

    Ok(CompleteWdrReport {
        report,
        efficiency,
        load_profile,
        database_stats,
        top_sql,
        cache_io_stats,
        object_stats,
    })
}

fn parse_report_metadata(document: &Html, instance_name: &str) -> Result<WdrReport, WdrProbeError> {
    let report_id = 0; // Will be assigned by database

    // Extract snapshot times from the snapshot info table
    let snapshot_times = extract_snapshot_times(document)?;

    Ok(WdrReport {
        id: report_id,
        instance_name: instance_name.to_string(),
        generation_time: chrono::Utc::now().to_rfc3339(),
        snapshot_start: snapshot_times.0,
        snapshot_end: snapshot_times.1,
        file_path: None,
        file_size: None,
        status: "Success".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

fn extract_snapshot_times(document: &Html) -> Result<(String, String), WdrProbeError> {
    // Look for the snapshot info table
    let table_selector =
        Selector::parse("table[summary='This table displays snapshot info']").unwrap();
    if let Some(table) = document.select(&table_selector).next() {
        let td_selector = Selector::parse("td").unwrap();
        let cells: Vec<String> = table
            .select(&td_selector)
            .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .collect();

        // Cells: 1, start_time, end_time, 2, start_time, end_time
        // We want the first snapshot's start and second snapshot's end
        if cells.len() >= 6 {
            return Ok((cells[1].clone(), cells[5].clone()));
        }
    }

    // Fallback
    Ok(("Unknown".to_string(), "Unknown".to_string()))
}

fn parse_efficiency_metrics(document: &Html, report_id: i64) -> EfficiencyMetrics {
    println!("Parsing efficiency metrics...");

    // Look for Instance Efficiency Percentages table
    let table_selector = Selector::parse(
        "table[summary='This table displays Instance Efficiency Percentages (Target 100%)']",
    )
    .unwrap();

    let mut buffer_hit_percent = 0.0;
    let mut cpu_efficiency_percent = 0.0;
    let mut soft_parse_rate_percent = 0.0;
    let mut execution_efficiency_percent = 0.0;

    if let Some(table) = document.select(&table_selector).next() {
        let row_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        for (row_idx, row) in table.select(&row_selector).enumerate() {
            if row_idx == 0 {
                continue; // 跳过表头
            }

            let cells: Vec<String> = row
                .select(&td_selector)
                .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();

            if cells.len() >= 2 {
                let metric_name = cells.get(0).unwrap_or(&"".to_string()).clone();
                let metric_value = cells
                    .get(1)
                    .unwrap_or(&"0".to_string())
                    .replace(",", "")
                    .parse::<f64>()
                    .unwrap_or(0.0);

                match metric_name.as_str() {
                    "Buffer Hit %" => buffer_hit_percent = metric_value,
                    "Effective CPU %" => cpu_efficiency_percent = metric_value,
                    "Soft Parse %" => soft_parse_rate_percent = metric_value,
                    "Non-Parse CPU %" => execution_efficiency_percent = metric_value,
                    _ => {}
                }

                println!("  Efficiency metric: {} = {}", metric_name, metric_value);
            }
        }
    }

    let hard_parse_rate_percent = 100.0 - soft_parse_rate_percent;

    EfficiencyMetrics {
        report_id,
        buffer_hit_percent,
        cpu_efficiency_percent,
        soft_parse_rate_percent,
        hard_parse_rate_percent,
        execution_efficiency_percent,
    }
}

fn parse_load_profile(document: &Html, report_id: i64) -> LoadProfile {
    println!("Parsing load profile...");

    // Try to parse Time Model table (openGauss format) or Load Profile table
    let time_model_selector =
        Selector::parse("table[summary='This table displays Time model']").unwrap();
    let load_profile_selector =
        Selector::parse("table[summary='This table displays Load Profile']").unwrap();

    let mut db_time_per_sec = 0.0;
    let mut cpu_time_per_sec = 0.0;
    let mut io_requests_per_sec = 0.0;
    let mut total_transactions = 0;
    let commits_per_sec = 0.0;
    let rollbacks_per_sec = 0.0;

    // First try Time Model table (openGauss format)
    if let Some(table) = document.select(&time_model_selector).next() {
        println!("  Found Time Model table");
        let row_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        for (row_idx, row) in table.select(&row_selector).enumerate() {
            if row_idx == 0 {
                continue; // Skip header
            }

            let cells: Vec<String> = row
                .select(&td_selector)
                .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();

            if cells.len() >= 2 {
                let stat_name = cells.get(0).unwrap_or(&"".to_string()).clone();
                let value_us = cells
                    .get(1)
                    .unwrap_or(&"0".to_string())
                    .replace(",", "")
                    .parse::<f64>()
                    .unwrap_or(0.0);

                match stat_name.as_str() {
                    "DB_TIME" => db_time_per_sec = value_us,
                    "CPU_TIME" => cpu_time_per_sec = value_us,
                    "DATA_IO_TIME" => io_requests_per_sec = value_us,
                    _ => {}
                }

                println!("  Time Model: {} = {} us", stat_name, value_us);
            }
        }
    } else if let Some(table) = document.select(&load_profile_selector).next() {
        let row_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        for (row_idx, row) in table.select(&row_selector).enumerate() {
            if row_idx == 0 {
                continue; // 跳过表头
            }

            let cells: Vec<String> = row
                .select(&td_selector)
                .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();

            if cells.len() >= 3 {
                let metric = cells.get(0).unwrap_or(&"".to_string()).clone();
                let per_second = cells
                    .get(1)
                    .unwrap_or(&"0".to_string())
                    .replace(",", "")
                    .parse::<f64>()
                    .unwrap_or(0.0);
                let per_transaction = cells
                    .get(2)
                    .unwrap_or(&"0".to_string())
                    .replace(",", "")
                    .parse::<f64>()
                    .unwrap_or(0.0);

                match metric.as_str() {
                    "DB Time(us)" => db_time_per_sec = per_second,
                    "CPU Time(us)" => cpu_time_per_sec = per_second,
                    "Read IO requests" => io_requests_per_sec = per_second,
                    "Write IO requests" => {
                        // 总IO请求 = 读 + 写
                        io_requests_per_sec += per_second;
                    }
                    "Executes (SQL)" => {
                        total_transactions = per_second as u64;
                    }
                    _ => {}
                }

                println!(
                    "  Load Profile metric: {} = {} / {}",
                    metric, per_second, per_transaction
                );
            }
        }
    }

    LoadProfile {
        report_id,
        db_time_per_sec,
        cpu_time_per_sec,
        io_requests_per_sec,
        total_transactions,
        commits_per_sec,
        rollbacks_per_sec,
    }
}

fn parse_database_stats(document: &Html, report_id: i64) -> Vec<DatabaseStats> {
    println!("Parsing database stats...");
    let mut stats = Vec::new();

    // Look for Database Stat table
    let table_selector =
        Selector::parse("table[summary='This table displays Database Stat']").unwrap();
    if let Some(table) = document.select(&table_selector).next() {
        let row_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        for (row_idx, row) in table.select(&row_selector).enumerate() {
            if row_idx == 0 {
                continue; // Skip header
            }

            let cells: Vec<String> = row
                .select(&td_selector)
                .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();

            if cells.len() >= 18 {
                let parse_u64 = |s: &str| -> u64 { s.parse().unwrap_or(0) };
                let parse_f64 = |s: &str| -> f64 { s.parse().unwrap_or(0.0) };

                stats.push(DatabaseStats {
                    id: 0,
                    report_id,
                    db_name: cells[0].clone(),
                    backends: parse_u64(&cells[1]),
                    xact_commit: parse_u64(&cells[2]),
                    xact_rollback: parse_u64(&cells[3]),
                    blks_read: parse_u64(&cells[4]),
                    blks_hit: parse_u64(&cells[5]),
                    tuple_returned: parse_u64(&cells[6]),
                    tuple_fetched: parse_u64(&cells[7]),
                    tuple_inserted: parse_u64(&cells[8]),
                    tuple_updated: parse_u64(&cells[9]),
                    tuple_deleted: parse_u64(&cells[10]),
                    conflicts: parse_u64(&cells[11]),
                    temp_files: parse_u64(&cells[12]),
                    temp_bytes: parse_u64(&cells[13]),
                    deadlocks: parse_u64(&cells[14]),
                    blk_read_time: parse_f64(&cells[15]),
                    blk_write_time: parse_f64(&cells[16]),
                    stats_reset: Some(cells[17].clone()),
                });
            }
        }
    }

    stats
}

fn parse_top_sqls(document: &Html, report_id: i64) -> Vec<TopSql> {
    println!("Parsing top SQLs...");
    let mut sqls = Vec::new();

    // Look for all tables in the SQL Statistics section
    let table_selector = Selector::parse("table.tdiff").unwrap();
    println!("CompleteWDRParser: Looking for SQL tables...");

    // Counter to limit how many SQLs we parse (for performance)
    let mut total_sqls = 0;

    // Find all tables that might contain SQL data
    for (table_idx, table) in document.select(&table_selector).enumerate() {
        if total_sqls >= 200 {
            break; // Limit to 200 SQLs for performance
        }

        // Check if this table looks like a SQL stats table by looking at headers
        let header_selector = Selector::parse("th").unwrap();
        let headers: Vec<String> = table
            .select(&header_selector)
            .map(|h| h.text().collect::<String>())
            .collect();

        // SQL stats tables have many columns including "Unique SQL Id" and "SQL Text"
        if headers.len() > 20 && headers.iter().any(|h| h.contains("Unique SQL Id")) {
            println!(
                "CompleteWDRParser: Found SQL table {} with {} columns",
                table_idx,
                headers.len()
            );

            // Parse rows from this table
            let row_selector = Selector::parse("tr").unwrap();
            for (row_idx, row) in table.select(&row_selector).enumerate() {
                // Skip header rows (row_idx == 0)
                if row_idx == 0 {
                    continue;
                }

                match parse_sql_row_from_table(&row, total_sqls as i32 + 1) {
                    Ok(mut sql) => {
                        sql.report_id = report_id; // Set the report_id
                        println!(
                            "CompleteWDRParser: Parsed SQL {}: {}",
                            total_sqls,
                            sql.sql_text.chars().take(50).collect::<String>()
                        );
                        sqls.push(sql);
                        total_sqls += 1;

                        if total_sqls >= 200 {
                            break;
                        }
                    }
                    Err(e) => {
                        println!(
                            "CompleteWDRParser: Failed to parse row {} in table {}: {}",
                            row_idx, table_idx, e
                        );
                    }
                }
            }
        }
    }

    println!("CompleteWDRParser: Total SQLs parsed: {}", sqls.len());
    sqls
}

fn parse_sql_row_from_table(
    row: &scraper::ElementRef,
    rank: i32,
) -> Result<TopSql, crate::utils::error::WdrProbeError> {
    let cells: Vec<String> = row
        .select(&Selector::parse("td").unwrap())
        .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
        .collect();

    // Check if this is a SQL text table (4 columns) or full stats table (26 columns)
    let is_sql_text_table = cells.len() == 4
        || (cells.len() >= 4
            && cells
                .get(3)
                .map(|c| {
                    c.len() > 10
                        && (c.contains("SELECT") || c.contains("UPDATE") || c.contains("INSERT"))
                })
                .unwrap_or(false));

    if is_sql_text_table {
        // SQL text table: Unique SQL Id, Node Name, User Name, SQL Text
        if cells.len() < 4 {
            return Err(crate::utils::error::WdrProbeError::Parse(format!(
                "Invalid SQL text row format: expected at least 4 columns, got {}",
                cells.len()
            )));
        }

        let unique_sql_id = cells.get(0).unwrap_or(&"0".to_string()).clone();
        let sql_text = cells.get(3).unwrap_or(&"".to_string()).clone();

        Ok(TopSql {
            id: 0,
            report_id: 0,
            sql_id: Some(unique_sql_id),
            sql_text,
            executions: 0,
            total_elapsed_time: 0.0,
            cpu_time: 0.0,
            io_time: 0.0,
            buffer_gets: 0,
            disk_reads: 0,
            rows_processed: 0,
            first_load_time: chrono::Utc::now().to_rfc3339(),
            last_load_time: chrono::Utc::now().to_rfc3339(),
            is_hot_sql: true,
            rank_by_time: Some(rank),
        })
    } else {
        // Full SQL stats table with many columns
        if cells.len() < 14 {
            return Err(crate::utils::error::WdrProbeError::Parse(format!(
                "Invalid SQL row format: expected at least 14 columns, got {}",
                cells.len()
            )));
        }

        // Parse cell values with proper error handling
        let parse_cell = |idx: usize, default: i64| -> i64 {
            cells
                .get(idx)
                .unwrap_or(&"0".to_string())
                .replace(",", "") // Remove commas from numbers
                .parse::<i64>()
                .unwrap_or(default)
        };

        let parse_float = |idx: usize, default: f64| -> f64 {
            cells
                .get(idx)
                .unwrap_or(&"0".to_string())
                .replace(",", "") // Remove commas from numbers
                .parse::<f64>()
                .unwrap_or(default)
        };

        let unique_sql_id = cells.get(0).unwrap_or(&"0".to_string()).clone();

        // Extract SQL text from the last column (index 24 for openGauss WDR format)
        // Columns: 0=Unique SQL Id, 1=User Name, 2=Total Elapse Time, 3=CPU Time,
        // 4=Avg Elapse Time, 5=Returned Rows, 6=Calls, 7=Tuples Read, 8=Physical Read,
        // 9=Logical Read, ..., 24=SQL Text (truncated)
        let sql_text = if cells.len() > 24 {
            cells.get(24).unwrap_or(&"".to_string()).clone()
        } else {
            "".to_string()
        };

        Ok(TopSql {
            id: 0,
            report_id: 0,
            sql_id: Some(unique_sql_id),
            sql_text,
            executions: parse_cell(6, 0) as u64, // Calls (column index 6)
            total_elapsed_time: parse_float(2, 0.0), // Total Elapse Time (column index 2)
            cpu_time: parse_float(3, 0.0), // CPU Time (column index 3)
            io_time: parse_float(13, 0.0), // Data IO Time (column index 13)
            buffer_gets: parse_cell(9, 0) as u64, // Logical Read (column index 9)
            disk_reads: parse_cell(8, 0) as u64,  // Physical Read (column index 8)
            rows_processed: parse_cell(5, 0) as u64, // Returned Rows (column index 5)
            first_load_time: chrono::Utc::now().to_rfc3339(),
            last_load_time: chrono::Utc::now().to_rfc3339(),
            is_hot_sql: true,
            rank_by_time: Some(rank),
        })
    }
}

fn parse_cache_io_stats(document: &Html, report_id: i64) -> Vec<CacheIoStats> {
    println!("Parsing cache IO stats...");
    let mut stats = Vec::new();

    // Look for cache IO stat tables (multiple tables for different orderings)
    // The summary is: "This table displays User table IO activity ordered by ..."
    let table_selector = Selector::parse("table[summary*='User table IO activity']").unwrap();

    let mut table_count = 0;
    for table in document.select(&table_selector) {
        table_count += 1;
        println!(
            "  Found cache IO table {}: {:?}",
            table_count,
            table.value().attr("summary")
        );

        let row_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        let mut row_count = 0;
        for (row_idx, row) in table.select(&row_selector).enumerate() {
            if row_idx == 0 {
                continue; // Skip header
            }

            let cells: Vec<String> = row
                .select(&td_selector)
                .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();

            row_count += 1;
            println!("    Row {}: {} cells", row_count, cells.len());

            // Debug: print first few rows
            if row_count <= 3 {
                println!("      Cells: {:?}", cells);
            }

            // Object Type column determines if it's table or index
            if cells.len() >= 8 {
                let object_type = if cells[2].to_lowercase().contains("index") {
                    "index"
                } else {
                    "table"
                };

                let parse_u64 = |s: &str| -> u64 { s.parse().unwrap_or(0) };
                let parse_ratio = |s: &str| -> f64 { s.replace("%", "").parse().unwrap_or(0.0) };

                stats.push(CacheIoStats {
                    id: 0,
                    report_id,
                    schema_name: cells[0].clone(),
                    object_name: cells[1].clone(),
                    object_type: object_type.to_string(),
                    heap_blks_read: parse_u64(&cells[3]),
                    heap_blks_hit: parse_u64(&cells[4]),
                    heap_blks_hit_ratio: parse_ratio(&cells[5]),
                    idx_blks_read: parse_u64(&cells[6]),
                    idx_blks_hit: parse_u64(&cells[7]),
                    idx_blks_hit_ratio: if cells.len() > 8 {
                        parse_ratio(&cells[8])
                    } else {
                        0.0
                    },
                    toast_blks_read: 0, // TODO: Parse if available
                    toast_blks_hit: 0,
                    toast_blks_hit_ratio: 0.0,
                    tidx_blks_read: 0,
                    tidx_blks_hit: 0,
                    tidx_blks_hit_ratio: 0.0,
                });
            }
        }
        println!("  Table {}: processed {} data rows", table_count, row_count);
    }

    stats
}

fn parse_object_stats(document: &Html, report_id: i64) -> Vec<ObjectStats> {
    println!("Parsing object stats...");
    let mut stats = Vec::new();

    // Look for User Tables stats and User Index stats tables
    // The summary is: "This table displays User Tables stats"
    for obj_type in &["User Tables", "User Index"] {
        let table_selector = Selector::parse(&format!(
            "table[summary*='This table displays {} stats']",
            obj_type
        ))
        .unwrap();

        for table in document.select(&table_selector) {
            let row_selector = Selector::parse("tr").unwrap();
            let td_selector = Selector::parse("td").unwrap();

            for (row_idx, row) in table.select(&row_selector).enumerate() {
                if row_idx == 0 {
                    continue; // Skip header
                }

                let cells: Vec<String> = row
                    .select(&td_selector)
                    .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .collect();

                if cells.len() >= 12 {
                    let parse_u64 = |s: &str| -> u64 { s.parse().unwrap_or(0) };

                    // For User Tables: DB Name, Schema, Relname, Seq Scan, Seq Tup Read, Index Scan, Index Tup Fetch, etc.
                    // For User Index: DB Name, Schema, Relname, ...
                    let object_type = if obj_type.contains("Tables") {
                        "table"
                    } else {
                        "index"
                    };

                    stats.push(ObjectStats {
                        id: 0,
                        report_id,
                        schema_name: format!("{}.{}", cells[0], cells[1]), // DB.Schema
                        object_name: cells[2].clone(),                     // Table/Index name
                        object_type: object_type.to_string(),
                        total_scans: parse_u64(&cells[3]),
                        seq_scans: parse_u64(&cells[4]),
                        idx_scans: parse_u64(&cells[5]),
                        seq_reads: parse_u64(&cells[6]),
                        idx_reads: parse_u64(&cells[7]),
                        inserts: parse_u64(&cells[8]),
                        updates: parse_u64(&cells[9]),
                        deletes: parse_u64(&cells[10]),
                        dead_tuples: parse_u64(&cells[11]),
                        needs_vacuum: false, // TODO: Calculate based on dead tuples ratio
                    });
                }
            }
        }
    }

    stats
}
