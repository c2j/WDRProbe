use ogsql_parser::analyzer::schema::SchemaMap;
use wdrprobe_core::database::DatabaseOperations;
use wdrprobe_core::database::DatabasePool;

/// Parse a JSON schema string into SchemaMap
/// Format: { "table_name": { "column_name": "data_type" } }
pub fn parse_schema_json(json: &str) -> Result<SchemaMap, String> {
    serde_json::from_str(json).map_err(|e| format!("Invalid schema JSON: {}", e))
}

/// Extract schema from WDR report data
/// Uses object_stats table to get table names, and best-effort column extraction from top SQLs
pub fn extract_schema_from_wdr(
    pool: &DatabasePool,
    report_id: i64,
) -> Result<SchemaMap, String> {
    let mut schema: SchemaMap = SchemaMap::new();

    // Get object stats for table names
    let objects = pool
        .get_object_stats(report_id)
        .map_err(|e| format!("Failed to get object stats: {}", e))?;

    // Register table names (without column info — WDR reports don't have column-level data)
    for obj in &objects {
        if obj.object_type == "table" || obj.object_type == "TABLE" {
            schema
                .entry(obj.object_name.to_lowercase().clone())
                .or_insert_with(std::collections::HashMap::new);
        }
    }

    // Get top SQLs and try to extract table.column references
    let top_sqls = pool
        .get_top_sqls_by_report(report_id)
        .map_err(|e| format!("Failed to get top SQLs: {}", e))?;

    for sql in &top_sqls {
        extract_columns_from_sql(&sql.sql_text, &mut schema);
    }

    Ok(schema)
}

/// Best-effort column extraction from SQL text
/// Looks for patterns like "table.column" or "column" in SELECT/WHERE clauses
fn extract_columns_from_sql(sql_text: &str, schema: &mut SchemaMap) {
    let re = regex::Regex::new(r"(?i)\b(\w+)\.(\w+)\b").unwrap();
    for cap in re.captures_iter(sql_text) {
        let table = cap[1].to_lowercase();
        let column = cap[2].to_lowercase();
        // Skip obvious non-table prefixes
        if matches!(
            table.as_str(),
            "t" | "a" | "b" | "c" | "t1" | "t2" | "a1" | "b1"
        ) {
            continue;
        }
        schema
            .entry(table)
            .or_insert_with(std::collections::HashMap::new)
            .entry(column)
            .or_insert_with(|| "unknown".to_string());
    }
}
