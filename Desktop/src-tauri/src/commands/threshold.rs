// Threshold commands
// IPC commands for threshold configuration
// Per Constitution Principle IV - DTO format with audit trail
// Per Constitution Principle IX - All changes logged to audit_logs

use wdrprobe_core::database::DatabaseOperations;
use wdrprobe_core::database::DatabasePool;
use wdrprobe_core::models::threshold::*;
use tauri::State;

/// Get all threshold configurations, optionally filtered by category
#[tauri::command(rename_all = "camelCase")]
pub async fn get_threshold_configs(
    pool: State<'_, DatabasePool>,
    category: Option<String>,
) -> Result<ThresholdConfigList, String> {
    let pool_ref = pool.inner();

    let thresholds = DatabaseOperations::get_threshold_configs(pool_ref, category.as_deref())
        .map_err(|e| format!("Failed to retrieve thresholds: {}", e))?;

    let total = thresholds.len() as i64;

    Ok(ThresholdConfigList { thresholds, total })
}

/// Get a specific threshold configuration by key
#[tauri::command(rename_all = "camelCase")]
pub async fn get_threshold_config(
    pool: State<'_, DatabasePool>,
    config_key: String,
) -> Result<ThresholdConfig, String> {
    let pool_ref = pool.inner();

    let threshold = DatabaseOperations::get_threshold_config(pool_ref, &config_key)
        .map_err(|e| format!("Failed to retrieve threshold: {}", e))?
        .ok_or_else(|| format!("Threshold not found: {}", config_key))?;

    Ok(threshold)
}

/// Update a single threshold value per Constitution IV DTO format
#[tauri::command(rename_all = "camelCase")]
pub async fn update_threshold(
    pool: State<'_, DatabasePool>,
    config_key: String,
    value: f64,
    changed_by: String,
    change_reason: String,
) -> Result<UpdateThresholdResult, String> {
    let pool_ref = pool.inner();

    // Create request for validation
    let request = ThresholdUpdateRequest {
        config_key: config_key.clone(),
        value,
        changed_by: changed_by.clone(),
        change_reason: change_reason.clone(),
    };

    // Validate DTO format per Constitution IV
    validate_threshold_update_dto(&request)?;

    // Get current threshold for validation and old value
    let current = DatabaseOperations::get_threshold_config(pool_ref, &config_key)
        .map_err(|e| format!("Failed to retrieve threshold: {}", e))?
        .ok_or_else(|| format!("Threshold not found: {}", config_key))?;

    // Validate value within bounds
    validate_value_within_bounds(&current, value)?;

    // Validate data type
    validate_data_type(&current, value)?;

    let old_value = current.value;

    // Update threshold
    DatabaseOperations::update_threshold_config(pool_ref, &config_key, value, &changed_by)
        .map_err(|e| format!("Failed to update threshold: {}", e))?;

    // Log to audit_logs per Constitution IX
    let audit_log = wdrprobe_core::models::AuditLog {
        id: 0,
        timestamp: chrono::Utc::now().to_rfc3339(),
        user_id: Some(changed_by.clone()),
        action: "ThresholdUpdate".to_string(),
        entity_type: "threshold".to_string(),
        entity_id: Some(current.id),
        old_value: Some(old_value.to_string()),
        new_value: Some(value.to_string()),
        ip_address: None,
        success: true,
        error_message: None,
        details: Some(change_reason.clone()),
    };

    let _ = DatabaseOperations::create_audit_log(pool_ref, &audit_log);

    Ok(UpdateThresholdResult {
        success: true,
        threshold_id: current.id,
        old_value,
        new_value: value,
        updated_at: chrono::Utc::now().to_rfc3339(),
        message: Some("Threshold updated successfully".to_string()),
    })
}

