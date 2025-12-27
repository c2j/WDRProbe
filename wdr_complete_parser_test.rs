// 完整的WDR解析测试程序
// 测试所有WDR报告部分的解析功能

use scraper::{Html, Selector};
use std::fs::File;
use std::io::Read;

fn main() {
    let file_path = "/Users/c2j/Desktop/Desktop_Projects/DB/WDRProbe/example/opengauss_v1.html";

    println!("=== WDR报告完整解析测试 ===\n");
    println!("正在解析文件: {}\n", file_path);

    // 读取HTML文件
    let mut file = File::open(file_path).expect("无法打开文件");
    let mut html_content = String::new();
    file.read_to_string(&mut html_content).expect("无法读取文件");

    let document = Html::parse_document(&html_content);

    // 1. 测试基本报告信息解析
    println!("=== 1. 基本报告信息 ===");
    parse_basic_info(&document);

    // 2. 测试数据库统计解析
    println!("\n=== 2. 数据库统计解析 ===");
    parse_database_stats(&document);

    // 3. 测试Load Profile解析
    println!("\n=== 3. Load Profile解析 ===");
    parse_load_profile(&document);

    // 4. 测试SQL统计解析 - 寻找所有可能的SQL表
    println!("\n=== 4. SQL统计解析 ===");
    parse_sql_statistics(&document);

    // 5. 测试Object Stats解析
    println!("\n=== 5. Object Stats解析 ===");
    parse_object_stats(&document);

    println!("\n=== 解析完成 ===");
}

fn parse_basic_info(document: &Html) {
    // 提取报告标题
    let title_selector = Selector::parse("h1").unwrap();
    if let Some(title) = document.select(&title_selector).next() {
        println!("报告标题: {}", title.text().collect::<Vec<_>>().join(" "));
    }

    // 提取快照信息
    let snapshot_selector = Selector::parse("table[summary='This table displays snapshot info']").unwrap();
    if let Some(table) = document.select(&snapshot_selector).next() {
        let row_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        let mut snapshots = Vec::new();
        for (row_idx, row) in table.select(&row_selector).enumerate() {
            if row_idx == 0 {
                continue; // 跳过表头
            }

            let cells: Vec<String> = row.select(&td_selector)
                .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();

            if cells.len() >= 3 {
                let snapshot_id = cells.get(0).unwrap_or(&"".to_string()).clone();
                let start_time = cells.get(1).unwrap_or(&"".to_string()).clone();
                let end_time = cells.get(2).unwrap_or(&"".to_string()).clone();

                println!("快照 {}: {} - {}", snapshot_id, start_time, end_time);
                snapshots.push((snapshot_id, start_time, end_time));
            }
        }
        println!("共找到 {} 个快照", snapshots.len());
    }

    // 提取主机信息
    let host_selector = Selector::parse("table[summary='This table displays host info']").unwrap();
    if let Some(table) = document.select(&host_selector).next() {
        let row_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        for (row_idx, row) in table.select(&row_selector).enumerate() {
            if row_idx == 0 {
                continue; // 跳过表头
            }

            let cells: Vec<String> = row.select(&td_selector)
                .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();

            if cells.len() >= 6 {
                println!("主机节点: {}", cells.get(0).unwrap_or(&"".to_string()));
                println!("CPU数量: {}", cells.get(1).unwrap_or(&"".to_string()));
                println!("物理内存: {}", cells.get(5).unwrap_or(&"".to_string()));
                break;
            }
        }
    }
}

