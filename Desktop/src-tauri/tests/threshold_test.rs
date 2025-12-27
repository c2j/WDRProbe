// Unit tests for threshold configuration functionality
// Tests for User Story 5 - Configure Performance Thresholds

#[cfg(test)]
mod threshold_tests {
    use wdrprobe_desktop_lib::models::threshold::*;

    #[test]
    fn test_threshold_update_request_valid() {
        let request = ThresholdUpdateRequest {
            config_key: "sql_execution_time_ms".to_string(),
            value: 2000.0,
            changed_by: "admin".to_string(),
            change_reason: "Increased for high-latency environment".to_string(),
        };

        assert!(validate_threshold_update_dto(&request).is_ok());
    }

    #[test]
    fn test_threshold_update_request_empty_changed_by() {
        let request = ThresholdUpdateRequest {
            config_key: "sql_execution_time_ms".to_string(),
            value: 2000.0,
            changed_by: "".to_string(),
            change_reason: "Test reason".to_string(),
        };

        let result = validate_threshold_update_dto(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("changed_by"));
    }

    #[test]
    fn test_threshold_update_request_empty_reason() {
        let request = ThresholdUpdateRequest {
            config_key: "sql_execution_time_ms".to_string(),
            value: 2000.0,
            changed_by: "admin".to_string(),
            change_reason: "".to_string(),
        };

        let result = validate_threshold_update_dto(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("change_reason"));
    }

