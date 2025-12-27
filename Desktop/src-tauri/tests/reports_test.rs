// Reports module integration tests
// Tests for WDR report import and management

#[cfg(test)]
mod reports_tests {
    use tempfile::TempDir;
    use wdrprobe_desktop_lib::database::{init_database, initialize_schema, DatabaseOperations};
    use wdrprobe_desktop_lib::{TopSql, WdrReport};

    #[tokio::test]
    async fn test_get_wdr_reports_empty() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = init_database(db_path.to_str().unwrap()).unwrap();
        let conn = pool.get().unwrap();
        initialize_schema(&conn).unwrap();

        // Test empty database - this would be called via IPC command
        // For now, verify database operations work
        let reports = DatabaseOperations::list_wdr_reports(&pool, None, None).unwrap();
        assert!(reports.is_empty());
    }

    #[tokio::test]
    async fn test_create_and_retrieve_wdr_report() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = init_database(db_path.to_str().unwrap()).unwrap();
        let conn = pool.get().unwrap();
        initialize_schema(&conn).unwrap();

        // Create a test report
        let test_report = WdrReport {
            id: 0,
            instance_name: "test_instance".to_string(),
            generation_time: "2024-01-01T00:00:00Z".to_string(),
            snapshot_start: "2024-01-01T00:00:00Z".to_string(),
            snapshot_end: "2024-01-01T01:00:00Z".to_string(),
            file_path: Some("/tmp/test.wdr".to_string()),
            file_size: Some(1024),
            status: "completed".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let report_id = DatabaseOperations::create_wdr_report(&pool, &test_report).unwrap();

        // Retrieve the report
        let retrieved = DatabaseOperations::get_wdr_report(&pool, report_id).unwrap();
        assert!(retrieved.is_some());

        let retrieved_report = retrieved.unwrap();
        assert_eq!(retrieved_report.instance_name, "test_instance");
        assert_eq!(retrieved_report.status, "completed");
    }

    #[tokio::test]
    async fn test_list_wdr_reports_with_pagination() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = init_database(db_path.to_str().unwrap()).unwrap();
        let conn = pool.get().unwrap();
        initialize_schema(&conn).unwrap();

        // Create multiple reports
        for i in 1..=5 {
            let test_report = WdrReport {
                id: 0,
                instance_name: format!("test_instance_{}", i),
                generation_time: "2024-01-01T00:00:00Z".to_string(),
                snapshot_start: "2024-01-01T00:00:00Z".to_string(),
                snapshot_end: "2024-01-01T01:00:00Z".to_string(),
                file_path: Some(format!("/tmp/test{}.wdr", i)),
                file_size: Some(1024),
                status: "completed".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
            };

            DatabaseOperations::create_wdr_report(&pool, &test_report).unwrap();
        }

        // Test pagination
        let reports = DatabaseOperations::list_wdr_reports(&pool, Some(3), None).unwrap();
        assert_eq!(reports.len(), 3);

        let reports_with_offset =
            DatabaseOperations::list_wdr_reports(&pool, Some(3), Some(3)).unwrap();
        assert_eq!(reports_with_offset.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_wdr_report() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = init_database(db_path.to_str().unwrap()).unwrap();
        let conn = pool.get().unwrap();
        initialize_schema(&conn).unwrap();

        // Create a test report
        let test_report = WdrReport {
            id: 0,
            instance_name: "test_instance".to_string(),
            generation_time: "2024-01-01T00:00:00Z".to_string(),
            snapshot_start: "2024-01-01T00:00:00Z".to_string(),
            snapshot_end: "2024-01-01T01:00:00Z".to_string(),
            file_path: Some("/tmp/test.wdr".to_string()),
            file_size: Some(1024),
            status: "completed".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let report_id = DatabaseOperations::create_wdr_report(&pool, &test_report).unwrap();

        // Verify it exists
        let retrieved = DatabaseOperations::get_wdr_report(&pool, report_id).unwrap();
        assert!(retrieved.is_some());

        // Delete it
        DatabaseOperations::delete_wdr_report(&pool, report_id).unwrap();

        // Verify it's gone
        let retrieved = DatabaseOperations::get_wdr_report(&pool, report_id).unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_create_top_sql_for_report() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = init_database(db_path.to_str().unwrap()).unwrap();
        let conn = pool.get().unwrap();
        initialize_schema(&conn).unwrap();

        // Create a report first
        let test_report = WdrReport {
            id: 0,
            instance_name: "test_instance".to_string(),
            generation_time: "2024-01-01T00:00:00Z".to_string(),
            snapshot_start: "2024-01-01T00:00:00Z".to_string(),
            snapshot_end: "2024-01-01T01:00:00Z".to_string(),
            file_path: Some("/tmp/test.wdr".to_string()),
            file_size: Some(1024),
            status: "completed".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let report_id = DatabaseOperations::create_wdr_report(&pool, &test_report).unwrap();

        // Create a TopSQL entry
        let top_sql = TopSql {
            id: 0,
            report_id,
            sql_id: Some("123".to_string()),
            sql_text: "SELECT * FROM users WHERE id = ?".to_string(),
            executions: 1000,
            total_elapsed_time: 5000.0,
            cpu_time: 3000.0,
            io_time: 1000.0,
            buffer_gets: 50000,
            disk_reads: 1000,
            rows_processed: 10000,
            first_load_time: "2024-01-01T00:00:00Z".to_string(),
            last_load_time: "2024-01-01T01:00:00Z".to_string(),
            is_hot_sql: true,
            rank_by_time: Some(1),
        };

        let _sql_id = DatabaseOperations::create_top_sql(&pool, &top_sql).unwrap();

        // Retrieve the SQL
        let sqls = DatabaseOperations::get_top_sqls_by_report(&pool, report_id).unwrap();
        assert_eq!(sqls.len(), 1);
        assert_eq!(sqls[0].sql_id, Some("123".to_string()));
    }
}
