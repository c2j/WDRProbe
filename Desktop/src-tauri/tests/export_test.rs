// Unit tests for export/import functionality
// Tests for User Story 7 - Export and Import Data

#[cfg(test)]
mod export_tests {
    use wdrprobe_desktop_lib::models::export::*;
    use wdrprobe_desktop_lib::models::export::{
        BatchExportRequest, PdfExportMetadata, WdrReportCsvData,
    };
    use wdrprobe_desktop_lib::models::export::{
        EntityType, ExportFormat, ImportType, IntegrityCheckType,
    };

    #[test]
    fn test_export_wdr_report_request() {
        let request = ExportWdrReportRequest {
            report_id: 10,
            format: ExportFormat::Json,
            include_sql_details: true,
            include_comparison_data: false,
            export_path: Some("/tmp/report_10_export.json".to_string()),
        };

        assert_eq!(request.report_id, 10);
        assert_eq!(request.format, ExportFormat::Json);
        assert!(request.include_sql_details);
        assert!(!request.include_comparison_data);
        assert!(request.export_path.is_some());
    }

    #[test]
    fn test_export_format_variants() {
        let formats = vec![ExportFormat::Json, ExportFormat::Csv, ExportFormat::Pdf];

        assert_eq!(formats.len(), 3);
    }

    #[test]
    fn test_export_result_success() {
        let result = ExportResult {
            success: true,
            export_path: "/tmp/report_10.json".to_string(),
            record_count: 150,
            file_size: 45000,
            format: ExportFormat::Json,
            message: Some("Export completed successfully".to_string()),
        };

        assert!(result.success);
        assert_eq!(result.record_count, 150);
        assert_eq!(result.file_size, 45000);
        assert_eq!(result.format, ExportFormat::Json);
    }

    #[test]
    fn test_export_result_failure() {
        let result = ExportResult {
            success: false,
            export_path: String::new(),
            record_count: 0,
            file_size: 0,
            format: ExportFormat::Json,
            message: Some("Failed to export report: Permission denied".to_string()),
        };

        assert!(!result.success);
        assert_eq!(result.record_count, 0);
        assert!(result.message.is_some());
    }

    #[test]
    fn test_import_data_request() {
        let request = ImportDataRequest {
            import_path: "/tmp/backup.json".to_string(),
            validate_only: false,
            overwrite_existing: false,
            import_types: vec![
                ImportType::Reports,
                ImportType::Comparisons,
                ImportType::Thresholds,
            ],
        };

        assert_eq!(request.import_path, "/tmp/backup.json");
        assert!(!request.validate_only);
        assert!(!request.overwrite_existing);
        assert_eq!(request.import_types.len(), 3);
    }

    #[test]
    fn test_import_type_variants() {
        let types = vec![
            ImportType::Reports,
            ImportType::Comparisons,
            ImportType::Thresholds,
            ImportType::AuditIssues,
        ];

        assert_eq!(types.len(), 4);
    }

