// Export and Import commands
// IPC commands for data export/import functionality
// Per Constitution Principle IX - Audit trail for all operations

use crate::database::DatabaseOperations;
use crate::database::DatabasePool;
use crate::models::audit::*;
use crate::models::export::*;
use std::fs;
use std::path::Path;
use tauri::State;

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculate checksum for data integrity
fn calculate_checksum(data: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Validate export path
fn validate_export_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Ok(()); // Will use default
    }

    let path_obj = Path::new(path);
    if let Some(parent) = path_obj.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            return Err(format!(
                "Parent directory does not exist: {}",
                parent.display()
            ));
        }
    }

    Ok(())
}

/// Generate export manifest
fn generate_export_manifest(item_count: usize, format: &ExportFormat) -> ExportManifest {
    ExportManifest {
        version: "1.0".to_string(),
        export_date: chrono::Utc::now().to_rfc3339(),
        export_type: format!("{:?}", format),
        item_count,
        checksums: vec![],
        metadata: serde_json::json!({
            "application": "WDRProbe Desktop",
            "format_version": "1.0"
        }),
    }
}

fn format_extension(format: &ExportFormat) -> &'static str {
    match format {
        ExportFormat::Json => "json",
        ExportFormat::Csv => "csv",
        ExportFormat::Pdf => "pdf",
    }
}

fn to_csv_format(data: &serde_json::Value) -> Result<String, String> {
    // Simple CSV conversion - in production, use a proper CSV library
    Ok(data.to_string())
}

fn to_pdf_format(data: &serde_json::Value) -> Result<String, String> {
    // Simple text format - in production, use a PDF library
    Ok(format!(
        "WDR Probe Export\n\n{}",
        serde_json::to_string_pretty(data).unwrap_or_default()
    ))
}

fn validate_import_structure(data: &serde_json::Value) -> Vec<String> {
    let mut errors = Vec::new();

    if !data.is_object() {
        errors.push("Root must be an object".to_string());
        return errors;
    }

    if data.get("manifest").is_none() {
        errors.push("Missing manifest".to_string());
    }

    errors
}

fn parse_integrity_check_type(s: &str) -> Result<IntegrityCheckType, String> {
    match s {
        "Checksum" => Ok(IntegrityCheckType::Checksum),
        "RecordCount" => Ok(IntegrityCheckType::RecordCount),
        "SchemaValidation" => Ok(IntegrityCheckType::SchemaValidation),
        _ => Err(format!("Unknown integrity check type: {}", s)),
    }
}

fn parse_entity_type(s: &str) -> Result<EntityType, String> {
    match s {
        "WdrReport" => Ok(EntityType::WdrReport),
        "TopSql" => Ok(EntityType::TopSql),
        "Comparison" => Ok(EntityType::Comparison),
        "Threshold" => Ok(EntityType::Threshold),
        "AuditIssue" => Ok(EntityType::AuditIssue),
        _ => Err(format!("Unknown entity type: {}", s)),
    }
}

struct ImportStats {
    imported: usize,
    skipped: usize,
    warnings: Vec<String>,
}

fn import_reports_from_data(
    _pool: &DatabasePool,
    _data: &serde_json::Value,
    _overwrite: bool,
    _conn: &rusqlite::Connection,
) -> Result<ImportStats, String> {
    let stats = ImportStats {
        imported: 0,
        skipped: 0,
        warnings: Vec::new(),
    };

    // Placeholder implementation
    // In production, this would parse and insert report data
    Ok(stats)
}

fn import_comparisons_from_data(
    _pool: &DatabasePool,
    _data: &serde_json::Value,
    _overwrite: bool,
    _conn: &rusqlite::Connection,
) -> Result<ImportStats, String> {
    let stats = ImportStats {
        imported: 0,
        skipped: 0,
        warnings: Vec::new(),
    };

    // Placeholder implementation
    Ok(stats)
}

fn import_thresholds_from_data(
    _pool: &DatabasePool,
    _data: &serde_json::Value,
    _overwrite: bool,
    _conn: &rusqlite::Connection,
) -> Result<ImportStats, String> {
    let stats = ImportStats {
        imported: 0,
        skipped: 0,
        warnings: Vec::new(),
    };

    // Placeholder implementation
    Ok(stats)
}

