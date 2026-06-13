// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adapters;
mod commands;

use commands::audit;
use commands::comparison;
use commands::dashboard;
use commands::execution_plan;
use commands::export;
use commands::reports;
use commands::threshold;
use wdrprobe_core::database::{init_database, initialize_schema};
use wdrprobe_core::database::schema::{initialize_default_thresholds, initialize_sample_audit_issues};
use tauri::Manager;

#[cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

pub type Result<T> = std::result::Result<T, anyhow::Error>;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path_resolver()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

            let db_path = app_data_dir.join("wdrprobe.db");

            // Initialize database
            let pool =
                init_database(db_path.to_str().unwrap()).expect("Failed to initialize database");

            // Initialize schema
            let conn = pool.get().expect("Failed to get database connection");
            initialize_schema(&conn).expect("Failed to initialize schema");

            // Initialize default thresholds
            initialize_default_thresholds(&conn).expect("Failed to initialize default thresholds");

            // Initialize sample audit issues
            initialize_sample_audit_issues(&conn).expect("Failed to initialize sample audit issues");

            // Store database pool in app state
            app.manage(pool);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Dashboard commands
            dashboard::get_instance_summaries,
            dashboard::get_dashboard_metrics,
            // Reports commands
            reports::import_wdr_report,
            reports::get_wdr_reports,
            reports::get_wdr_report_detail,
            reports::delete_wdr_report,
            reports::get_hot_sqls,
            // Execution plan commands
            execution_plan::get_wdr_hot_sqls,
            execution_plan::get_execution_plan,
            execution_plan::parse_execution_plan,
            execution_plan::analyze_execution_plan_command,
            execution_plan::save_execution_plan,
            execution_plan::get_saved_plans,
            execution_plan::delete_execution_plan,
            execution_plan::generate_optimization_sql,
            execution_plan::parse_explain_with_ogexplain,
            execution_plan::diagnose_explain_plan,
            execution_plan::get_explain_heatmap,
            execution_plan::get_explain_waterfall,
            execution_plan::list_diagnostic_rules,
            // Comparison commands
            comparison::get_comparisons,
            comparison::get_comparison_summary,
            comparison::get_comparison_details,
            comparison::create_comparison,
            comparison::delete_comparison,
            comparison::get_comparison_chart_data,
            // Threshold commands
            threshold::get_threshold_configs,
            threshold::get_threshold_config,
            threshold::update_threshold,
            threshold::batch_update_thresholds,
            threshold::reset_threshold_to_default,
            threshold::get_threshold_templates,
            threshold::apply_threshold_template,
            threshold::get_threshold_history,
            threshold::validate_threshold_value,
            // Audit commands
            audit::run_sql_audit,
            audit::get_sql_audit_issues,
            audit::update_audit_issue_status,
            audit::bulk_update_audit_issues,
            // Export/Import commands
            export::export_wdr_report,
            export::export_comparison,
            export::import_data,
            export::validate_data_integrity,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
