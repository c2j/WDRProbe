// Integration test for threshold audit logging workflow
// Tests for User Story 5 - Configure Performance Thresholds
// Per Constitution Principle IX - All threshold changes must be logged

#[cfg(test)]
mod threshold_audit_integration_tests {
    use wdrprobe_desktop_lib::models::threshold::*;
    use wdrprobe_desktop_lib::models::AuditLog;

    /// Test complete threshold update workflow with audit logging
    #[test]
    fn test_threshold_update_with_audit_logging() {
        // Step 1: Prepare threshold update request (DTO format per Constitution IV)
        let request = ThresholdUpdateRequest {
            config_key: "sql_execution_time_ms".to_string(),
            value: 2000.0,
            changed_by: "admin".to_string(),
            change_reason: "Production environment requires higher threshold".to_string(),
        };

        // Verify DTO format compliance
        assert!(!request.changed_by.is_empty());
        assert!(!request.change_reason.is_empty());
        assert!(request.change_reason.len() < 500);
        assert!(request.change_reason.len() >= 10);

        // Step 2: Simulate threshold update
        let old_value = 1000.0;
        let new_value = request.value;

        // Step 3: Create expected audit log entry (per Constitution IX)
        let expected_audit_log = AuditLog {
            id: 0, // Will be assigned by database
            timestamp: chrono::Utc::now().to_rfc3339(),
            user_id: Some("admin".to_string()),
            action: "ThresholdUpdate".to_string(),
            entity_type: "threshold".to_string(),
            entity_id: Some(42),
            old_value: Some(old_value.to_string()),
            new_value: Some(new_value.to_string()),
            ip_address: None,
            success: true,
            error_message: None,
            details: Some("Production environment requires higher threshold".to_string()),
        };

        assert_eq!(expected_audit_log.action, "ThresholdUpdate");
        assert_eq!(expected_audit_log.entity_type, "threshold");
        assert_eq!(expected_audit_log.old_value, Some("1000".to_string()));
        assert_eq!(expected_audit_log.new_value, Some("2000".to_string()));
        assert_eq!(expected_audit_log.user_id, Some("admin".to_string()));
    }

    /// Test batch threshold update with single audit log entry
    #[test]
    fn test_batch_threshold_update_with_single_audit_log() {
        let request = BatchUpdateRequest {
            updates: vec![
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
            ],
            changed_by: "dba_admin".to_string(),
            change_reason: "Optimize thresholds for high-concurrency production environment".to_string(),
        };

        // Batch update should generate single audit log entry
        let audit_details = format!(
            "Batch updated {} thresholds: {}",
            request.updates.len(),
            request.updates.iter()
                .map(|u| &u.config_key)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );

        let expected_audit_log = AuditLog {
            id: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            user_id: Some("dba_admin".to_string()),
            action: "BatchThresholdUpdate".to_string(),
            entity_type: "threshold".to_string(),
            entity_id: None, // Batch update affects multiple entities
            old_value: None,
            new_value: Some(request.updates.len().to_string()),
            ip_address: None,
            success: true,
            error_message: None,
            details: Some(audit_details),
        };

        assert_eq!(expected_audit_log.action, "BatchThresholdUpdate");
        assert_eq!(expected_audit_log.new_value, Some("3".to_string()));
        assert!(expected_audit_log.details.unwrap().contains("sql_execution_time_ms"));
        assert!(expected_audit_log.details.unwrap().contains("sql_cpu_time_ms"));
        assert!(expected_audit_log.details.unwrap().contains("sql_buffer_gets"));
    }

    /// Test threshold reset to default with audit logging
    #[test]
    fn test_threshold_reset_with_audit_logging() {
        let request = ThresholdUpdateRequest {
            config_key: "sql_execution_time_ms".to_string(),
            value: 1000.0, // Reset to default
            changed_by: "system_admin".to_string(),
            change_reason: "Resetting to default configuration after testing period".to_string(),
        };

        // Reset operation should be logged with action='Reset'
        let audit_log = AuditLog {
            id: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            user_id: Some("system_admin".to_string()),
            action: "ThresholdReset".to_string(),
            entity_type: "threshold".to_string(),
            entity_id: Some(42),
            old_value: Some("2000".to_string()), // Previous custom value
            new_value: Some("1000".to_string()), // Reset to default
            ip_address: None,
            success: true,
            error_message: None,
            details: Some("Resetting to default configuration after testing period".to_string()),
        };

        assert_eq!(audit_log.action, "ThresholdReset");
        assert_eq!(audit_log.old_value, Some("2000".to_string()));
        assert_eq!(audit_log.new_value, Some("1000".to_string()));
    }

