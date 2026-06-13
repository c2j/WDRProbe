use std::fs::File;
use std::io::Read;
use scraper::{Html, Selector};

fn main() {
    let mut file = File::open("/Users/c2j/Desktop/Desktop_Projects/DB/WDRProbe/example/opengauss_v1.html").unwrap();
    let mut html_content = String::new();
    file.read_to_string(&mut html_content).unwrap();

    let document = Html::parse_document(&html_content);

    println!("Searching for SQL tables...");

    // Look for SQL tables
    let table_selector = Selector::parse("table.tdiff").unwrap();
    let mut sql_count = 0;

    for (table_idx, table) in document.select(&table_selector).enumerate() {
        // Check headers
        let header_selector = Selector::parse("th").unwrap();
        let headers: Vec<String> = table.select(&header_selector)
            .map(|h| h.text().collect::<String>())
            .collect();

        if headers.len() > 20 && headers.iter().any(|h| h.contains("Unique SQL Id")) {
            println!("\nFound SQL table {} with {} columns", table_idx, headers.len());

            // Parse rows
            let row_selector = Selector::parse("tr").unwrap();
            let td_selector = Selector::parse("td").unwrap();

            for (row_idx, row) in table.select(&row_selector).enumerate() {
                if row_idx == 0 {
                    continue; // Skip header
                }

                let cells: Vec<String> = row.select(&td_selector)
                    .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .collect();

                if cells.len() >= 26 {
                    sql_count += 1;
                    let unique_sql_id = cells.get(0).unwrap_or(&"0".to_string()).clone();
                    let sql_text = cells.get(25).unwrap_or(&"".to_string()).clone();
                    let calls = cells.get(4).unwrap_or(&"0".to_string()).clone();

                    println!("SQL {}: id={}, calls={}, text={}",
                             sql_count,
                             unique_sql_id,
                             calls,
                             &sql_text[..sql_text.len().min(50)]);

                    if sql_count >= 5 {
                        break;
                    }
                }
            }
        }
    }

    println!("\nTotal SQL statements found: {}", sql_count);

    // Also check database stats
    println!("\n\nChecking Database Stats...");
    let db_table_selector = Selector::parse("table[summary='This table displays Database Stat']").unwrap();
    if let Some(db_table) = document.select(&db_table_selector).next() {
        println!("Found Database Stats table");
        let row_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        for (row_idx, row) in db_table.select(&row_selector).enumerate() {
            if row_idx == 0 {
                continue;
            }

            let cells: Vec<String> = row.select(&td_selector)
                .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();

            if cells.len() >= 18 {
                let db_name = cells.get(0).unwrap_or(&"".to_string());
                let backends = cells.get(1).unwrap_or(&"0".to_string());
                println!("Database: {}, backends: {}", db_name, backends);
            }
        }
    }

    // Check Object Stats
    println!("\n\nChecking Object Stats...");
    let obj_table_selector = Selector::parse("table[summary^='This table displays Object stats']").unwrap();
    if let Some(obj_table) = document.select(&obj_table_selector).next() {
        println!("Found Object Stats table");
        let row_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        let mut obj_count = 0;
        for (row_idx, row) in obj_table.select(&row_selector).enumerate() {
            if row_idx == 0 {
                continue;
            }

            let cells: Vec<String> = row.select(&td_selector)
                .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();

            if cells.len() >= 12 {
                obj_count += 1;
                let schema = cells.get(0).unwrap_or(&"".to_string());
                let name = cells.get(1).unwrap_or(&"".to_string());
                let obj_type = cells.get(2).unwrap_or(&"".to_string());

                println!("Object {}: {}.{} ({})", obj_count, schema, name, obj_type);

                if obj_count >= 3 {
                    break;
                }
            }
        }
    }
}
