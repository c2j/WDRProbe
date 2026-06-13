// Integration test for ogexplain-core adapter
use wdrprobe_desktop_lib::adapters::ogexplain_adapter;

#[test]
fn test_parse_simple_explain() {
    let explain_text = "QUERY PLAN\n----------------------------------------------------\nSeq Scan on t_order  (cost=0.00..1500.00 rows=100000 width=100)";
    let result = ogexplain_core::parse(explain_text);
    assert!(result.is_ok(), "Should parse simple EXPLAIN: {:?}", result.err());
    let plan = result.unwrap();
    let node = ogexplain_adapter::convert_plan_node(&plan.root);
    assert!(node.operation.contains("SeqScan"), "Should detect Seq Scan, got: {}", node.operation);
    assert!(node.cost > 0.0, "Should have cost");
}

#[test]
fn test_diagnose_explain_plan() {
    let explain_text = "QUERY PLAN\n----------------------------------------------------\nSeq Scan on large_table  (cost=0.00..50000.00 rows=500000 width=200)";
    let plan = ogexplain_core::parse(explain_text).expect("Should parse");
    let report = ogexplain_core::analyze(&plan);
    let response = ogexplain_adapter::convert_diagnostic_report(&report, &plan);
    assert_eq!(response.stats.total_findings, report.findings.len());
    assert_eq!(response.plan.operation, format!("{:?}", plan.root.node_type));
}

#[test]
fn test_heatmap_no_analyze() {
    let explain_text = "QUERY PLAN\n----------------------------------------------------\nSeq Scan on t  (cost=0.00..10.00 rows=100 width=10)";
    let plan = ogexplain_core::parse(explain_text).expect("Should parse");
    let hm = ogexplain_core::heatmap(&plan);
    assert!(hm.is_none(), "Heatmap should be None without ANALYZE data");
}
