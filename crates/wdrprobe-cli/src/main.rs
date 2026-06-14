use clap::{Parser, Subcommand};

mod commands;
mod output;

/// WDRProbe CLI — GaussDB WDR Report Analysis Tool
#[derive(Parser)]
#[command(name = "wdrprobe", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Import a WDR HTML report file into the database
    Import {
        /// Path to the WDR HTML file
        #[arg(long)]
        file: String,

        /// Path to the SQLite database file (created if not exists)
        #[arg(long, default_value = "./wdrprobe.db")]
        db: String,

        /// Instance name for the report
        #[arg(long, default_value = "unknown")]
        instance: String,
    },

    /// List all imported WDR reports in the database
    List {
        /// Path to the SQLite database file
        #[arg(long, default_value = "./wdrprobe.db")]
        db: String,

        /// Output format: text or json
        #[arg(long, default_value = "text")]
        format: String,

        /// Maximum number of reports to show
        #[arg(long)]
        limit: Option<i32>,
    },

    /// Show detailed analysis of a specific WDR report
    Analyze {
        /// Path to the SQLite database file
        #[arg(long, default_value = "./wdrprobe.db")]
        db: String,

        /// Report ID to analyze
        #[arg(long)]
        report_id: i64,

        /// Output format: text or json
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Query SQL audit issues for a report
    Audit {
        /// Path to the SQLite database file
        #[arg(long, default_value = "./wdrprobe.db")]
        db: String,

        /// Report ID to audit (optional — if omitted, shows all)
        #[arg(long)]
        report_id: Option<i64>,

        /// Output format: text or json
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Export a WDR report to JSON or CSV file
    Export {
        /// Path to the SQLite database file
        #[arg(long, default_value = "./wdrprobe.db")]
        db: String,

        /// Report ID to export
        #[arg(long)]
        report_id: i64,

        /// Output format: json or csv
        #[arg(long, default_value = "json")]
        format: String,

        /// Output file path (defaults to stdout)
        #[arg(short = 'o', long)]
        output: Option<String>,
    },

    /// Diagnose an EXPLAIN plan using ogexplain-core (25 diagnostic rules)
    #[cfg(feature = "diagnostic-engines")]
    Diagnose {
        /// Path to a file containing the EXPLAIN output text
        #[arg(long)]
        plan_file: Option<String>,

        /// Inline EXPLAIN output text
        #[arg(long)]
        plan_text: Option<String>,

        /// Optional SQL text for rewrite suggestions
        #[arg(long)]
        sql: Option<String>,

        /// Output format: text or json
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Rewrite SQL using metamorphosis rules (SELECT * expansion, subquery-to-join, etc.)
    #[cfg(feature = "diagnostic-engines")]
    Rewrite(crate::commands::rewrite::RewriteArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Import { file, db, instance } => commands::import::run(file, db, instance),
        Commands::List { db, format, limit } => commands::list::run(db, format, limit),
        Commands::Analyze {
            db,
            report_id,
            format,
        } => commands::analyze::run(db, report_id, format),
        Commands::Audit {
            db,
            report_id,
            format,
        } => commands::audit::run(db, report_id, format),
        Commands::Export {
            db,
            report_id,
            format,
            output,
        } => commands::export::run(db, report_id, &format, output),
        #[cfg(feature = "diagnostic-engines")]
        Commands::Diagnose {
            plan_file,
            plan_text,
            sql,
            format,
        } => commands::diagnose::run(plan_file, plan_text, sql, format),
        #[cfg(feature = "diagnostic-engines")]
        Commands::Rewrite(args) => commands::rewrite::run(args),
    }
}

/// Truncate SQL text for display, appending "..." if truncated
#[allow(dead_code)]
pub fn truncate_sql(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len.saturating_sub(3)])
    }
}
