use std::fs;
use crate::output;

/// Run the diagnose command: analyze an EXPLAIN plan
pub fn run(
    plan_file: Option<String>,
    plan_text: Option<String>,
    sql: Option<String>,
    format: String,
) -> anyhow::Result<()> {
    let fmt = output::OutputFormat::parse(&format);

    // Get plan text from file or inline
    let text = match (plan_file, plan_text) {
        (Some(path), _) => {
            fs::read_to_string(&path)
                .map_err(|e| anyhow::anyhow!("Failed to read plan file '{}': {}", path, e))?
        }
        (None, Some(txt)) => txt,
        (None, None) => {
            anyhow::bail!("Either --plan-file or --plan-text is required");
        }
    };

    // Parse the EXPLAIN plan
    let plan = ogexplain_core::parse(&text)
        .map_err(|e| anyhow::anyhow!("Failed to parse EXPLAIN plan: {}", e))?;

    // Analyze
    let report = if let Some(sql_text) = &sql {
        ogexplain_core::analyze_with_rewrite(&plan, Some(sql_text))
    } else {
        ogexplain_core::analyze(&plan)
    };

    // Heatmap and waterfall (optional diagnostics)
    let heatmap = ogexplain_core::heatmap(&plan);
    let waterfall = ogexplain_core::waterfall(&plan);

    match fmt {
        output::OutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct DiagnoseOutput<'a> {
                findings: &'a [ogexplain_core::analyzer::report::Finding],
                stats: &'a ogexplain_core::analyzer::context::GlobalStats,
                plan: &'a ogexplain_core::model::ExplainPlan,
                heatmap: Option<ogexplain_core::analyzer::heatmap::PlanHeatmap>,
                waterfall: Option<ogexplain_core::analyzer::waterfall::PlanWaterfall>,
            }

            let out = DiagnoseOutput {
                findings: &report.findings,
                stats: &report.stats,
                plan: &plan,
                heatmap,
                waterfall,
            };

            output::print_json(&out)?;
        }
        output::OutputFormat::Text => {
            println!("=== EXPLAIN Plan Diagnostics ===");
            println!("Total findings: {}", report.findings.len());
            println!();

            if report.findings.is_empty() {
                println!("No diagnostic findings — plan looks clean.");
            } else {
                // Count by severity
                let critical_count = report
                    .findings
                    .iter()
                    .filter(|f| f.severity == ogexplain_core::analyzer::report::Severity::Critical)
                    .count();
                let warning_count = report
                    .findings
                    .iter()
                    .filter(|f| f.severity == ogexplain_core::analyzer::report::Severity::Warning)
                    .count();
                let info_count = report
                    .findings
                    .iter()
                    .filter(|f| f.severity == ogexplain_core::analyzer::report::Severity::Info)
                    .count();

                println!(
                    "Critical: {} | Warning: {} | Info: {}",
                    critical_count, warning_count, info_count
                );
                println!();

                for finding in &report.findings {
                    let icon = match finding.severity {
                        ogexplain_core::analyzer::report::Severity::Critical => "🔴 CRITICAL",
                        ogexplain_core::analyzer::report::Severity::Warning => "🟡 WARNING",
                        ogexplain_core::analyzer::report::Severity::Info => "ℹ️ INFO",
                    };

                    println!("{} [{}]: {}", icon, finding.rule_id, finding.title);
                    println!("   Detail: {}", finding.detail);

                    if let Some(node_type) = &finding.node_type {
                        if let Some(line) = finding.node_line {
                            println!("   Node: {} (line {})", node_type, line);
                        } else {
                            println!("   Node: {}", node_type);
                        }
                    } else if let Some(line) = finding.node_line {
                        println!("   Line: {}", line);
                    }

                    if let Some(suggestion) = &finding.suggestion {
                        println!("   Suggestion: {}", suggestion);
                    }

                    if let Some(rewrite) = &finding.sql_rewrite {
                        println!(
                            "   SQL Rewrite available: {}",
                            rewrite.explanation
                        );
                    }
                    println!();
                }
            }

            // Plan summary
            println!("--- Plan Summary ---");
            println!("  Total nodes: {}", report.stats.total_nodes);
            println!("  Max depth:   {}", report.stats.max_depth);
            println!(
                "  Max node time: {:.2}ms",
                report.stats.max_node_time_ms
            );
            println!("  Max node rows: {:.0}", report.stats.max_node_rows);
            println!();

            // Heatmap info
            if let Some(hm) = heatmap {
                println!("--- Cost Deviation Heatmap ---");
                println!("  Max Q-Error: {:.2}", hm.summary.max_qerror);
                println!("  Nodes analyzed: {}", hm.entries.len());
                println!();
            }

            // Waterfall info
            if let Some(wf) = waterfall {
                println!("--- Resource Waterfall ---");
                println!("  CPU bottlenecks: {}", wf.bottlenecks.cpu_bottlenecks.len());
                println!("  Memory bottlenecks: {}", wf.bottlenecks.memory_bottlenecks.len());
                println!();
            }
        }
    }

    Ok(())
}
