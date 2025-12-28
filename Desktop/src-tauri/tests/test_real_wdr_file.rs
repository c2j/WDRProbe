// 测试真实的用户WDR报告文件
#[cfg(test)]
mod tests {
    use wdrprobe_desktop_lib::parsers::complete_wdr_parser::parse_complete_wdr_report;

    #[test]
    fn test_real_wdr_file_parsing() {
        // 用户提供的WDR报告文件路径
        let file_path = "/Users/c2j/Desktop/GaussDB-Toolkit/1759244988894_F-PMAS_低负载.html";

        println!("=== 测试解析真实WDR报告 ===");
        println!("文件路径: {}", file_path);
        println!();

        if !std::path::Path::new(file_path).exists() {
            println!("Skipping test: User file not found at {}", file_path);
            return;
        }

        match parse_complete_wdr_report(file_path, "dn_6001_6002_6003".to_string()) {
            Ok(report) => {
                println!("=== WDR解析成功! ===\n");

                println!("【报告基本信息】");
                println!("  实例名: {}", report.report.instance_name);
                println!("  快照开始: {}", report.report.snapshot_start);
                println!("  快照结束: {}", report.report.snapshot_end);
                println!("  状态: {}", report.report.status);
                println!();

                println!("【效率指标】");
                println!("  缓冲命中率: {:.2}%", report.efficiency.buffer_hit_percent);
                println!(
                    "  CPU效率: {:.2}%",
                    report.efficiency.cpu_efficiency_percent
                );
                println!(
                    "  软解析率: {:.2}%",
                    report.efficiency.soft_parse_rate_percent
                );
                println!(
                    "  硬解析率: {:.2}%",
                    report.efficiency.hard_parse_rate_percent
                );
                println!();

                println!("【Load Profile】");
                println!(
                    "  DB Time(微秒/秒): {:.2}",
                    report.load_profile.db_time_per_sec
                );
                println!(
                    "  CPU Time(微秒/秒): {:.2}",
                    report.load_profile.cpu_time_per_sec
                );
                println!(
                    "  IO请求次数: {:.2}",
                    report.load_profile.io_requests_per_sec
                );
                println!();

                println!("【数据库统计】 ({} 个数据库):", report.database_stats.len());
                for (i, db) in report.database_stats.iter().take(5).enumerate() {
                    println!(
                        "  {}. {} - 连接: {}, 提交: {}, 回滚: {}, 块读: {}, 块命中: {}",
                        i + 1,
                        db.db_name,
                        db.backends,
                        db.xact_commit,
                        db.xact_rollback,
                        db.blks_read,
                        db.blks_hit
                    );
                }
                println!();

                println!(
                    "【SQL统计】 (共 {} 条SQL，显示前10条):",
                    report.top_sql.len()
                );
                for (i, sql) in report.top_sql.iter().take(10).enumerate() {
                    let sql_preview = if sql.sql_text.len() > 60 {
                        format!("{}...", &sql.sql_text[..60])
                    } else {
                        sql.sql_text.clone()
                    };
                    let sql_id_display = sql.sql_id.as_ref().map(|s| s.as_str()).unwrap_or("N/A");
                    println!(
                        "  {}. SQL_ID: {} | 执行: {}次 | 总耗时: {:.2}s | CPU: {:.2}s | {}",
                        i + 1,
                        sql_id_display,
                        sql.executions,
                        sql.total_elapsed_time / 1_000_000.0,
                        sql.cpu_time / 1_000_000.0,
                        sql_preview
                    );
                }
                println!();

                println!(
                    "【缓存IO统计】 ({} 项，显示前5项):",
                    report.cache_io_stats.len()
                );
                for (i, cache) in report.cache_io_stats.iter().take(5).enumerate() {
                    println!(
                        "  {}. {}.{} - 堆命中率: {:.2}%, 索引命中率: {:.2}%",
                        i + 1,
                        cache.schema_name,
                        cache.object_name,
                        cache.heap_blks_hit_ratio,
                        cache.idx_blks_hit_ratio
                    );
                }
                println!();

                println!(
                    "【对象统计】 ({} 项，显示前5项):",
                    report.object_stats.len()
                );
                for (i, obj) in report.object_stats.iter().take(5).enumerate() {
                    println!(
                        "  {}. {}.{} - 类型: {}, 总扫描: {}, 顺序扫描: {}, 索引扫描: {}",
                        i + 1,
                        obj.schema_name,
                        obj.object_name,
                        obj.object_type,
                        obj.total_scans,
                        obj.seq_scans,
                        obj.idx_scans
                    );
                }
                println!();

                // 验证数据完整性
                println!("【数据完整性验证】");
                assert!(!report.report.instance_name.is_empty(), "实例名不应为空");
                assert!(report.top_sql.len() > 0, "应该解析出SQL统计");

                println!("  ✓ 实例名: {}", report.report.instance_name);
                println!("  ✓ SQL数量: {}", report.top_sql.len());
                println!("  ✓ 数据库数量: {}", report.database_stats.len());
                println!("  ✓ 缓存IO数量: {}", report.cache_io_stats.len());
                println!("  ✓ 对象统计数量: {}", report.object_stats.len());
                println!();

                println!("=== 真实WDR报告解析测试通过! ===");
            }
            Err(e) => {
                panic!("解析失败: {:?}\n\n请确保文件存在且可访问: {}", e, file_path);
            }
        }
    }
}