/// Update multiple thresholds in a single transaction
#[tauri::command(rename_all = "camelCase")]
pub async fn batch_update_thresholds(
    pool: State<'_, DatabasePool>,
    updates: Vec<ThresholdUpdate>,
    changed_by: String,
    change_reason: String,
) -> Result<BatchUpdateResult, String> {
    let pool_ref = pool.inner();

    // Validate batch request
    if updates.is_empty() {
        return Err("No updates provided".to_string());
    }

    if changed_by.trim().is_empty() {
        return Err("changed_by is required".to_string());
    }

    if change_reason.trim().is_empty() || change_reason.len() < 10 {
        return Err("change_reason is required and must be at least 10 characters".to_string());
    }

    if change_reason.len() > 500 {
        return Err("change_reason too long (max 500 characters)".to_string());
    }

    let mut failed_updates = Vec::new();
    let mut updated_count = 0;

    // Process each update
    for update in &updates {
        // Get current threshold
        let current = match DatabaseOperations::get_threshold_config(pool_ref, &update.config_key) {
            Ok(Some(threshold)) => threshold,
            Ok(None) => {
                failed_updates.push(FailedUpdate {
                    config_key: update.config_key.clone(),
                    error: "Threshold not found".to_string(),
                });
                continue;
            }
            Err(e) => {
                failed_updates.push(FailedUpdate {
                    config_key: update.config_key.clone(),
                    error: e.to_string(),
                });
                continue;
            }
        };

        // Validate value
        if let Err(e) = validate_value_within_bounds(&current, update.value) {
            failed_updates.push(FailedUpdate {
                config_key: update.config_key.clone(),
                error: e.to_string(),
            });
            continue;
        }

        if let Err(e) = validate_data_type(&current, update.value) {
            failed_updates.push(FailedUpdate {
                config_key: update.config_key.clone(),
                error: e.to_string(),
            });
            continue;
        }

        // Update threshold
        match DatabaseOperations::update_threshold_config(
            pool_ref,
            &update.config_key,
            update.value,
            &changed_by,
        ) {
            Ok(_) => updated_count += 1,
            Err(e) => {
                failed_updates.push(FailedUpdate {
                    config_key: update.config_key.clone(),
                    error: e.to_string(),
                });
            }
        }
    }

    let success = failed_updates.is_empty();

    // Log batch update to audit
    let audit_details = format!(
        "Batch updated {} thresholds: {}",
        updated_count,
        updates
            .iter()
            .map(|u| &u.config_key)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    );

    let audit_log = wdrprobe_core::models::AuditLog {
        id: 0,
        timestamp: chrono::Utc::now().to_rfc3339(),
        user_id: Some(changed_by),
        action: "BatchThresholdUpdate".to_string(),
        entity_type: "threshold".to_string(),
        entity_id: None,
        old_value: None,
        new_value: Some(updated_count.to_string()),
        ip_address: None,
        success,
        error_message: if success {
            None
        } else {
            Some("Some updates failed".to_string())
        },
        details: Some(audit_details),
    };

    let _ = DatabaseOperations::create_audit_log(pool_ref, &audit_log);

    let failed_count = failed_updates.len();
    Ok(BatchUpdateResult {
        success,
        updated_count,
        failed_updates,
        message: if success {
            Some(format!("Successfully updated {} thresholds", updated_count))
        } else {
            Some(format!(
                "Updated {} thresholds, {} failed",
                updated_count, failed_count
            ))
        },
    })
}

/// Reset a threshold to its default value
#[tauri::command(rename_all = "camelCase")]
pub async fn reset_threshold_to_default(
    pool: State<'_, DatabasePool>,
    config_key: String,
    changed_by: String,
    change_reason: String,
) -> Result<ResetThresholdResult, String> {
    let pool_ref = pool.inner();

    // Get current threshold
    let current = DatabaseOperations::get_threshold_config(pool_ref, &config_key)
        .map_err(|e| format!("Failed to retrieve threshold: {}", e))?
        .ok_or_else(|| format!("Threshold not found: {}", config_key))?;

    let default_value = current.default_value;
    let old_value = current.value;

    // Reset to default
    DatabaseOperations::update_threshold_config(pool_ref, &config_key, default_value, &changed_by)
        .map_err(|e| format!("Failed to reset threshold: {}", e))?;

    // Log to audit
    let audit_log = wdrprobe_core::models::AuditLog {
        id: 0,
        timestamp: chrono::Utc::now().to_rfc3339(),
        user_id: Some(changed_by),
        action: "ThresholdReset".to_string(),
        entity_type: "threshold".to_string(),
        entity_id: Some(current.id),
        old_value: Some(old_value.to_string()),
        new_value: Some(default_value.to_string()),
        ip_address: None,
        success: true,
        error_message: None,
        details: Some(change_reason),
    };

    let _ = DatabaseOperations::create_audit_log(pool_ref, &audit_log);

    Ok(ResetThresholdResult {
        success: true,
        threshold_id: current.id,
        reset_to_value: default_value,
        message: Some("Threshold reset to default".to_string()),
    })
}

/// Get available threshold templates
#[tauri::command(rename_all = "camelCase")]
pub async fn get_threshold_templates() -> Result<ThresholdTemplateList, String> {
    let templates = get_default_templates();
    Ok(ThresholdTemplateList { templates })
}

