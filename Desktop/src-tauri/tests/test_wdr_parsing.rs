// 测试WDR解析器修复
#[cfg(test)]
mod tests {
    use wdrprobe_desktop_lib::parsers::wdr_parser::parse_top_sqls;
    use wdrprobe_desktop_lib::utils::WdrProbeError;

    #[test]
    fn test_parse_wdr_sqls() {
        let file_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../example/opengauss_v1.html");

        if !file_path.exists() {
            println!("Skipping test: Example file not found at {}", file_path.display());
            return;
        }

        match parse_top_sqls(file_path.to_str().unwrap()) {
            Ok(sqls) => {
                println!("成功解析 {} 条SQL", sqls.len());

                // 打印前5条SQL的详细信息
                for (i, sql) in sqls.iter().take(5).enumerate() {
                    println!("SQL {}:", i + 1);
                    println!("  ID: {:?}", sql.sql_id);
                    println!("  文本: {}", &sql.sql_text[..sql.sql_text.len().min(80)]);
                    println!("  执行次数: {}", sql.executions);
                    println!("  总耗时: {}", sql.total_elapsed_time);
                    println!("  CPU时间: {}", sql.cpu_time);
                    println!("  逻辑读: {}", sql.buffer_gets);
                    println!("  物理读: {}", sql.disk_reads);
                    println!("  返回行数: {}", sql.rows_processed);
                    println!();
                }

                // 验证至少解析了一些SQL
                assert!(!sqls.is_empty(), "应该解析到至少一条SQL");

                // 验证SQL文本不为空
                let sqls_with_text: Vec<_> =
                    sqls.iter().filter(|s| !s.sql_text.is_empty()).collect();

                println!("有文本的SQL数量: {}/{}", sqls_with_text.len(), sqls.len());
                assert!(!sqls_with_text.is_empty(), "至少应该有一些SQL包含文本");
            }
            Err(e) => {
                panic!("解析失败: {:?}", e);
            }
        }
    }
}