fn parse_database_stats(document: &Html) {
    let db_table_selector = Selector::parse("table[summary='This table displays Database Stat']").unwrap();

    if let Some(db_table) = document.select(&db_table_selector).next() {
        println!("找到数据库统计表");

        let row_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();
        let th_selector = Selector::parse("th").unwrap();

        // 解析表头
        let headers: Vec<String> = db_table.select(&th_selector)
            .map(|h| h.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .collect();

        println!("表头列数: {}", headers.len());
        if headers.len() > 0 {
            println!("前5列: {:?}", headers.iter().take(5).collect::<Vec<_>>());
        }

        let mut db_count = 0;
        for (row_idx, row) in db_table.select(&row_selector).enumerate() {
            if row_idx == 0 {
                continue; // 跳过表头
            }

            let cells: Vec<String> = row.select(&td_selector)
                .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();

            if cells.len() >= 17 {
                db_count += 1;
                let db_name = cells.get(0).unwrap_or(&"".to_string());
                let backends = cells.get(1).unwrap_or(&"0".to_string());
                let xact_commit = cells.get(2).unwrap_or(&"0".to_string());
                let xact_rollback = cells.get(3).unwrap_or(&"0".to_string());

                println!("数据库 {}: {} (连接数: {}, 提交: {}, 回滚: {})",
                         db_count, db_name, backends, xact_commit, xact_rollback);

                if db_count >= 3 {
                    break;
                }
            }
        }
        println!("共解析 {} 个数据库", db_count);
    } else {
        println!("未找到数据库统计表");
    }
}

fn parse_load_profile(document: &Html) {
    let load_table_selector = Selector::parse("table[summary='This table displays Load Profile']").unwrap();

    if let Some(load_table) = document.select(&load_table_selector).next() {
        println!("找到Load Profile表");

        let row_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        let mut metrics_count = 0;
        for (row_idx, row) in load_table.select(&row_selector).enumerate() {
            if row_idx == 0 {
                continue; // 跳过表头
            }

            let cells: Vec<String> = row.select(&td_selector)
                .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();

            if cells.len() >= 3 {
                metrics_count += 1;
                let metric = cells.get(0).unwrap_or(&"".to_string());
                let per_second = cells.get(1).unwrap_or(&"0".to_string());
                let per_transaction = cells.get(2).unwrap_or(&"0".to_string());

                println!("指标 {}: {} (每秒: {}, 每事务: {})",
                         metrics_count, metric, per_second, per_transaction);

                if metrics_count >= 10 {
                    break;
                }
            }
        }
        println!("共解析 {} 个负载指标", metrics_count);
    } else {
        println!("未找到Load Profile表");
    }
}

fn parse_sql_statistics(document: &Html) {
    let table_selector = Selector::parse("table.tdiff").unwrap();
    let mut sql_table_count = 0;
    let mut total_sql_count = 0;

    println!("搜索所有table.tdiff表...");

    for (table_idx, table) in document.select(&table_selector).enumerate() {
        // 检查表头
        let th_selector = Selector::parse("th").unwrap();
        let headers: Vec<String> = table.select(&th_selector)
            .map(|h| h.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .collect();

        println!("\n表 {}: {} 列", table_idx, headers.len());

        if headers.len() > 0 {
            println!("表头: {:?}", headers.iter().take(5).collect::<Vec<_>>());
        }

        // 检查是否是SQL统计表
        let is_sql_table = headers.iter().any(|h| h.contains("Unique SQL Id"));

        if is_sql_table {
            sql_table_count += 1;
            println!("  -> 这是SQL统计表!");

            let row_selector = Selector::parse("tr").unwrap();
            let td_selector = Selector::parse("td").unwrap();

            let mut sql_count_in_table = 0;
            for (row_idx, row) in table.select(&row_selector).enumerate() {
                if row_idx == 0 {
                    continue; // 跳过表头
                }

                let cells: Vec<String> = row.select(&td_selector)
                    .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .collect();

                if cells.len() >= 4 {
                    sql_count_in_table += 1;
                    total_sql_count += 1;

                    let sql_id = cells.get(0).unwrap_or(&"".to_string());
                    let node_name = cells.get(1).unwrap_or(&"".to_string());
                    let user_name = cells.get(2).unwrap_or(&"".to_string());
                    let sql_text = cells.get(3).unwrap_or(&"".to_string());

                    if sql_count_in_table <= 3 {
                        println!("  SQL {}: ID={}, 用户={}, SQL={}",
                                 sql_count_in_table,
                                 sql_id,
                                 user_name,
                                 &sql_text[..sql_text.len().min(60)]);
                    }
                }
            }
            println!("  -> 此表共 {} 条SQL", sql_count_in_table);
        } else {
            // 检查是否是其他类型的表
            let summary = table.attr("summary").unwrap_or("");
            if !summary.is_empty() {
                println!("  -> 这是其他类型表: {}", summary);
            }
        }

        // 限制搜索的表数量，避免处理过多
        if table_idx > 50 {
            break;
        }
    }

    println!("\n总共找到 {} 个SQL统计表，共 {} 条SQL", sql_table_count, total_sql_count);

    // 如果没有找到SQL统计表，尝试查找SQL文本表
    if sql_table_count == 0 {
        println!("\n未找到SQL统计表，尝试查找SQL文本...");
        let sql_text_selectors = vec![
            "#SQL_Text1",
            "#SQL_Text2",
            "#SQL_Text3",
        ];

        for selector in sql_text_selectors {
            if let Some(element) = document.select(&Selector::parse(selector).unwrap()).next() {
                println!("找到SQL文本区域: {}", selector);

                let row_selector = Selector::parse("tr").unwrap();
                let td_selector = Selector::parse("td").unwrap();

                let mut sql_count = 0;
                for row in element.select(&row_selector) {
                    let cells: Vec<String> = row.select(&td_selector)
                        .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        .collect();

                    if cells.len() >= 2 {
                        sql_count += 1;
                        if sql_count <= 3 {
                            println!("  SQL {}: {}", sql_count, &cells[1][..cells[1].len().min(60)]);
                        }
                    }
                }
                println!("  此区域共 {} 条SQL", sql_count);
            }
        }
    }
}

fn parse_object_stats(document: &Html) {
    let obj_table_selector = Selector::parse("table[summary^='This table displays Object stats']").unwrap();

    if let Some(obj_table) = document.select(&obj_table_selector).next() {
        println!("找到Object Stats表");

        let row_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        let mut obj_count = 0;
        for (row_idx, row) in obj_table.select(&row_selector).enumerate() {
            if row_idx == 0 {
                continue; // 跳过表头
            }

            let cells: Vec<String> = row.select(&td_selector)
                .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();

            if cells.len() >= 3 {
                obj_count += 1;
                let schema = cells.get(0).unwrap_or(&"".to_string());
                let name = cells.get(1).unwrap_or(&"".to_string());
                let obj_type = cells.get(2).unwrap_or(&"".to_string());

                println!("对象 {}: {}.{} ({})", obj_count, schema, name, obj_type);

                if obj_count >= 5 {
                    break;
                }
            }
        }
        println!("共解析 {} 个对象", obj_count);
    } else {
        // 尝试其他可能的表名
        let other_selectors = vec![
            "table[summary*='Object']",
            "table[summary*='Index']",
            "table[summary*='Table']",
        ];

        for selector in other_selectors {
            if let Some(table) = document.select(&Selector::parse(selector).unwrap()).next() {
                println!("找到相关表: {}", selector);
                break;
            }
        }

        println!("未找到Object Stats表");
    }
}
