use std::fs::File;
use std::io::Read;
use scraper::{Html, Selector};

fn main() {
    let mut file = File::open("/Users/c2j/Desktop/Desktop_Projects/DB/WDRProbe/example/opengauss_v1.html").unwrap();
    let mut html_content = String::new();
    file.read_to_string(&mut html_content).unwrap();

    let document = Html::parse_document(&html_content);

    println!("Searching for Cache IO tables...");

    let table_selector = Selector::parse("table[summary*='User table IO activity']").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();

    let mut table_count = 0;
    let mut total_rows = 0;

    for (table_idx, table) in document.select(&table_selector).enumerate() {
        table_count += 1;
        println!("\nFound Cache IO table {}: {:?}", table_idx, table.value().attr("summary"));

        for (row_idx, row) in table.select(&row_selector).enumerate() {
            if row_idx == 0 {
                continue; // Skip header
            }

            let cells: Vec<String> = row.select(&td_selector)
                .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();

            if !cells.is_empty() && cells[0].len() > 0 {
                total_rows += 1;
                println!("  Row {}: {} cells, DB: {}, Schema: {}",
                         row_idx, cells.len(),
                         cells.get(0).unwrap_or(&"N/A".to_string()),
                         cells.get(1).unwrap_or(&"N/A".to_string()));

                if total_rows >= 3 {
                    break;
                }
            }
        }
    }

    println!("\nFound {} Cache IO tables with {} total rows", table_count, total_rows);
}
