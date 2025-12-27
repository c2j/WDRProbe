// Execution plan data models
// Contains models for SQL execution plans

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionPlanNode {
    pub operation: String,
    pub cost: f64,
    pub rows: u64,
    pub actual_rows: Option<u64>,
    pub actual_time: Option<f64>,
    pub width: Option<u32>,
    pub children: Vec<ExecutionPlanNode>,
    pub node_details: PlanNodeDetails,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanNodeDetails {
    pub output: Option<Vec<String>>,
    pub filter: Option<String>,
    pub buffers: Option<String>,
    pub join_type: Option<String>,
    pub hash_keys: Option<Vec<String>>,
    pub index_name: Option<String>,
    pub table_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqlExecutionPlan {
    pub id: i64,
    pub sql_id: Option<i64>,
    pub plan_tree: ExecutionPlanNode,
    pub created_at: String,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionPlanResponse {
    pub success: bool,
    pub plan_tree: ExecutionPlanNode,
    pub plan_metadata: PlanMetadata,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanMetadata {
    pub total_cost: f64,
    pub total_rows: u64,
    pub plan_depth: u32,
    pub node_count: u32,
    pub optimization_warnings: u32,
    pub estimated_time_ms: f64,
    pub gaussdb_format: bool,
    pub has_actual_stats: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WdrHotSql {
    pub id: i64,
    pub report_id: i64,
    pub sql_id: Option<String>,
    pub sql_text: String,
    pub executions: u64,
    pub total_elapsed_time: f64,
    pub cpu_time: f64,
    pub rank: i32,
    pub instance_name: String,
    pub generation_time: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WdrHotSqlList {
    pub hot_sqls: Vec<WdrHotSql>,
    pub total: i64,
}