/// Apply a template to set multiple threshold values
#[tauri::command(rename_all = "camelCase")]
pub async fn apply_threshold_template(
    pool: State<'_, DatabasePool>,
    template_name: String,
    changed_by: String,
    change_reason: String,
) -> Result<ApplyTemplateResult, String> {
    let pool_ref = pool.inner();

    // Validate request
    if changed_by.trim().is_empty() {
        return Err("changed_by is required".to_string());
    }

    if change_reason.trim().is_empty() || change_reason.len() < 10 {
        return Err("change_reason is required and must be at least 10 characters".to_string());
    }

    // Get template
    let templates = get_default_templates();
    let template = templates
        .iter()
        .find(|t| t.name == template_name)
        .ok_or_else(|| format!("Template not found: {}", template_name))?;

    let mut updated_count = 0;

    // Apply each threshold in template
    for threshold_value in &template.thresholds {
        match DatabaseOperations::update_threshold_config(
            pool_ref,
            &threshold_value.config_key,
            threshold_value.value,
            &changed_by,
        ) {
            Ok(_) => updated_count += 1,
            Err(_) => {} // Continue even if one fails
        }
    }

    // Log to audit
    let audit_log = wdrprobe_core::models::AuditLog {
        id: 0,
        timestamp: chrono::Utc::now().to_rfc3339(),
        user_id: Some(changed_by),
        action: "TemplateApplication".to_string(),
        entity_type: "threshold_template".to_string(),
        entity_id: None,
        old_value: None,
        new_value: Some(template_name.clone()),
        ip_address: None,
        success: true,
        error_message: None,
        details: Some(format!(
            "Applied template '{}' affecting {} thresholds",
            template_name, updated_count
        )),
    };

    let _ = DatabaseOperations::create_audit_log(pool_ref, &audit_log);

    Ok(ApplyTemplateResult {
        success: true,
        template_name,
        updated_count,
        message: Some(format!(
            "Template applied successfully - {} thresholds updated",
            updated_count
        )),
    })
}

/// Get change history for a specific threshold
#[tauri::command(rename_all = "camelCase")]
pub async fn get_threshold_history(
    pool: State<'_, DatabasePool>,
    config_key: String,
    limit: Option<i32>,
) -> Result<ThresholdHistory, String> {
    let pool_ref = pool.inner();

    // Get audit logs for this threshold
    let limit = limit.unwrap_or(50).min(100);
    let audit_logs = DatabaseOperations::get_audit_logs(pool_ref, Some(limit), None)
        .map_err(|e| format!("Failed to retrieve audit logs: {}", e))?;

    // Filter and convert to threshold changes
    let history: Vec<ThresholdChange> = audit_logs
        .into_iter()
        .filter(|log| {
            log.entity_type == "threshold"
                && (log.action == "ThresholdUpdate" || log.action == "ThresholdReset")
                && log
                    .details
                    .as_ref()
                    .map(|d| d.contains(&config_key))
                    .unwrap_or(false)
        })
        .map(|log| {
            let old_val = log.old_value.and_then(|v| v.parse().ok());
            let new_val = log.new_value.and_then(|v| v.parse().ok());

            ThresholdChange {
                old_value: old_val.unwrap_or(0.0),
                new_value: new_val.unwrap_or(0.0),
                changed_by: log.user_id.unwrap_or_else(|| "Unknown".to_string()),
                change_reason: log.details.unwrap_or_else(|| String::new()),
                timestamp: log.timestamp,
            }
        })
        .collect();

    Ok(ThresholdHistory {
        config_key,
        history,
    })
}

/// Validate a threshold value without updating
#[tauri::command(rename_all = "camelCase")]
pub async fn validate_threshold_value(
    pool: State<'_, DatabasePool>,
    config_key: String,
    value: f64,
) -> Result<ValidationResult, String> {
    let pool_ref = pool.inner();

    let current = match DatabaseOperations::get_threshold_config(pool_ref, &config_key) {
        Ok(Some(threshold)) => threshold,
        Ok(None) => {
            return Ok(ValidationResult {
                valid: false,
                message: Some(format!("Threshold not found: {}", config_key)),
                suggested_value: None,
            });
        }
        Err(e) => {
            return Err(format!("Failed to retrieve threshold: {}", e));
        }
    };

    // Validate bounds
    match validate_value_within_bounds(&current, value) {
        Ok(_) => {}
        Err(e) => {
            return Ok(ValidationResult {
                valid: false,
                message: Some(e),
                suggested_value: current.default_value.into(),
            });
        }
    }

    // Validate data type
    match validate_data_type(&current, value) {
        Ok(_) => {}
        Err(e) => {
            return Ok(ValidationResult {
                valid: false,
                message: Some(e),
                suggested_value: current.default_value.into(),
            });
        }
    }

    Ok(ValidationResult {
        valid: true,
        message: None,
        suggested_value: None,
    })
}