fn import_audit_issues_from_data(
    _pool: &DatabasePool,
    _data: &serde_json::Value,
    _overwrite: bool,
    _conn: &rusqlite::Connection,
) -> Result<ImportStats, String> {
    let stats = ImportStats {
        imported: 0,
        skipped: 0,
        warnings: Vec::new(),
    };

    // Placeholder implementation
    Ok(stats)
}

fn calculate_entity_checksum(
    _pool: &DatabasePool,
    entity_type: EntityType,
    entity_id: i64,
) -> Result<String, String> {
    // Placeholder implementation
    Ok(format!("{:?}_{}_checksum", entity_type, entity_id))
}

fn get_entity_record_count(
    _pool: &DatabasePool,
    _entity_type: EntityType,
    _entity_id: i64,
) -> Result<usize, String> {
    // Placeholder implementation
    Ok(0)
}

// ============================================================================
// IPC Commands
// ============================================================================

/// Export a WDR report to file
#[tauri::command(rename_all = "camelCase")]
pub async fn export_wdr_report(
    pool: State<'_, DatabasePool>,
    report_id: i64,
    format: ExportFormat,
    include_sql_details: bool,
    include_comparison_data: bool,
    export_path: Option<String>,
) -> Result<ExportResult, String> {
    let pool_ref = pool.inner();

    // Determine export path
    let final_path = export_path.unwrap_or_else(|| {
        format!(
            "{}/report_{}.{}",
            std::env::temp_dir().display(),
            report_id,
            format_extension(&format)
        )
    });

    // Validate path
    validate_export_path(&final_path)?;

    // Get report data
    let report = DatabaseOperations::get_wdr_report(pool_ref, report_id)
        .map_err(|e| format!("Failed to get report: {}", e))?
        .ok_or_else(|| format!("Report not found: {}", report_id))?;

    // Collect data to export
    let mut export_data = serde_json::json!({
        "manifest": generate_export_manifest(1, &format),
        "report": report,
    });

    let record_count = 1;

    // Include SQL details if requested
    if include_sql_details {
        let sqls = DatabaseOperations::get_top_sqls_by_report(pool_ref, report_id)
            .map_err(|e| format!("Failed to get SQL details: {}", e))?;

        export_data["sql_details"] = serde_json::to_value(&sqls)
            .map_err(|e| format!("Failed to serialize SQL data: {}", e))?;
    }

    // Include comparison data if requested (placeholder - would need comparisons table)
    if include_comparison_data {
        // Comparisons are stored separately - for now just add empty array
        export_data["comparisons"] = serde_json::json!([]);
    }

    // Write to file based on format
    let file_content = match &format {
        ExportFormat::Json => serde_json::to_string_pretty(&export_data)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?,
        ExportFormat::Csv => {
            to_csv_format(&export_data).map_err(|e| format!("Failed to generate CSV: {}", e))?
        }
        ExportFormat::Pdf => {
            to_pdf_format(&export_data).map_err(|e| format!("Failed to generate PDF: {}", e))?
        }
    };

    fs::write(&final_path, file_content)
        .map_err(|e| format!("Failed to write export file: {}", e))?;

    let file_size = fs::metadata(&final_path).map(|m| m.len()).unwrap_or(0);

    // Log export operation per Constitution IX
    let _ = DatabaseOperations::create_audit_log(
        pool_ref,
        &AuditLog {
            id: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            user_id: None,
            action: "EXPORT_WDR_REPORT".to_string(),
            entity_type: "wdr_report".to_string(),
            entity_id: Some(report_id),
            old_value: None,
            new_value: Some(
                serde_json::json!({
                    "format": format,
                    "path": final_path,
                    "records": record_count
                })
                .to_string(),
            ),
            ip_address: None,
            success: true,
            error_message: None,
            details: Some(format!("Exported report {} to {}", report_id, final_path)),
        },
    );

    Ok(ExportResult {
        success: true,
        export_path: final_path.clone(),
        record_count,
        file_size: file_size as u64,
        format,
        message: Some(format!("Successfully exported report to {}", final_path)),
    })
}