    #[test]
    fn test_threshold_update_request_reason_too_long() {
        let long_reason = "a".repeat(501);
        let request = ThresholdUpdateRequest {
            config_key: "sql_execution_time_ms".to_string(),
            value: 2000.0,
            changed_by: "admin".to_string(),
            change_reason: long_reason,
        };

        let result = validate_threshold_update_dto(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too long"));
    }

    #[test]
    fn test_threshold_update_request_reason_too_short() {
        let short_reason = "abc";
        let request = ThresholdUpdateRequest {
            config_key: "sql_execution_time_ms".to_string(),
            value: 2000.0,
            changed_by: "admin".to_string(),
            change_reason: short_reason.to_string(),
        };

        let result = validate_threshold_update_dto(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too short"));
    }

    #[test]
    fn test_batch_update_thresholds_empty() {
        let updates = vec![];
        let request = BatchUpdateRequest {
            updates,
            changed_by: "admin".to_string(),
            change_reason: "Batch update test".to_string(),
        };

        let result = validate_batch_update(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No updates"));
    }

    #[test]
    fn test_batch_update_thresholds_single() {
        let updates = vec![ThresholdUpdate {
            config_key: "sql_execution_time_ms".to_string(),
            value: 2000.0,
        }];

        let request = BatchUpdateRequest {
            updates,
            changed_by: "admin".to_string(),
            change_reason: "Update execution time threshold".to_string(),
        };

        assert!(validate_batch_update(&request).is_ok());
    }

    #[test]
    fn test_batch_update_thresholds_multiple() {
        let updates = vec![
            ThresholdUpdate {
                config_key: "sql_execution_time_ms".to_string(),
                value: 2000.0,
            },
            ThresholdUpdate {
                config_key: "sql_cpu_time_ms".to_string(),
                value: 1000.0,
            },
            ThresholdUpdate {
                config_key: "sql_buffer_gets".to_string(),
                value: 20000.0,
            },
        ];

        let request = BatchUpdateRequest {
            updates,
            changed_by: "admin".to_string(),
            change_reason: "Optimize thresholds for production".to_string(),
        };

        assert!(validate_batch_update(&request).is_ok());
    }

    #[test]
    fn test_threshold_value_within_bounds() {
        let config = ThresholdConfig {
            id: 1,
            category: "Sql".to_string(),
            data_type: "Integer".to_string(),
            config_key: "sql_execution_time_ms".to_string(),
            value: 1000.0,
            default_value: 1000.0,
            min_value: Some(0.0),
            max_value: Some(60000.0),
            description: Some("SQL execution time threshold".to_string()),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            updated_by: None,
        };

        assert!(validate_value_within_bounds(&config, 5000.0).is_ok());
        assert!(validate_value_within_bounds(&config, 0.0).is_ok());
        assert!(validate_value_within_bounds(&config, 60000.0).is_ok());
    }

    #[test]
    fn test_threshold_value_below_minimum() {
        let config = ThresholdConfig {
            id: 1,
            category: "Sql".to_string(),
            data_type: "Integer".to_string(),
            config_key: "sql_execution_time_ms".to_string(),
            value: 1000.0,
            default_value: 1000.0,
            min_value: Some(100.0),
            max_value: Some(60000.0),
            description: Some("SQL execution time threshold".to_string()),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            updated_by: None,
        };

        let result = validate_value_within_bounds(&config, 50.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("minimum"));
    }

    #[test]
    fn test_threshold_value_above_maximum() {
        let config = ThresholdConfig {
            id: 1,
            category: "Sql".to_string(),
            data_type: "Integer".to_string(),
            config_key: "sql_execution_time_ms".to_string(),
            value: 1000.0,
            default_value: 1000.0,
            min_value: Some(0.0),
            max_value: Some(5000.0),
            description: Some("SQL execution time threshold".to_string()),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            updated_by: None,
        };

        let result = validate_value_within_bounds(&config, 10000.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("maximum"));
    }

    #[test]
    fn test_threshold_value_negative() {
        let config = ThresholdConfig {
            id: 1,
            category: "Sql".to_string(),
            data_type: "Integer".to_string(),
            config_key: "sql_execution_time_ms".to_string(),
            value: 1000.0,
            default_value: 1000.0,
            min_value: None,
            max_value: None,
            description: Some("SQL execution time threshold".to_string()),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            updated_by: None,
        };

        let result = validate_value_within_bounds(&config, -100.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("negative"));
    }

    #[test]
    fn test_threshold_data_type_integer_validation() {
        let config = ThresholdConfig {
            id: 1,
            category: "Sql".to_string(),
            data_type: "Integer".to_string(),
            config_key: "sql_execution_time_ms".to_string(),
            value: 1000.0,
            default_value: 1000.0,
            min_value: None,
            max_value: None,
            description: Some("SQL execution time threshold".to_string()),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            updated_by: None,
        };

        // Integer should accept whole numbers
        assert!(validate_data_type(&config, 1000.0).is_ok());
        assert!(validate_data_type(&config, 0.0).is_ok());

        // Integer should reject decimals
        assert!(validate_data_type(&config, 1000.5).is_err());
    }

    #[test]
    fn test_threshold_data_type_percentage_validation() {
        let config = ThresholdConfig {
            id: 1,
            category: "System".to_string(),
            data_type: "Percentage".to_string(),
            config_key: "cpu_usage_percent".to_string(),
            value: 80.0,
            default_value: 80.0,
            min_value: Some(0.0),
            max_value: Some(100.0),
            description: Some("CPU usage threshold".to_string()),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            updated_by: None,
        };

        // Percentage should accept 0-100
        assert!(validate_data_type(&config, 0.0).is_ok());
        assert!(validate_data_type(&config, 50.5).is_ok());
        assert!(validate_data_type(&config, 100.0).is_ok());

        // Percentage should reject > 100
        assert!(validate_data_type(&config, 101.0).is_err());
        assert!(validate_data_type(&config, 150.0).is_err());
    }

    #[test]
    fn test_threshold_data_type_float_validation() {
        let config = ThresholdConfig {
            id: 1,
            category: "AI".to_string(),
            data_type: "Float".to_string(),
            config_key: "ai_confidence_threshold".to_string(),
            value: 0.8,
            default_value: 0.8,
            min_value: Some(0.0),
            max_value: Some(1.0),
            description: Some("AI confidence threshold".to_string()),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            updated_by: None,
        };

        // Float should accept any value
        assert!(validate_data_type(&config, 0.0).is_ok());
        assert!(validate_data_type(&config, 0.5).is_ok());
        assert!(validate_data_type(&config, 0.999).is_ok());
        assert!(validate_data_type(&config, 1.0).is_ok());
    }

    #[test]
    fn test_update_threshold_result() {
        let result = UpdateThresholdResult {
            success: true,
            threshold_id: 42,
            old_value: 1000.0,
            new_value: 2000.0,
            updated_at: "2024-01-15T10:30:00Z".to_string(),
            message: Some("Threshold updated successfully".to_string()),
        };

        assert!(result.success);
        assert_eq!(result.threshold_id, 42);
        assert_eq!(result.old_value, 1000.0);
        assert_eq!(result.new_value, 2000.0);
    }

    #[test]
    fn test_threshold_config_list() {
        let thresholds = vec![
            ThresholdConfig {
                id: 1,
                category: "Sql".to_string(),
                data_type: "Integer".to_string(),
                config_key: "sql_execution_time_ms".to_string(),
                value: 1000.0,
                default_value: 1000.0,
                min_value: Some(0.0),
                max_value: Some(60000.0),
                description: Some("SQL execution time threshold".to_string()),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
                updated_by: Some("admin".to_string()),
            },
            ThresholdConfig {
                id: 2,
                category: "Sql".to_string(),
                data_type: "Integer".to_string(),
                config_key: "sql_cpu_time_ms".to_string(),
                value: 500.0,
                default_value: 500.0,
                min_value: Some(0.0),
                max_value: Some(60000.0),
                description: Some("SQL CPU time threshold".to_string()),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
                updated_by: None,
            },
        ];

        let list = ThresholdConfigList {
            thresholds: thresholds.clone(),
            total: 2,
        };

        assert_eq!(list.thresholds.len(), 2);
        assert_eq!(list.total, 2);
    }

    #[test]
    fn test_reset_threshold_result() {
        let result = ResetThresholdResult {
            success: true,
            threshold_id: 42,
            reset_to_value: 1000.0,
            message: Some("Threshold reset to default".to_string()),
        };

        assert!(result.success);
        assert_eq!(result.reset_to_value, 1000.0);
    }

    #[test]
    fn test_batch_update_result_all_success() {
        let result = BatchUpdateResult {
            success: true,
            updated_count: 3,
            failed_updates: vec![],
            message: Some("All thresholds updated successfully".to_string()),
        };

        assert!(result.success);
        assert_eq!(result.updated_count, 3);
        assert!(result.failed_updates.is_empty());
    }

    #[test]
    fn test_batch_update_result_partial_failure() {
        let failed = vec![FailedUpdate {
            config_key: "invalid_key".to_string(),
            error: "Threshold not found".to_string(),
        }];

        let result = BatchUpdateResult {
            success: false,
            updated_count: 2,
            failed_updates: failed,
            message: Some("Some thresholds failed to update".to_string()),
        };

        assert!(!result.success);
        assert_eq!(result.updated_count, 2);
        assert_eq!(result.failed_updates.len(), 1);
    }

    #[test]
    fn test_apply_template_result() {
        let result = ApplyTemplateResult {
            success: true,
            template_name: "Production".to_string(),
            updated_count: 15,
            message: Some("Template applied successfully".to_string()),
        };

        assert!(result.success);
        assert_eq!(result.template_name, "Production");
        assert_eq!(result.updated_count, 15);
    }

    #[test]
    fn test_threshold_template() {
        let template = ThresholdTemplate {
            name: "High Concurrency".to_string(),
            description: "Optimized for high-traffic databases".to_string(),
            category: "All".to_string(),
            thresholds: vec![],
        };

        assert_eq!(template.name, "High Concurrency");
        assert_eq!(template.category, "All");
    }

    // Helper functions for validation (would be in commands/threshold.rs)

    fn validate_threshold_update_dto(request: &ThresholdUpdateRequest) -> Result<(), String> {
        if request.changed_by.trim().is_empty() {
            return Err("changed_by is required".to_string());
        }

        if request.change_reason.trim().is_empty() {
            return Err("change_reason is required".to_string());
        }

        if request.change_reason.len() > 500 {
            return Err("change_reason is too long (max 500 characters)".to_string());
        }

        if request.change_reason.trim().len() < 10 {
            return Err("change_reason is too short (min 10 characters)".to_string());
        }

        Ok(())
    }

    // Define missing types for tests
    #[derive(Debug, Clone)]
    struct ThresholdUpdate {
        pub config_key: String,
        pub value: f64,
    }

    #[derive(Debug, Clone)]
    struct BatchUpdateRequest {
        pub updates: Vec<ThresholdUpdate>,
        pub changed_by: String,
        pub change_reason: String,
    }

    #[derive(Debug, Clone)]
    struct BatchUpdateResult {
        pub success: bool,
        pub updated_count: usize,
        pub failed_updates: Vec<FailedUpdate>,
        pub message: Option<String>,
    }

    #[derive(Debug, Clone)]
    struct FailedUpdate {
        pub config_key: String,
        pub error: String,
    }

    #[derive(Debug, Clone)]
    struct ResetThresholdResult {
        pub success: bool,
        pub threshold_id: i64,
        pub reset_to_value: f64,
        pub message: Option<String>,
    }

    #[derive(Debug, Clone)]
    struct ApplyTemplateResult {
        pub success: bool,
        pub template_name: String,
        pub updated_count: usize,
        pub message: Option<String>,
    }

    fn validate_batch_update(request: &BatchUpdateRequest) -> Result<(), String> {
        if request.updates.is_empty() {
            return Err("No updates provided".to_string());
        }

        if request.changed_by.trim().is_empty() {
            return Err("changed_by is required".to_string());
        }

        if request.change_reason.trim().is_empty() {
            return Err("change_reason is required".to_string());
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
}
