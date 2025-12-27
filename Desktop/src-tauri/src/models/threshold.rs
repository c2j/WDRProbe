// Threshold configuration data models
// Per Constitution Principle IV - DTO format with audit trail

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThresholdConfig {
    pub id: i64,
    pub category: String,
    pub data_type: String,
    pub config_key: String,
    pub value: f64,
    pub default_value: f64,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub description: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThresholdUpdateRequest {
    pub config_key: String,
    pub value: f64,
    pub changed_by: String,
    pub change_reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThresholdUpdate {
    pub config_key: String,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchUpdateRequest {
    pub updates: Vec<ThresholdUpdate>,
    pub changed_by: String,
    pub change_reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThresholdTemplate {
    pub name: String,
    pub description: String,
    pub category: String,
    pub thresholds: Vec<TemplateThreshold>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateThreshold {
    pub config_key: String,
    pub value: f64,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThresholdConfigList {
    pub thresholds: Vec<ThresholdConfig>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateThresholdResult {
    pub success: bool,
    pub threshold_id: i64,
    pub old_value: f64,
    pub new_value: f64,
    pub updated_at: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchUpdateResult {
    pub success: bool,
    pub updated_count: usize,
    pub failed_updates: Vec<FailedUpdate>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FailedUpdate {
    pub config_key: String,
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResetThresholdResult {
    pub success: bool,
    pub threshold_id: i64,
    pub reset_to_value: f64,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApplyTemplateResult {
    pub success: bool,
    pub template_name: String,
    pub updated_count: usize,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub message: Option<String>,
    pub suggested_value: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThresholdHistory {
    pub config_key: String,
    pub history: Vec<ThresholdChange>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThresholdChange {
    pub old_value: f64,
    pub new_value: f64,
    pub changed_by: String,
    pub change_reason: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThresholdTemplateList {
    pub templates: Vec<ThresholdTemplate>,
}
