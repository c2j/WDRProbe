use rusqlite::params;
use wdrprobe_core::database::{get_connection, init_database, initialize_schema};

use crate::output;

/// A row from the sql_audit_issues table
#[derive(serde::Serialize)]
struct AuditIssue {
    id: i64,
    report_id: Option<i64>,
    sql_id: Option<i64>,
    issue_type: String,
    severity: String,
    title: String,
    description: Option<String>,
    problematic_sql: Option<String>,
    recommendation: Option<String>,
    status: String,
    detected_at: Option<String>,
}

/// Run the audit command: query SQL audit issues
pub fn run(db: String, report_id: Option<i64>, format: String) -> anyhow::Result<()> {
    let fmt = output::OutputFormat::parse(&format);

    let pool = init_database(&db)
        .map_err(|e| anyhow::anyhow!("Failed to init database: {}", e))?;
    let conn = get_connection(&pool)
        .map_err(|e| anyhow::anyhow!("Failed to get connection: {}", e))?;
    initialize_schema(&conn)
        .map_err(|e| anyhow::anyhow!("Failed to initialize schema: {}", e))?;

    let issues = query_audit_issues(&conn, report_id)?;

    match fmt {
        output::OutputFormat::Json => {
            output::print_json(&issues)?;
        }
        output::OutputFormat::Text => {
            if issues.is_empty() {
                println!("No audit issues found.");
                return Ok(());
            }

            if let Some(rid) = report_id {
                println!("=== SQL Audit Issues (Report #{}) ===", rid);
            } else {
                println!("=== SQL Audit Issues (All Reports) ===");
            }
            println!();

            for issue in &issues {
                let severity_prefix = match issue.severity.to_lowercase().as_str() {
                    "critical" => "🔴 CRITICAL",
                    "high" => "🟡 HIGH",
                    "medium" => "⚠️ MEDIUM",
                    "low" => "ℹ️ LOW",
                    _ => "🔵 INFO",
                };

                println!("{}: {}", severity_prefix, issue.title);
                if let Some(desc) = &issue.description {
                    println!("   Description: {}", desc);
                }
                if let Some(sql) = &issue.problematic_sql {
                    println!("   SQL: {}", sql);
                }
                if let Some(rec) = &issue.recommendation {
                    println!("   Recommendation: {}", rec);
                }
                println!("   Status: {} | Type: {}", issue.status, issue.issue_type);
                if let Some(dt) = &issue.detected_at {
                    println!("   Detected: {}", dt);
                }
                println!();
            }
        }
    }

    Ok(())
}

/// Query audit issues from the database
fn query_audit_issues(conn: &rusqlite::Connection, report_id: Option<i64>) -> anyhow::Result<Vec<AuditIssue>> {
    let sql = if report_id.is_some() {
        "SELECT id, report_id, sql_id, issue_type, severity, title, description, problematic_sql, recommendation, status, detected_at FROM sql_audit_issues WHERE report_id = ? ORDER BY detected_at DESC"
    } else {
        "SELECT id, report_id, sql_id, issue_type, severity, title, description, problematic_sql, recommendation, status, detected_at FROM sql_audit_issues ORDER BY detected_at DESC"
    };

    let mut stmt = conn.prepare(sql)?;

    let rows = if let Some(rid) = report_id {
        stmt.query_map(params![rid], row_mapper)?
    } else {
        stmt.query_map([], row_mapper)?
    };

    let mut issues = Vec::new();
    for row in rows {
        issues.push(row?);
    }

    Ok(issues)
}

fn row_mapper(row: &rusqlite::Row) -> rusqlite::Result<AuditIssue> {
    Ok(AuditIssue {
        id: row.get("id")?,
        report_id: row.get("report_id")?,
        sql_id: row.get("sql_id")?,
        issue_type: row.get("issue_type")?,
        severity: row.get("severity")?,
        title: row.get("title")?,
        description: row.get("description")?,
        problematic_sql: row.get("problematic_sql")?,
        recommendation: row.get("recommendation")?,
        status: row.get("status")?,
        detected_at: row.get("detected_at")?,
    })
}
