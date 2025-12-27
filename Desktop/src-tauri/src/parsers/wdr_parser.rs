// WDR file parser
// Parses HTML and raw WDR report files

use crate::models::{EfficiencyMetrics, LoadProfile, TopSql, WdrReport};
use crate::utils::WdrProbeError;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Parse WDR report from HTML file
pub fn parse_wdr_html(file_path: &str) -> Result<WdrReport, WdrProbeError> {
    let file = File::open(file_path).map_err(|e| WdrProbeError::Io(e))?;

    let reader = BufReader::new(file);
    let html_content: String = reader.lines().collect::<Result<Vec<_>, _>>()?.join("\n");

    let document = Html::parse_document(&html_content);

    // Extract instance name
    let instance_name = extract_instance_name(&document)?;

    // Extract generation time
    let generation_time = extract_generation_time(&document)?;

    // Extract snapshot period
    let (snapshot_start, snapshot_end) = extract_snapshot_period(&document)?;

    // Get file size
    let file_size = Path::new(file_path).metadata().map(|m| m.len()).ok();

    let report = WdrReport {
        id: 0, // Will be assigned by database
        instance_name,
        generation_time,
        snapshot_start,
        snapshot_end,
        file_path: Some(file_path.to_string()),
        file_size,
        status: "completed".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(report)
}

/// Parse WDR report from raw text file
pub fn parse_wdr_raw(file_path: &str) -> Result<WdrReport, WdrProbeError> {
    let file = File::open(file_path).map_err(|e| WdrProbeError::Io(e))?;

    let reader = BufReader::new(file);

    let mut instance_name = String::new();
    let mut generation_time = String::new();
    let mut snapshot_start = String::new();
    let mut snapshot_end = String::new();

    for line in reader.lines() {
        let line = line.map_err(|e| WdrProbeError::Io(e))?;

        // Parse instance name
        if let Some(idx) = line.find("Instance Name:") {
            instance_name = line[idx + 14..].trim().to_string();
        }

        // Parse generation time
        if let Some(idx) = line.find("Report Generation Time:") {
            generation_time = line[idx + 24..].trim().to_string();
        }

        // Parse snapshot period
        if let Some(idx) = line.find("Snapshot Start:") {
            snapshot_start = line[idx + 16..].trim().to_string();
        }
        if let Some(idx) = line.find("Snapshot End:") {
            snapshot_end = line[idx + 14..].trim().to_string();
        }
    }

    let file_size = Path::new(file_path).metadata().map(|m| m.len()).ok();

    let report = WdrReport {
        id: 0,
        instance_name: if !instance_name.is_empty() {
            instance_name
        } else {
            "Unknown".to_string()
        },
        generation_time: if !generation_time.is_empty() {
            generation_time
        } else {
            chrono::Utc::now().to_rfc3339()
        },
        snapshot_start: if !snapshot_start.is_empty() {
            snapshot_start
        } else {
            "Unknown".to_string()
        },
        snapshot_end: if !snapshot_end.is_empty() {
            snapshot_end
        } else {
            "Unknown".to_string()
        },
        file_path: Some(file_path.to_string()),
        file_size,
        status: "completed".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(report)
}

/// Parse Top SQL statements from WDR report
pub fn parse_top_sqls(file_path: &str) -> Result<Vec<TopSql>, WdrProbeError> {
    let file = File::open(file_path).map_err(|e| WdrProbeError::Io(e))?;

    let reader = BufReader::new(file);
    let html_content: String = reader.lines().collect::<Result<Vec<_>, _>>()?.join("\n");

    let document = Html::parse_document(&html_content);
    let mut sqls = Vec::new();

    // Look for all tables in the SQL Statistics section
    let table_selector = Selector::parse("table.tdiff").unwrap();
    println!("WDR Parser: Looking for SQL tables...");

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
                "WDR Parser: Found SQL table {} with {} columns",
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
                    Ok(sql) => {
                        println!(
                            "WDR Parser: Parsed SQL {}: {}",
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
                            "WDR Parser: Failed to parse row {} in table {}: {}",
                            row_idx, table_idx, e
                        );
                    }
                }
            }
        }
    }

    println!("WDR Parser: Total SQLs parsed: {}", sqls.len());

    // If no SQLs were found with the new method, fall back to raw parsing
    if sqls.is_empty() {
        println!("WDR Parser: No SQLs found with table parsing, trying raw text parsing...");
        sqls = parse_top_sqls_raw(file_path)?;
        println!("WDR Parser: Raw parsing found {} SQLs", sqls.len());
    }

    Ok(sqls)
}

