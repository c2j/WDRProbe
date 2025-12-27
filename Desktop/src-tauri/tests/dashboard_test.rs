// Dashboard integration tests
// Tests for dashboard IPC commands

#[cfg(test)]
mod dashboard_tests {
    use tempfile::TempDir;
    use wdrprobe_desktop_lib::database::{init_database, initialize_schema, DatabaseOperations};
    use wdrprobe_desktop_lib::WdrReport;

    #[tokio::test]
    async fn test_get_instance_summaries_empty_database() {
        // Setup temporary database
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Initialize database and schema
        let pool = init_database(db_path.to_str().unwrap()).unwrap();
        let conn = pool.get().unwrap();
        initialize_schema(&conn).unwrap();

        // Test that no instances exist in empty database
        let instances = DatabaseOperations::get_instance_summaries(&pool).unwrap();
        assert!(instances.is_empty());
    }

    #[tokio::test]
    async fn test_get_instance_summaries_with_data() {
        // Setup temporary database
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Initialize database and schema
        let pool = init_database(db_path.to_str().unwrap()).unwrap();
        let conn = pool.get().unwrap();
        initialize_schema(&conn).unwrap();

        // Insert test data
        let test_report = WdrReport {
            id: 0,
            instance_name: "test_instance_1".to_string(),
            generation_time: "2024-01-01T00:00:00Z".to_string(),
            snapshot_start: "2024-01-01T00:00:00Z".to_string(),
            snapshot_end: "2024-01-01T01:00:00Z".to_string(),
            file_path: Some("/tmp/test.wdr".to_string()),
            file_size: Some(1024),
            status: "completed".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        DatabaseOperations::create_wdr_report(&pool, &test_report).unwrap();

        // Test with data
        let instances = DatabaseOperations::get_instance_summaries(&pool).unwrap();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].instance_name, "test_instance_1");
        assert_eq!(instances[0].report_count, 1);
    }

    #[tokio::test]
    async fn test_dashboard_metrics_calculation() {
        // Setup temporary database
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Initialize database and schema
        let pool = init_database(db_path.to_str().unwrap()).unwrap();
        let conn = pool.get().unwrap();
        initialize_schema(&conn).unwrap();

        // Insert test data
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

        DatabaseOperations::create_wdr_report(&pool, &test_report).unwrap();

        // Test metrics calculation
        let metrics = DatabaseOperations::get_dashboard_metrics(&pool, None).unwrap();
        assert!(metrics.instance_name.is_none()); // No filter
        assert!(metrics.cpu_usage_percent >= 0.0);
        assert!(metrics.memory_usage_percent >= 0.0);
        assert!(metrics.tps_raw >= 0.0);
        assert!(metrics.qps_raw >= 0.0);

        // Check formatted values
        assert!(metrics.cpu.ends_with('%'));
        assert!(metrics.mem.ends_with('%'));
        assert!(metrics.tps.ends_with('k'));
        assert!(metrics.qps.ends_with('k'));

        // Check health distribution
        assert_eq!(metrics.health_distribution.len(), 3);
        assert!(metrics.trend_data.len() > 0);
        assert!(metrics.hot_issues.len() > 0);
    }

    #[tokio::test]
    async fn test_dashboard_metrics_with_instance_filter() {
        // Setup temporary database
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Initialize database and schema
        let pool = init_database(db_path.to_str().unwrap()).unwrap();
        let conn = pool.get().unwrap();
        initialize_schema(&conn).unwrap();

        // Insert test data
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

        DatabaseOperations::create_wdr_report(&pool, &test_report).unwrap();

        // Test metrics with instance filter
        let instance_name = Some("test_instance");
        let metrics = DatabaseOperations::get_dashboard_metrics(&pool, instance_name).unwrap();
        assert_eq!(
            metrics.instance_name,
            Some(instance_name.unwrap().to_string())
        );
    }

    #[tokio::test]
    async fn test_multiple_instances() {
        // Setup temporary database
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Initialize database and schema
        let pool = init_database(db_path.to_str().unwrap()).unwrap();
        let conn = pool.get().unwrap();
        initialize_schema(&conn).unwrap();

        // Insert test data for multiple instances
        for i in 1..=3 {
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

        // Test multiple instances
        let instances = DatabaseOperations::get_instance_summaries(&pool).unwrap();
        assert_eq!(instances.len(), 3);

        // Verify each instance has correct count
        for i in 1..=3 {
            let instance_name = format!("test_instance_{}", i);
            let summary = instances
                .iter()
                .find(|s| s.instance_name == instance_name)
                .unwrap();
            assert_eq!(summary.report_count, 1);
        }
    }
}
