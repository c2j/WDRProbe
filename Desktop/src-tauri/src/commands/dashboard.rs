// Dashboard commands
// IPC commands for dashboard data retrieval

use wdrprobe_core::database::{DatabaseOperations, DatabasePool};
use wdrprobe_core::models::{DashboardMetrics, InstanceSummary};

/// Get instance summaries for dashboard
#[tauri::command]
pub async fn get_instance_summaries(
    pool: tauri::State<'_, DatabasePool>,
) -> Result<Vec<InstanceSummary>, String> {
    pool.get_instance_summaries().map_err(|e| e.to_string())
}

/// Get dashboard metrics and trend data
#[tauri::command]
pub async fn get_dashboard_metrics(
    pool: tauri::State<'_, DatabasePool>,
    instance_name: Option<String>,
) -> Result<DashboardMetrics, String> {
    let instance_name_ref = instance_name.as_deref();
    pool.get_dashboard_metrics(instance_name_ref)
        .map_err(|e| e.to_string())
}
