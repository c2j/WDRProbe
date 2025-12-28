// 测试完整的WDR解析器
#[cfg(test)]
mod tests {
    use wdrprobe_desktop_lib::parsers::complete_wdr_parser::parse_complete_wdr_report;

    #[test]
    fn test_complete_wdr_parsing() {
        let file_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../example/opengauss_v1.html");

        if !file_path.exists() {
            println!(
                "Skipping test: Example file not found at {}",
                file_path.display()
            );
            return;
        }

        match parse_complete_wdr_report(file_path.to_str().unwrap(), "test_instance".to_string()) {
            Ok(report) => {
                println!("=== 完整WDR解析测试成功 ===\n");

                println!("报告基本信息:");
                println!("  实例名: {}", report.report.instance_name);
                println!("  快照开始: {}", report.report.snapshot_start);
                println!("  快照结束: {}", report.report.snapshot_end);
                println!("  状态: {}", report.report.status);
                println!();

                println!("数据库统计 ({} 个数据库):", report.database_stats.len());
                for (i, db) in report.database_stats.iter().take(3).enumerate() {
                    println!(
                        "  {}: {} (连接: {}, 提交: {}, 回滚: {})",
                        i + 1,
                        db.db_name,
                        db.backends,
                        db.xact_commit,
                        db.xact_rollback
                    );
                }
                println!();

                println!("Load Profile:");
                println!("  DB Time(us/秒): {}", report.load_profile.db_time_per_sec);
                println!(
                    "  CPU Time(us/秒): {}",
                    report.load_profile.cpu_time_per_sec
                );
                println!(
                    "  IO请求(次/秒): {}",
                    report.load_profile.io_requests_per_sec
                );
                println!();

                println!("效率指标:");
                println!("  缓冲命中率: {}%", report.efficiency.buffer_hit_percent);
                println!("  CPU效率: {}%", report.efficiency.cpu_efficiency_percent);
                println!();

                println!("SQL统计 ({} 条SQL):", report.top_sql.len());
                for (i, sql) in report.top_sql.iter().take(5).enumerate() {
                    println!(
                        "  {}: {} (执行: {}, 总耗时: {}, CPU: {})",
                        i + 1,
                        &sql.sql_text[..sql.sql_text.len().min(60)],
                        sql.executions,
                        sql.total_elapsed_time,
                        sql.cpu_time
                    );
                }
                println!();

                println!("缓存IO统计 ({} 项):", report.cache_io_stats.len());
                for (i, cache) in report.cache_io_stats.iter().take(3).enumerate() {
                    println!(
                        "  {}: {}.{} (命中率: {}%)",
                        i + 1,
                        cache.schema_name,
                        cache.object_name,
                        cache.heap_blks_hit_ratio
                    );
                }
                println!();

                println!("Object统计 ({} 项):", report.object_stats.len());
                for (i, obj) in report.object_stats.iter().take(3).enumerate() {
                    println!(
                        "  {}: {}.{} (类型: {}, 扫描: {})",
                        i + 1,
                        obj.schema_name,
                        obj.object_name,
                        obj.object_type,
                        obj.total_scans
                    );
                }
                println!();

                // 验证
                assert!(!report.report.instance_name.is_empty(), "实例名不应为空");
                assert!(!report.database_stats.is_empty(), "应该有数据库统计");
                assert!(!report.top_sql.is_empty(), "应该有SQL统计");
                assert!(report.top_sql.len() > 0, "至少应该有一条SQL");

                println!("=== 所有验证通过! ===");
            }
            Err(e) => {
                panic!("解析失败: {:?}", e);
            }
        }
    }
}