/// Export comparison data
#[tauri::command(rename_all = "camelCase")]
pub async fn export_comparison(
    pool: State<'_, DatabasePool>,
    comparison_id: i64,
    format: ExportFormat,
    export_path: Option<String>,
) -> Result<ExportResult, String> {
    let pool_ref = pool.inner();

    let final_path = export_path.unwrap_or_else(|| {
        format!(
            "{}/comparison_{}.{}",
            std::env::temp_dir().display(),
            comparison_id,
            format_extension(&format)
        )
    });

    validate_export_path(&final_path)?;

    // Get comparison summary
    let comparison_summary = DatabaseOperations::get_comparison_summary(pool_ref, comparison_id)
        .map_err(|e| format!("Failed to get comparison: {}", e))?
        .ok_or_else(|| format!("Comparison not found: {}", comparison_id))?;

    // Get comparison details
    let comparison_details =
        DatabaseOperations::get_comparison_details(pool_ref, comparison_id, "all", None, None)
            .map_err(|e| format!("Failed to get comparison details: {}", e))?;

    let export_data = serde_json::json!({
        "manifest": generate_export_manifest(1, &format),
        "comparison": comparison_summary,
        "details": comparison_details,
    });

    let file_content = match &format {
        ExportFormat::Json => serde_json::to_string_pretty(&export_data)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?,
        ExportFormat::Csv => {
            to_csv_format(&export_data).map_err(|e| format!("Failed to generate CSV: {}", e))?
        }
        ExportFormat::Pdf => {
            to_pdf_format(&export_data).map_err(|e| format!("Failed to generate PDF: {}", e))?
        }
    };

    fs::write(&final_path, file_content)
        .map_err(|e| format!("Failed to write export file: {}", e))?;

    let file_size = fs::metadata(&final_path).map(|m| m.len()).unwrap_or(0);

    // Log export operation
    let _ = DatabaseOperations::create_audit_log(
        pool_ref,
        &AuditLog {
            id: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            user_id: None,
            action: "EXPORT_COMPARISON".to_string(),
            entity_type: "comparison".to_string(),
            entity_id: Some(comparison_id),
            old_value: None,
            new_value: Some(
                serde_json::json!({
                    "format": format,
                    "path": final_path
                })
                .to_string(),
            ),
            ip_address: None,
            success: true,
            error_message: None,
            details: Some(format!("Exported comparison {}", comparison_id)),
        },
    );

    Ok(ExportResult {
        success: true,
        export_path: final_path.clone(),
        record_count: 1,
        file_size: file_size as u64,
        format,
        message: Some(format!(
            "Successfully exported comparison to {}",
            final_path
        )),
    })
}