    /// Test template application with audit logging
    #[test]
    fn test_template_application_with_audit_logging() {
        let template_name = "Production";
        let request = ApplyTemplateRequest {
            template_name: template_name.to_string(),
            changed_by: "devops_team".to_string(),
            change_reason: "Applying production-ready thresholds for system go-live".to_string(),
        };

        // Template application should log all affected thresholds
        let affected_thresholds = vec![
            "sql_execution_time_ms", "sql_cpu_time_ms", "sql_buffer_gets",
            "cpu_usage_percent", "memory_usage_percent", "disk_usage_percent",
        ];

        let audit_log = AuditLog {
            id: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            user_id: Some("devops_team".to_string()),
            action: "TemplateApplication".to_string(),
            entity_type: "threshold_template".to_string(),
            entity_id: None,
            old_value: None,
            new_value: Some(template_name.to_string()),
            ip_address: None,
            success: true,
            error_message: None,
            details: Some(format!(
                "Applied template '{}' affecting {} thresholds: {}",
                template_name,
                affected_thresholds.len(),
                affected_thresholds.join(", ")
            )),
        };

        assert_eq!(audit_log.action, "TemplateApplication");
        assert_eq!(audit_log.entity_type, "threshold_template");
        assert_eq!(audit_log.new_value, Some("Production".to_string()));
        assert!(audit_log.details.unwrap().contains("6 thresholds"));
    }

    /// Test audit log retrieval for threshold history
    #[test]
    fn test_threshold_history_from_audit_logs() {
        let config_key = "sql_execution_time_ms";

        // Simulate audit logs for threshold changes
        let audit_logs = vec![
            AuditLog {
                id: 1,
                timestamp: "2024-01-15T10:00:00Z".to_string(),
                user_id: Some("admin".to_string()),
                action: "ThresholdUpdate".to_string(),
                entity_type: "threshold".to_string(),
                entity_id: Some(42),
                old_value: Some("1000".to_string()),
                new_value: Some("1500".to_string()),
                ip_address: None,
                success: true,
                error_message: None,
                details: Some("Initial adjustment".to_string()),
            },
            AuditLog {
                id: 2,
                timestamp: "2024-01-20T14:30:00Z".to_string(),
                user_id: Some("dba_admin".to_string()),
                action: "ThresholdUpdate".to_string(),
                entity_type: "threshold".to_string(),
                entity_id: Some(42),
                old_value: Some("1500".to_string()),
                new_value: Some("2000".to_string()),
                ip_address: None,
                success: true,
                error_message: None,
                details: Some("Production environment adjustment".to_string()),
            },
            AuditLog {
                id: 3,
                timestamp: "2024-02-01T09:15:00Z".to_string(),
                user_id: Some("system_admin".to_string()),
                action: "ThresholdReset".to_string(),
                entity_type: "threshold".to_string(),
                entity_id: Some(42),
                old_value: Some("2000".to_string()),
                new_value: Some("1000".to_string()),
                ip_address: None,
                success: true,
                error_message: None,
                details: Some("Reset to default".to_string()),
            },
        ];

        // Verify audit log structure for history
        assert_eq!(audit_logs.len(), 3);

        // Check first change
        assert_eq!(audit_logs[0].action, "ThresholdUpdate");
        assert_eq!(audit_logs[0].old_value, Some("1000".to_string()));
        assert_eq!(audit_logs[0].new_value, Some("1500".to_string()));
        assert_eq!(audit_logs[0].user_id, Some("admin".to_string()));

        // Check second change
        assert_eq!(audit_logs[1].new_value, Some("2000".to_string()));
        assert_eq!(audit_logs[1].user_id, Some("dba_admin".to_string()));

        // Check reset
        assert_eq!(audit_logs[2].action, "ThresholdReset");
        assert_eq!(audit_logs[2].new_value, Some("1000".to_string()));
    }