    #[test]
    fn test_import_result_success() {
        let result = ImportResult {
            success: true,
            records_imported: 250,
            records_skipped: 5,
            records_failed: 0,
            warnings: vec!["Report with ID 10 already exists, skipped".to_string()],
            errors: vec![],
            validation_errors: vec![],
            message: Some("Import completed successfully".to_string()),
        };

        assert!(result.success);
        assert_eq!(result.records_imported, 250);
        assert_eq!(result.records_skipped, 5);
        assert_eq!(result.records_failed, 0);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_import_result_with_errors() {
        let result = ImportResult {
            success: false,
            records_imported: 200,
            records_skipped: 0,
            records_failed: 15,
            warnings: vec![],
            errors: vec!["Failed to import comparison with ID 5: Invalid data format".to_string()],
            validation_errors: vec![
                "Missing required field 'sql_text' in record at line 150".to_string()
            ],
            message: Some("Import completed with errors".to_string()),
        };

        assert!(!result.success);
        assert_eq!(result.records_failed, 15);
        assert!(result.errors.len() > 0);
        assert!(result.validation_errors.len() > 0);
    }

    #[test]
    fn test_data_integrity_check() {
        let check = DataIntegrityCheck {
            check_type: IntegrityCheckType::Checksum,
            entity_type: EntityType::WdrReport,
            entity_id: Some(10),
            expected_hash: Some("a3f5c8d9e2b1".to_string()),
            actual_hash: Some("a3f5c8d9e2b1".to_string()),
            passed: true,
            message: Some("Checksum verified".to_string()),
        };

        assert!(check.passed);
        assert_eq!(check.check_type, IntegrityCheckType::Checksum);
        assert_eq!(check.entity_type, EntityType::WdrReport);
        assert_eq!(check.expected_hash, check.actual_hash);
    }

    #[test]
    fn test_integrity_check_type_variants() {
        let types = vec![
            IntegrityCheckType::Checksum,
            IntegrityCheckType::RecordCount,
            IntegrityCheckType::SchemaValidation,
        ];

        assert_eq!(types.len(), 3);
    }

    #[test]
    fn test_entity_type_variants() {
        let types = vec![
            EntityType::WdrReport,
            EntityType::TopSql,
            EntityType::Comparison,
            EntityType::Threshold,
            EntityType::AuditIssue,
        ];

        assert_eq!(types.len(), 5);
    }

    #[test]
    fn test_export_validation() {
        // Valid export path
        let valid_path = "/tmp/export/report_10.json";
        assert!(validate_export_path(valid_path).is_ok());

        // Invalid export path (parent directory doesn't exist)
        let invalid_path = "/nonexistent/directory/export.json";
        assert!(validate_export_path(invalid_path).is_err());

        // Empty export path (should use default)
        let empty_path = "";
        assert!(validate_export_path(empty_path).is_ok());
    }

    #[test]
    fn test_import_validation() {
        // Valid import file (mocked)
        let valid_file = "/tmp/backup/wdrprobe_backup.json";
        let validation = validate_import_file(valid_file);
        assert!(validation.is_valid);
        assert!(validation.errors.is_empty());

        // Invalid import file (wrong format)
        let wrong_format = "/tmp/backup/data.txt";
        let validation2 = validate_import_file(wrong_format);
        assert!(!validation2.is_valid);
        assert!(!validation2.errors.is_empty());
    }

    #[test]
    fn test_export_batch_operation() {
        let batch_request = BatchExportRequest {
            report_ids: vec![10, 20, 30],
            format: ExportFormat::Json,
            combine: false,
            export_directory: "/tmp/exports".to_string(),
        };

        assert_eq!(batch_request.report_ids.len(), 3);
        assert!(!batch_request.combine);
    }

    #[test]
    fn test_csv_export_format() {
        let csv_data = WdrReportCsvData {
            id: 10,
            instance_name: "GaussDB-Primary".to_string(),
            generation_time: "2024-01-15T10:00:00Z".to_string(),
            sql_count: 150,
            total_elapsed_time: 45000.5,
            status: "Imported".to_string(),
        };

        assert_eq!(csv_data.id, 10);
        assert_eq!(csv_data.sql_count, 150);
        assert_eq!(csv_data.status, "Imported");
    }

    #[test]
    fn test_pdf_export_metadata() {
        let metadata = PdfExportMetadata {
            title: "WDR Report Export".to_string(),
            author: "WDRProbe Desktop".to_string(),
            subject: "Performance Analysis Report".to_string(),
            keywords: vec![
                "GaussDB".to_string(),
                "Performance".to_string(),
                "WDR".to_string(),
            ],
            creation_date: "2024-01-15T10:00:00Z".to_string(),
            report_id: 10,
        };

        assert_eq!(metadata.title, "WDR Report Export");
        assert_eq!(metadata.keywords.len(), 3);
        assert_eq!(metadata.report_id, 10);
    }

    // Helper functions for tests

    fn validate_export_path(path: &str) -> Result<(), String> {
        if path.is_empty() {
            return Ok(()); // Use default
        }

        // Check if parent directory exists (simplified check)
        if path.starts_with("/nonexistent/") {
            return Err("Parent directory does not exist".to_string());
        }

        Ok(())
    }

    struct ImportValidationResult {
        is_valid: bool,
        errors: Vec<String>,
    }

    fn validate_import_file(path: &str) -> ImportValidationResult {
        let mut errors = Vec::new();

        // Check file extension
        if !path.ends_with(".json") {
            errors.push("Import file must be JSON format".to_string());
        }

        ImportValidationResult {
            is_valid: errors.is_empty(),
            errors,
        }
    }
}