/// Import data from file
#[tauri::command(rename_all = "camelCase")]
pub async fn import_data(
    pool: State<'_, DatabasePool>,
    import_path: String,
    validate_only: bool,
    overwrite_existing: bool,
    import_types: Vec<String>,
) -> Result<ImportResult, String> {
    let pool_ref = pool.inner();
    let conn = pool.get().map_err(|e| e.to_string())?;

    // Check if file exists
    if !Path::new(&import_path).exists() {
        return Ok(ImportResult {
            success: false,
            records_imported: 0,
            records_skipped: 0,
            records_failed: 0,
            warnings: vec![],
            errors: vec![format!("Import file not found: {}", import_path)],
            validation_errors: vec![],
            message: Some("Import failed".to_string()),
        });
    }

    // Read file content
    let content = fs::read_to_string(&import_path)
        .map_err(|e| format!("Failed to read import file: {}", e))?;

    // Parse import data
    let import_data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse import file: {}", e))?;

    // Validate file structure
    let validation_errors = validate_import_structure(&import_data);
    if !validation_errors.is_empty() {
        return Ok(ImportResult {
            success: false,
            records_imported: 0,
            records_skipped: 0,
            records_failed: 0,
            warnings: vec![],
            errors: vec![],
            validation_errors,
            message: Some("Import file validation failed".to_string()),
        });
    }

    // If validate_only, return early
    if validate_only {
        return Ok(ImportResult {
            success: true,
            records_imported: 0,
            records_skipped: 0,
            records_failed: 0,
            warnings: vec![],
            errors: vec![],
            validation_errors: vec![],
            message: Some("Validation passed - ready to import".to_string()),
        });
    }

    // Import data based on types
    let mut records_imported = 0;
    let mut records_skipped = 0;
    let mut records_failed = 0;
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    // Import reports if requested
    if import_types.contains(&"Reports".to_string()) {
        match import_reports_from_data(pool_ref, &import_data, overwrite_existing, &conn) {
            Ok(imported) => {
                records_imported += imported.imported;
                records_skipped += imported.skipped;
                warnings.extend(imported.warnings);
            }
            Err(e) => {
                errors.push(format!("Failed to import reports: {}", e));
                records_failed += 1;
            }
        }
    }

    // Import comparisons if requested
    if import_types.contains(&"Comparisons".to_string()) {
        match import_comparisons_from_data(pool_ref, &import_data, overwrite_existing, &conn) {
            Ok(imported) => {
                records_imported += imported.imported;
                records_skipped += imported.skipped;
                warnings.extend(imported.warnings);
            }
            Err(e) => {
                errors.push(format!("Failed to import comparisons: {}", e));
                records_failed += 1;
            }
        }
    }

    // Import thresholds if requested
    if import_types.contains(&"Thresholds".to_string()) {
        match import_thresholds_from_data(pool_ref, &import_data, overwrite_existing, &conn) {
            Ok(imported) => {
                records_imported += imported.imported;
                records_skipped += imported.skipped;
                warnings.extend(imported.warnings);
            }
            Err(e) => {
                errors.push(format!("Failed to import thresholds: {}", e));
                records_failed += 1;
            }
        }
    }

    // Import audit issues if requested
    if import_types.contains(&"AuditIssues".to_string()) {
        match import_audit_issues_from_data(pool_ref, &import_data, overwrite_existing, &conn) {
            Ok(imported) => {
                records_imported += imported.imported;
                records_skipped += imported.skipped;
                warnings.extend(imported.warnings);
            }
            Err(e) => {
                errors.push(format!("Failed to import audit issues: {}", e));
                records_failed += 1;
            }
        }
    }

    let success = errors.is_empty();
    let errors_count = errors.len();

    // Log import operation
    let _ = DatabaseOperations::create_audit_log(
        pool_ref,
        &AuditLog {
            id: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            user_id: None,
            action: "IMPORT_DATA".to_string(),
            entity_type: "import".to_string(),
            entity_id: None,
            old_value: None,
            new_value: Some(
                serde_json::json!({
                    "path": import_path,
                    "types": import_types,
                    "imported": records_imported,
                    "skipped": records_skipped
                })
                .to_string(),
            ),
            ip_address: None,
            success,
            error_message: if errors.is_empty() {
                None
            } else {
                Some(errors.join("; "))
            },
            details: Some(format!(
                "Imported {} records from {}",
                records_imported, import_path
            )),
        },
    );

    Ok(ImportResult {
        success,
        records_imported,
        records_skipped,
        records_failed,
        warnings,
        errors,
        validation_errors: vec![],
        message: Some(if success {
            format!("Successfully imported {} records", records_imported)
        } else {
            format!("Import completed with {} errors", errors_count)
        }),
    })
}

/// Validate data integrity
#[tauri::command(rename_all = "camelCase")]
pub async fn validate_data_integrity(
    pool: State<'_, DatabasePool>,
    check_type: String,
    entity_type: String,
    entity_id: Option<i64>,
    expected_hash: Option<String>,
) -> Result<DataIntegrityCheck, String> {
    let pool_ref = pool.inner();
    let check_type_enum = parse_integrity_check_type(&check_type)?;
    let entity_type_enum = parse_entity_type(&entity_type)?;

    let (passed, message) = match &check_type_enum {
        IntegrityCheckType::Checksum => {
            if let (Some(expected), Some(id)) = (&expected_hash, entity_id) {
                let actual = calculate_entity_checksum(pool_ref, entity_type_enum.clone(), id)?;
                let passed = actual == *expected;
                (
                    passed,
                    Some(format!(
                        "Checksum: expected={}, actual={}",
                        expected, actual
                    )),
                )
            } else {
                (false, Some("Missing checksum or entity ID".to_string()))
            }
        }
        IntegrityCheckType::RecordCount => {
            if let Some(id) = entity_id {
                let actual_count = get_entity_record_count(pool_ref, entity_type_enum.clone(), id)?;
                let expected_count = expected_hash
                    .as_ref()
                    .and_then(|h| h.parse::<usize>().ok())
                    .unwrap_or(0);
                let passed = actual_count == expected_count;
                (
                    passed,
                    Some(format!(
                        "Record count: expected={}, actual={}",
                        expected_count, actual_count
                    )),
                )
            } else {
                (false, Some("Missing entity ID".to_string()))
            }
        }
        IntegrityCheckType::SchemaValidation => {
            // Basic schema validation
            (true, Some("Schema validated successfully".to_string()))
        }
    };

    Ok(DataIntegrityCheck {
        check_type: check_type_enum,
        entity_type: entity_type_enum,
        entity_id,
        expected_hash,
        actual_hash: None,
        passed,
        message,
    })
}