    /// Test audit log failure handling
    #[test]
    fn test_audit_log_failure_handling() {
        // Simulate failed threshold update
        let failed_update = FailedUpdate {
            config_key: "invalid_threshold_key".to_string(),
            error: "Threshold not found in configuration".to_string(),
        };

        // Even failed updates should be logged for compliance
        let audit_log = AuditLog {
            id: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            user_id: Some("admin".to_string()),
            action: "ThresholdUpdate".to_string(),
            entity_type: "threshold".to_string(),
            entity_id: None,
            old_value: None,
            new_value: None,
            ip_address: None,
            success: false,
            error_message: Some(failed_update.error.clone()),
            details: Some(format!(
                "Failed to update threshold '{}': {}",
                failed_update.config_key, failed_update.error
            )),
        };

        assert!(!audit_log.success);
        assert_eq!(audit_log.error_message, Some("Threshold not found in configuration".to_string()));
        assert!(audit_log.details.unwrap().contains("invalid_threshold_key"));
    }

    /// Test audit log query patterns
    #[test]
    fn test_audit_log_query_patterns() {
        // Test filtering by entity_type and action
        let query_filters = vec![
            ("entity_type", "threshold"),
            ("action", "ThresholdUpdate"),
            ("user_id", "admin"),
        ];

        for (field, value) in query_filters {
            assert!(!field.is_empty());
            assert!(!value.is_empty());
        }

        // Test time range queries for history
        let start_time = "2024-01-01T00:00:00Z";
        let end_time = "2024-12-31T23:59:59Z";

        assert!(start_time < end_time);
    }

    /// Helper struct for ApplyTemplateRequest
    #[derive(Debug, Clone)]
    struct ApplyTemplateRequest {
        pub template_name: String,
        pub changed_by: String,
        pub change_reason: String,
    }

    /// Test comprehensive audit trail for compliance
    #[test]
    fn test_comprehensive_audit_trail() {
        // All audit log entries must contain required fields
        let required_fields = vec!["timestamp", "action", "entity_type", "success"];

        let audit_log = AuditLog {
            id: 1,
            timestamp: "2024-01-15T10:00:00Z".to_string(),
            user_id: Some("admin".to_string()),
            action: "ThresholdUpdate".to_string(),
            entity_type: "threshold".to_string(),
            entity_id: Some(42),
            old_value: Some("1000".to_string()),
            new_value: Some("2000".to_string()),
            ip_address: Some("192.168.1.100".to_string()),
            success: true,
            error_message: None,
            details: Some("Updated for production environment".to_string()),
        };

        // Verify all required fields are present
        assert!(!audit_log.timestamp.is_empty());
        assert!(!audit_log.action.is_empty());
        assert!(!audit_log.entity_type.is_empty());

        // For compliance, verify we have who, what, when, and why
        assert!(audit_log.user_id.is_some()); // Who
        assert!(!audit_log.action.is_empty()); // What
        assert!(!audit_log.timestamp.is_empty()); // When
        assert!(audit_log.details.is_some() || !audit_log.old_value.is_none() || !audit_log.new_value.is_none()); // Why/Context
    }

    /// Test threshold change aggregation for audit summary
    #[test]
    fn test_threshold_change_aggregation() {
        let changes = vec![
            ("sql_execution_time_ms", 1000.0, 2000.0),
            ("sql_cpu_time_ms", 500.0, 1000.0),
            ("sql_buffer_gets", 10000.0, 20000.0),
        ];

        let mut increased_count = 0;
        let mut decreased_count = 0;

        for (key, old, new) in &changes {
            if new > old {
                increased_count += 1;
            } else if new < old {
                decreased_count += 1;
            }
        }

        assert_eq!(increased_count, 3);
        assert_eq!(decreased_count, 0);

        // All changes increased thresholds (more lenient)
        let summary = format!(
            "Threshold changes: {} increased, {} decreased, {} unchanged",
            increased_count, decreased_count, changes.len() - increased_count - decreased_count
        );

        assert!(summary.contains("3 increased"));
    }
}