/// Parse SQL row from WDR report table
fn parse_sql_row_from_table(row: &scraper::ElementRef, rank: i32) -> Result<TopSql, WdrProbeError> {
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
            return Err(WdrProbeError::Parse(format!(
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
            return Err(WdrProbeError::Parse(format!(
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

        // Map cells to fields according to actual WDR report format
        // 0: Unique SQL Id, 1: Node Name, 2: User Name,
        // 3: Total Elapse Time(us) or CPU Time(us) or other metric,
        // 4: Calls,
        // ... other metrics ...
        // Last column might be SQL Text in some tables

        let unique_sql_id = cells.get(0).unwrap_or(&"0".to_string()).clone();

        // Try to find SQL text - it might be in the last column or we might need to look for it
        let sql_text = if cells.len() > 25 {
            // If we have 26 columns, SQL text is likely at the end
            cells.get(25).unwrap_or(&"".to_string()).clone()
        } else if cells.len() >= 4 {
            // For other tables, try to find SQL text in column 3 or later
            if let Some(candidate) = cells.get(3) {
                if candidate.len() > 20
                    && (candidate.contains("SELECT")
                        || candidate.contains("UPDATE")
                        || candidate.contains("INSERT"))
                {
                    candidate.clone()
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        Ok(TopSql {
            id: 0,
            report_id: 0,
            sql_id: Some(unique_sql_id),
            sql_text,
            executions: parse_cell(4, 0) as u64, // Calls
            total_elapsed_time: parse_float(3, 0.0),
            cpu_time: parse_float(13, 0.0),
            io_time: parse_float(14, 0.0),
            buffer_gets: parse_cell(11, 0) as u64, // Logical Read
            disk_reads: parse_cell(12, 0) as u64,  // Physical Read
            rows_processed: parse_cell(8, 0) as u64, // Returned Rows
            first_load_time: chrono::Utc::now().to_rfc3339(),
            last_load_time: chrono::Utc::now().to_rfc3339(),
            is_hot_sql: true,
            rank_by_time: Some(rank),
        })
    }
}

/// Helper function to extract instance name from HTML
fn extract_instance_name(document: &Html) -> Result<String, WdrProbeError> {
    let selector = Selector::parse(".instance-name, .report-header h1, h1").unwrap();

    for element in document.select(&selector) {
        let text = element.text().collect::<Vec<_>>().join(" ");
        if !text.is_empty() {
            return Ok(text.trim().to_string());
        }
    }

    Ok("Unknown Instance".to_string())
}

/// Helper function to extract generation time from HTML
fn extract_generation_time(document: &Html) -> Result<String, WdrProbeError> {
    let selector = Selector::parse(".generation-time, .timestamp, .report-time").unwrap();

    for element in document.select(&selector) {
        let text = element.text().collect::<Vec<_>>().join(" ");
        if !text.is_empty() {
            return Ok(text.trim().to_string());
        }
    }

    Ok(chrono::Utc::now().to_rfc3339())
}

/// Helper function to extract snapshot period from HTML
fn extract_snapshot_period(document: &Html) -> Result<(String, String), WdrProbeError> {
    let selector = Selector::parse(".snapshot-start, .snapshot-end, .period").unwrap();

    let mut start = String::new();
    let mut end = String::new();

    for element in document.select(&selector) {
        let text = element.text().collect::<Vec<_>>().join(" ");
        if text.contains("Start") || text.contains("start") {
            start = text;
        } else if text.contains("End") || text.contains("end") {
            end = text;
        }
    }

    Ok((
        if !start.is_empty() {
            start
        } else {
            "Unknown".to_string()
        },
        if !end.is_empty() {
            end
        } else {
            "Unknown".to_string()
        },
    ))
}

/// Parse SQL row from HTML element
fn parse_sql_row(row: &scraper::ElementRef, rank: i32) -> Result<TopSql, WdrProbeError> {
    let cells: Vec<String> = row
        .select(&Selector::parse("td").unwrap())
        .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
        .collect();

    if cells.len() < 5 {
        return Err(WdrProbeError::Parse("Invalid SQL row format".to_string()));
    }

    Ok(TopSql {
        id: 0,
        report_id: 0, // Will be set when associated with report
        sql_id: Some(cells.get(0).unwrap_or(&"0".to_string()).clone()),
        sql_text: cells.get(1).unwrap_or(&"".to_string()).clone(),
        executions: cells
            .get(2)
            .unwrap_or(&"0".to_string())
            .parse()
            .unwrap_or(0),
        total_elapsed_time: cells
            .get(3)
            .unwrap_or(&"0".to_string())
            .parse()
            .unwrap_or(0.0),
        cpu_time: cells
            .get(4)
            .unwrap_or(&"0".to_string())
            .parse()
            .unwrap_or(0.0),
        io_time: cells
            .get(5)
            .unwrap_or(&"0".to_string())
            .parse()
            .unwrap_or(0.0),
        buffer_gets: cells
            .get(6)
            .unwrap_or(&"0".to_string())
            .parse()
            .unwrap_or(0),
        disk_reads: cells
            .get(7)
            .unwrap_or(&"0".to_string())
            .parse()
            .unwrap_or(0),
        rows_processed: cells
            .get(8)
            .unwrap_or(&"0".to_string())
            .parse()
            .unwrap_or(0),
        first_load_time: chrono::Utc::now().to_rfc3339(),
        last_load_time: chrono::Utc::now().to_rfc3339(),
        is_hot_sql: true,
        rank_by_time: Some(rank),
    })
}

/// Parse Top SQLs from raw text format
fn parse_top_sqls_raw(file_path: &str) -> Result<Vec<TopSql>, WdrProbeError> {
    let file = File::open(file_path).map_err(|e| WdrProbeError::Io(e))?;

    let reader = BufReader::new(file);
    let mut sqls = Vec::new();
    let mut rank = 1;

    for line in reader.lines() {
        let line = line.map_err(|e| WdrProbeError::Io(e))?;

        // Simple heuristic: lines with "SELECT" or "UPDATE" that are not comments
        if (line.starts_with("SELECT") || line.starts_with("UPDATE") || line.starts_with("INSERT"))
            && !line.starts_with("--")
        {
            let sql = TopSql {
                id: 0,
                report_id: 0,
                sql_id: Some(rank.to_string()),
                sql_text: line.trim().to_string(),
                executions: 0,
                total_elapsed_time: 0.0,
                cpu_time: 0.0,
                io_time: 0.0,
                buffer_gets: 0,
                disk_reads: 0,
                rows_processed: 0,
                first_load_time: chrono::Utc::now().to_rfc3339(),
                last_load_time: chrono::Utc::now().to_rfc3339(),
                is_hot_sql: false,
                rank_by_time: Some(rank),
            };

            sqls.push(sql);
            rank += 1;
        }
    }

    Ok(sqls)
}

/// Parse efficiency metrics from WDR report
pub fn parse_efficiency_metrics(
    _file_path: &str,
    report_id: i64,
) -> Result<EfficiencyMetrics, WdrProbeError> {
    // Placeholder implementation - would parse actual metrics from WDR file
    Ok(EfficiencyMetrics {
        report_id,
        buffer_hit_percent: 95.5,
        cpu_efficiency_percent: 88.2,
        soft_parse_rate_percent: 92.0,
        hard_parse_rate_percent: 8.0,
        execution_efficiency_percent: 90.5,
    })
}

/// Parse load profile from WDR report
pub fn parse_load_profile(_file_path: &str, report_id: i64) -> Result<LoadProfile, WdrProbeError> {
    // Placeholder implementation - would parse actual load profile from WDR file
    Ok(LoadProfile {
        report_id,
        db_time_per_sec: 125.5,
        cpu_time_per_sec: 98.3,
        io_requests_per_sec: 450.2,
        total_transactions: 50000,
        commits_per_sec: 125.0,
        rollbacks_per_sec: 2.5,
    })
}