// Validation functions (DTO format per Constitution IV)

pub fn validate_threshold_update_dto(request: &ThresholdUpdateRequest) -> Result<(), String> {
    if request.changed_by.trim().is_empty() {
        return Err("changed_by is required".to_string());
    }

    if request.change_reason.trim().is_empty() {
        return Err("change_reason is required".to_string());
    }

    if request.change_reason.len() > 500 {
        return Err("change_reason too long (max 500 characters)".to_string());
    }

    if request.change_reason.trim().len() < 10 {
        return Err("change_reason too short (min 10 characters)".to_string());
    }

    Ok(())
}

fn validate_value_within_bounds(config: &ThresholdConfig, value: f64) -> Result<(), String> {
    if let Some(min) = config.min_value {
        if value < min {
            return Err(format!("Value {} below minimum {}", value, min));
        }
    }

    if let Some(max) = config.max_value {
        if value > max {
            return Err(format!("Value {} above maximum {}", value, max));
        }
    }

    if value < 0.0 {
        return Err("Value cannot be negative".to_string());
    }

    Ok(())
}

fn validate_data_type(config: &ThresholdConfig, value: f64) -> Result<(), String> {
    match config.data_type.as_str() {
        "Integer" => {
            if value.fract() != 0.0 {
                return Err("Value must be an integer".to_string());
            }
        }
        "Percentage" => {
            if value > 100.0 {
                return Err("Percentage cannot exceed 100".to_string());
            }
            if value < 0.0 {
                return Err("Percentage cannot be negative".to_string());
            }
        }
        _ => {} // Float allows any value
    }

    Ok(())
}

// Default threshold templates

fn get_default_templates() -> Vec<ThresholdTemplate> {
    vec![
        ThresholdTemplate {
            name: "High Concurrency".to_string(),
            description: "Optimized for high-traffic databases".to_string(),
            category: "All".to_string(),
            thresholds: vec![
                TemplateThreshold {
                    config_key: "sql_execution_time_ms".to_string(),
                    value: 3000.0,
                    description: Some("Higher threshold for busy systems".to_string()),
                },
                TemplateThreshold {
                    config_key: "sql_cpu_time_ms".to_string(),
                    value: 2000.0,
                    description: Some("Allow more CPU time".to_string()),
                },
            ],
        },
        ThresholdTemplate {
            name: "Low Resource".to_string(),
            description: "Optimized for resource-constrained environments".to_string(),
            category: "All".to_string(),
            thresholds: vec![
                TemplateThreshold {
                    config_key: "sql_execution_time_ms".to_string(),
                    value: 500.0,
                    description: Some("Strict threshold for low resources".to_string()),
                },
                TemplateThreshold {
                    config_key: "cpu_usage_percent".to_string(),
                    value: 60.0,
                    description: Some("Lower CPU threshold".to_string()),
                },
            ],
        },
        ThresholdTemplate {
            name: "Development".to_string(),
            description: "Relaxed thresholds for development".to_string(),
            category: "All".to_string(),
            thresholds: vec![
                TemplateThreshold {
                    config_key: "sql_execution_time_ms".to_string(),
                    value: 5000.0,
                    description: Some("Very relaxed for dev".to_string()),
                },
                TemplateThreshold {
                    config_key: "sql_buffer_gets".to_string(),
                    value: 100000.0,
                    description: Some("Allow high buffer usage".to_string()),
                },
            ],
        },
        ThresholdTemplate {
            name: "Production".to_string(),
            description: "Strict thresholds for production systems".to_string(),
            category: "All".to_string(),
            thresholds: vec![
                TemplateThreshold {
                    config_key: "sql_execution_time_ms".to_string(),
                    value: 1000.0,
                    description: Some("Standard production threshold".to_string()),
                },
                TemplateThreshold {
                    config_key: "cpu_usage_percent".to_string(),
                    value: 80.0,
                    description: Some("Standard CPU threshold".to_string()),
                },
            ],
        },
        ThresholdTemplate {
            name: "GaussDB Optimized".to_string(),
            description: "Tuned specifically for GaussDB".to_string(),
            category: "All".to_string(),
            thresholds: vec![
                TemplateThreshold {
                    config_key: "sql_execution_time_ms".to_string(),
                    value: 1500.0,
                    description: Some("Optimized for GaussDB performance".to_string()),
                },
                TemplateThreshold {
                    config_key: "full_table_scan_cost".to_string(),
                    value: 500.0,
                    description: Some("GaussDB-specific scan threshold".to_string()),
                },
            ],
        },
    ]
}
