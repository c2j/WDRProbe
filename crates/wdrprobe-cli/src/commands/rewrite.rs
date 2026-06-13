use clap::Args;

#[derive(Args)]
pub struct RewriteArgs {
    /// SQL text to rewrite
    #[arg(long)]
    sql: Option<String>,

    /// Read SQL from file
    #[arg(long)]
    sql_file: Option<String>,

    /// Schema JSON: {"table": {"column": "type"}}
    #[arg(long)]
    schema_json: Option<String>,

    /// Output format: text or json
    #[arg(long, default_value = "text")]
    format: String,
}

pub fn run(args: RewriteArgs) -> anyhow::Result<()> {
    // Get SQL text
    let sql = if let Some(s) = args.sql {
        s
    } else if let Some(path) = args.sql_file {
        std::fs::read_to_string(&path)?
    } else {
        anyhow::bail!("Either --sql or --sql-file is required");
    };

    // Parse schema if provided
    let schema = if let Some(ref json) = args.schema_json {
        Some(
            serde_json::from_str::<std::collections::HashMap<
                String,
                std::collections::HashMap<String, String>,
            >>(json)
            .map_err(|e| anyhow::anyhow!("Invalid schema JSON: {}", e))?,
        )
    } else {
        None
    };

    // Create adapter and rewrite
    let registry = metamorphosis_core::RuleRegistry::new(metamorphosis_rules::builtin_rules());
    let engine = metamorphosis_core::RewriteEngine::new(registry);

    let (stmt_infos, errors) = ogsql_parser::Parser::parse_sql(&sql);
    if !errors.is_empty() {
        anyhow::bail!("Parse errors: {:?}", errors);
    }

    let statements: Vec<_> = stmt_infos.into_iter().map(|si| si.statement).collect();
    let config = metamorphosis_core::RewriteConfig::default();
    let ctx = metamorphosis_core::RewriteContext {
        version: None,
        schema: schema.as_ref(),
        config: &config,
        source_file: None,
        known_variables: None,
    };

    let result = engine.rewrite(&ctx, statements);

    // Output
    match args.format.as_str() {
        "json" => {
            let output = serde_json::json!({
                "original_sql": sql,
                "changed": result.changed,
                "suggestions": result.suggestions,
                "match_failures": result.match_failures,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            // Text format
            if result.changed {
                let formatter = ogsql_parser::formatter::SqlFormatter::new();
                let rewritten: Vec<_> = result
                    .statements
                    .iter()
                    .map(|s| formatter.format_statement(s))
                    .collect();
                println!("=== SQL Rewrite Result ===");
                println!("Original: {}", sql);
                println!("Rewritten: {}", rewritten.join(";\n"));
                println!("\nChanged: YES");
            } else {
                println!("=== SQL Rewrite Result ===");
                println!("No changes applied.");
                println!("\nChanged: NO");
            }

            if !result.suggestions.is_empty() {
                println!("\n--- Suggestions ---");
                for s in &result.suggestions {
                    println!(
                        "\n[{}] {} (confidence: {:?})",
                        s.rule_id, s.rule_description, s.confidence
                    );
                    for note in &s.notes {
                        println!("  - {}", note);
                    }
                }
            }

            if !result.match_failures.is_empty() {
                println!("\n--- Rule Match Results ---");
                for f in &result.match_failures {
                    println!("  {} → {}", f.rule_id, f.reason);
                }
            }
        }
    }

    Ok(())
}
