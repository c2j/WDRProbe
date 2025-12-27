// Test SQL parsing functionality
#[cfg(test)]
mod test_sql_parser {
    use scraper::{Html, Selector};
    use wdrprobe_desktop_lib::models::TopSql;

    #[test]
    fn test_parse_sql_from_html() {
        let html = r#"
        <html>
        <body>
        <h2>SQL Statistics</h2>
        <table class="tdiff">
            <tr>
                <th>Unique SQL Id</th>
                <th>Node Name</th>
                <th>User Name</th>
                <th>Total Elapse Time(us)</th>
                <th>Calls</th>
                <th>Avg Elapse Time(us)</th>
                <th>Min Elapse Time(us)</th>
                <th>Max Elapse Time(us)</th>
                <th>Returned Rows</th>
                <th>Tuples Read</th>
                <th>Tuples Affected</th>
                <th>Logical Read</th>
                <th>Physical Read</th>
                <th>CPU Time(us)</th>
                <th>Data IO Time</th>
                <th>SQL Text</th>
            </tr>
            <tr>
                <td>12345</td>
                <td>cn1</td>
                <td>admin</td>
                <td>1000000</td>
                <td>100</td>
                <td>10000</td>
                <td>5000</td>
                <td>50000</td>
                <td>1000</td>
                <td>10000</td>
                <td>100</td>
                <td>50000</td>
                <td>1000</td>
                <td>200000</td>
                <td>50000</td>
                <td>SELECT * FROM users WHERE id = ?</td>
            </tr>
        </table>
        </body>
        </html>
        "#;

        let document = Html::parse_document(html);
        println!("Document parsed, looking for tables...");

        // Check what tables we have
        let all_tables = Selector::parse("table").unwrap();
        for (i, table) in document.select(&all_tables).enumerate() {
            println!("Found table {}: {:?}", i, table.value().attr("class"));
        }

        let table_selector = Selector::parse("table.tdiff").unwrap();
        let row_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        let mut sqls = Vec::new();

        for table in document.select(&table_selector) {
            let headers: Vec<String> = table
                .select(&Selector::parse("th").unwrap())
                .map(|h| {
                    let text = h.text().collect::<String>();
                    println!("Header: '{}'", text);
                    text
                })
                .collect();

            println!("Table has {} headers", headers.len());

            if headers.len() > 15 && headers.iter().any(|h| h.contains("Unique SQL Id")) {
                println!("Found SQL table with {} columns", headers.len());

                for (row_idx, row) in table.select(&row_selector).enumerate() {
                    if row_idx == 0 {
                        continue; // Skip header
                    }

                    let cells: Vec<String> = row
                        .select(&td_selector)
                        .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        .collect();

                    if cells.len() >= 14 {
                        let parse_cell = |idx: usize, default: i64| -> i64 {
                            cells
                                .get(idx)
                                .unwrap_or(&"0".to_string())
                                .replace(",", "")
                                .parse::<i64>()
                                .unwrap_or(default)
                        };

                        let parse_float = |idx: usize, default: f64| -> f64 {
                            cells
                                .get(idx)
                                .unwrap_or(&"0".to_string())
                                .replace(",", "")
                                .parse::<f64>()
                                .unwrap_or(default)
                        };

                        let unique_sql_id = cells.get(0).unwrap_or(&"0".to_string()).clone();
                        let sql_text = cells
                            .get(cells.len() - 1)
                            .unwrap_or(&"".to_string())
                            .clone();

                        let sql = TopSql {
                            id: 0,
                            report_id: 1,
                            sql_id: Some(unique_sql_id),
                            sql_text,
                            executions: parse_cell(4, 0) as u64,
                            total_elapsed_time: parse_float(3, 0.0),
                            cpu_time: parse_float(13, 0.0),
                            io_time: parse_float(14, 0.0),
                            buffer_gets: parse_cell(11, 0) as u64,
                            disk_reads: parse_cell(12, 0) as u64,
                            rows_processed: parse_cell(8, 0) as u64,
                            first_load_time: chrono::Utc::now().to_rfc3339(),
                            last_load_time: chrono::Utc::now().to_rfc3339(),
                            is_hot_sql: true,
                            rank_by_time: Some(row_idx as i32),
                        };

                        println!(
                            "Parsed SQL: id={}, text={}, calls={}, elapsed={}",
                            sql.sql_id.as_ref().unwrap(),
                            sql.sql_text.chars().take(30).collect::<String>(),
                            sql.executions,
                            sql.total_elapsed_time
                        );

                        sqls.push(sql);
                    }
                }
            }
        }

        assert_eq!(sqls.len(), 1);
        assert_eq!(sqls[0].sql_id, Some("12345".to_string()));
        assert_eq!(sqls[0].executions, 100);
        assert_eq!(sqls[0].total_elapsed_time, 1000000.0);
        assert!(sqls[0].sql_text.contains("SELECT * FROM users"));
    }
}
