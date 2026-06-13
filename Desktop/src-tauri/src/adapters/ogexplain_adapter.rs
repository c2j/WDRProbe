//! ogexplain-core → WDRProbe 类型转换适配层
//!
//! 职责：
//! 1. ogexplain_core::model::PlanNode → WDRProbe ExecutionPlanNode
//! 2. ogexplain_core::analyzer::DiagnosticReport → 前端友好格式
//! 3. 热力图/瀑布图数据映射

use ogexplain_core::model::plan::{ExplainPlan, PlanNode};
use ogexplain_core::analyzer::report::{DiagnosticReport, Severity};
use wdrprobe_core::models::execution_plan::{
    ExecutionPlanNode, PlanNodeDetails,
};

/// Convert ogexplain-core PlanNode tree → WDRProbe ExecutionPlanNode tree (recursive)
pub fn convert_plan_node(node: &PlanNode) -> ExecutionPlanNode {
    let cost = node.estimated.as_ref().map(|e| e.total_cost).unwrap_or(0.0);
    let rows = node.estimated.as_ref().map(|e| e.plan_rows as u64).unwrap_or(0);
    let actual_rows = node.actual.as_ref().map(|a| a.rows as u64);
    let actual_time = node.actual.as_ref().map(|a| a.total_time_ms);
    let width = node.estimated.as_ref().map(|e| e.plan_width as u32);

    ExecutionPlanNode {
        operation: format!("{:?}", node.node_type),
        cost,
        rows,
        actual_rows,
        actual_time,
        width,
        children: node.children.iter().map(convert_plan_node).collect(),
        node_details: convert_node_details(node),
        warnings: Vec::new(),
        suggestions: Vec::new(),
    }
}

fn convert_node_details(node: &PlanNode) -> PlanNodeDetails {
    PlanNodeDetails {
        output: None,
        filter: node.properties.iter()
            .find(|p| p.label == "Filter")
            .map(|p| p.value.clone()),
        buffers: None,
        join_type: node.join_type.as_ref().map(|j| format!("{:?}", j)),
        hash_keys: None,
        index_name: node.properties.iter()
            .find(|p| p.label == "Index Name")
            .map(|p| p.value.clone()),
        table_name: node.relation.clone(),
    }
}

/// Convert severity from ogexplain-core to string
fn convert_severity(s: &Severity) -> String {
    match s {
        Severity::Critical => "critical".to_string(),
        Severity::Warning => "warning".to_string(),
        Severity::Info => "info".to_string(),
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct DiagnosticReportResponse {
    pub plan: ExecutionPlanNode,
    pub findings: Vec<FindingInfo>,
    pub stats: DiagnosticStats,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct FindingInfo {
    pub rule_id: String,
    pub severity: String,
    pub category: String,
    pub title: String,
    pub detail: String,
    pub node_line: Option<usize>,
    pub node_type: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct DiagnosticStats {
    pub total_findings: usize,
    pub critical_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct HeatmapData {
    pub nodes: Vec<HeatmapNode>,
    pub summary: HeatmapSummaryData,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct HeatmapNode {
    pub line_number: usize,
    pub node_type: String,
    pub estimated_rows: f64,
    pub actual_rows: f64,
    pub row_qerror: f64,
    pub row_ratio: f64,
    pub on_critical_path: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct HeatmapSummaryData {
    pub max_qerror: f64,
    pub max_qerror_line: usize,
    pub total_nodes: usize,
    pub deviated_count: usize,
    pub severe_count: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct WaterfallData {
    pub nodes: Vec<WaterfallNode>,
    pub bottlenecks: WaterfallBottlenecksData,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct WaterfallNode {
    pub line_number: usize,
    pub node_type: String,
    pub cpu_time_ms: Option<f64>,
    pub peak_memory_kb: Option<f64>,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub is_bottleneck: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct WaterfallBottlenecksData {
    pub cpu_bottlenecks: Vec<usize>,
    pub memory_bottlenecks: Vec<usize>,
    pub total_cpu_time_ms: f64,
    pub max_peak_memory_kb: f64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct RuleInfo {
    pub rule_id: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub severity: String,
}

pub fn convert_diagnostic_report(
    report: &DiagnosticReport,
    plan: &ExplainPlan,
) -> DiagnosticReportResponse {
    let mut critical = 0usize;
    let mut warning = 0usize;
    let mut info = 0usize;

    let findings: Vec<FindingInfo> = report
        .findings
        .iter()
        .map(|f| {
            match f.severity {
                Severity::Critical => critical += 1,
                Severity::Warning => warning += 1,
                Severity::Info => info += 1,
            }
            FindingInfo {
                rule_id: f.rule_id.clone(),
                severity: convert_severity(&f.severity),
                category: format!("{:?}", f.category),
                title: f.title.clone(),
                detail: f.detail.clone(),
                node_line: f.node_line,
                node_type: f.node_type.clone(),
                suggestion: f.suggestion.clone(),
            }
        })
        .collect();

    DiagnosticReportResponse {
        plan: convert_plan_node(&plan.root),
        findings,
        stats: DiagnosticStats {
            total_findings: report.findings.len(),
            critical_count: critical,
            warning_count: warning,
            info_count: info,
        },
    }
}

/// Generate heatmap data from ogexplain-core
pub fn convert_heatmap(
    hm: &ogexplain_core::analyzer::heatmap::PlanHeatmap,
) -> HeatmapData {
    let nodes: Vec<HeatmapNode> = hm
        .entries
        .iter()
        .map(|e| HeatmapNode {
            line_number: e.deviation.line_number,
            node_type: e.deviation.node_type.clone(),
            estimated_rows: e.deviation.estimated_rows,
            actual_rows: e.deviation.actual_rows,
            row_qerror: e.deviation.row_qerror,
            row_ratio: e.deviation.row_ratio,
            on_critical_path: e.on_critical_path,
        })
        .collect();

    HeatmapData {
        nodes,
        summary: HeatmapSummaryData {
            max_qerror: hm.summary.max_qerror,
            max_qerror_line: hm.summary.max_qerror_line,
            total_nodes: hm.summary.total_nodes,
            deviated_count: hm.summary.deviated_count,
            severe_count: hm.summary.severe_count,
        },
    }
}

/// Generate waterfall data from ogexplain-core
pub fn convert_waterfall(
    wf: &ogexplain_core::analyzer::waterfall::PlanWaterfall,
) -> WaterfallData {
    let nodes: Vec<WaterfallNode> = wf
        .entries
        .iter()
        .map(|e| WaterfallNode {
            line_number: e.metrics.line_number,
            node_type: e.metrics.node_type.clone(),
            cpu_time_ms: e.metrics.cpu_time_ms,
            peak_memory_kb: e.metrics.peak_memory_kb,
            cpu_percent: e.cpu_percent,
            memory_percent: e.memory_percent,
            is_bottleneck: e.is_bottleneck,
        })
        .collect();

    WaterfallData {
        nodes,
        bottlenecks: WaterfallBottlenecksData {
            cpu_bottlenecks: wf.bottlenecks.cpu_bottlenecks.clone(),
            memory_bottlenecks: wf.bottlenecks.memory_bottlenecks.clone(),
            total_cpu_time_ms: wf.bottlenecks.total_cpu_time_ms,
            max_peak_memory_kb: wf.bottlenecks.max_peak_memory_kb,
        },
    }
}
