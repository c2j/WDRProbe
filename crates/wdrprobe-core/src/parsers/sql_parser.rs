// SQL parser
// Parses SQL text and execution plans from various sources

use crate::models::execution_plan::{
    ExecutionPlanNode, ExecutionPlanResponse, PlanMetadata, PlanNodeDetails,
};
use serde_json::Value;

/// Parse execution plan from JSON format (GaussDB/PostgreSQL FORMAT JSON)
pub fn parse_execution_plan_json(plan_json: &str) -> Result<ExecutionPlanNode, String> {
    let parsed: Value =
        serde_json::from_str(plan_json).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // PostgreSQL/GaussDB FORMAT JSON returns an array with one element
    let plan = if let Some(arr) = parsed.as_array() {
        arr.get(0)
            .and_then(|v| v.get("Plan"))
            .ok_or_else(|| "No Plan found in JSON array".to_string())?
    } else {
        parsed
            .get("Plan")
            .ok_or_else(|| "No Plan found in JSON object".to_string())?
    };

    parse_plan_node(plan)
}

/// Parse execution plan from text format (GaussDB/PostgreSQL EXPLAIN output)
pub fn parse_execution_plan_text(plan_text: &str) -> Result<ExecutionPlanNode, String> {
    let lines: Vec<&str> = plan_text.lines().collect();

    if lines.is_empty() {
        return Err("Empty plan text".to_string());
    }

    // Check if this is SQL+PLAN format (SQL statement + EXPLAIN output)
    if is_sql_plan_format(&plan_text) {
        return parse_sql_plan_format(&plan_text);
    }

    // Parse the root node from first line
    let root_line = lines.get(0).ok_or("Empty plan")?;
    let root = parse_plan_line(root_line, 0)?;

    // Recursively parse child nodes
    let (tree, _) = parse_children(&lines, 1, 1, root)?;

    Ok(tree)
}

/// Check if the text contains SQL statements with EXPLAIN output
pub fn is_sql_plan_format(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    
    // Look for patterns that indicate SQL+PLAN format
    let mut has_sql = false;
    let mut has_explain = false;
    let mut has_gaussdb_prompt = false;
    
    for line in lines.iter().take(20) { // Check first 20 lines for indicators
        let trimmed = line.trim().to_uppercase();
        
        // GaussDB prompt pattern
        if trimmed.starts_with("GAUSSDB=#") {
            has_gaussdb_prompt = true;
        }
        
        // SQL statement patterns
        if trimmed.starts_with("SELECT ") || 
           trimmed.starts_with("INSERT ") ||
           trimmed.starts_with("UPDATE ") ||
           trimmed.starts_with("DELETE ") ||
           trimmed.starts_with("CREATE ") ||
           trimmed.starts_with("DROP ") ||
           trimmed.starts_with("ALTER ") ||
           trimmed.starts_with("WITH ") ||
           (has_gaussdb_prompt && trimmed.contains("EXPLAIN")) {
            has_sql = true;
        }
        
        // EXPLAIN output patterns - extended for GaussDB format
        if trimmed.contains("QUERY PLAN") || 
           trimmed.contains("HASH JOIN") ||
           trimmed.contains("SEQ SCAN") ||
           trimmed.contains("INDEX SCAN") ||
           trimmed.contains("NESTED LOOP") ||
           trimmed.contains("STREAMING") ||
           trimmed.contains("RETRIEVE") ||
           trimmed.contains("HASH COND") ||
           trimmed.contains("SPAWN ON") ||
           trimmed.starts_with("->") ||
           trimmed.starts_with("STREAMING") {
            has_explain = true;
        }
        
        // If we have both patterns, it's SQL+PLAN format
        if has_sql && has_explain {
            return true;
        }
    }
    
    false
}

/// Parse SQL+PLAN format (SQL statements with EXPLAIN output)
pub fn parse_sql_plan_format(text: &str) -> Result<ExecutionPlanNode, String> {
    let lines: Vec<&str> = text.lines().collect();
    
    // Detect format type
    let is_tabular_format = lines.iter().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("id |") || 
        trimmed.contains("| E-rows |") ||
        trimmed.starts_with("----+")
    });
    
    if is_tabular_format {
        parse_gaussdb_tabular_format(&lines)
    } else {
        parse_gaussdb_text_format(&lines)
    }
}

/// Parse the new tabular GaussDB format
fn parse_gaussdb_tabular_format(lines: &[&str]) -> Result<ExecutionPlanNode, String> {
    let mut sql_part = String::new();
    let mut plan_lines = Vec::new();
    let mut in_predicate_section = false;
    
    // Extract SQL from GaussDB prompt
    for line in lines {
        let trimmed = line.trim().to_uppercase();
        if trimmed.starts_with("GAUSSDB=#") && trimmed.contains("EXPLAIN") {
            // Extract SQL from the prompt
            sql_part = line.trim().to_string();
            // Remove "gaussdb=# " prefix
            sql_part = sql_part.replace("gaussdb=# ", "").trim().to_string();
            // Remove trailing semicolon if present
            if sql_part.ends_with(";") {
                sql_part = sql_part[..sql_part.len()-1].trim().to_string();
            }
        } else if trimmed.starts_with("ID |") || trimmed.starts_with("----+") {
            // Start collecting plan lines after header
            in_predicate_section = false;
        } else if trimmed.starts_with("PREDICATE INFORMATION") || trimmed.starts_with("----") {
            in_predicate_section = true;
        } else if !in_predicate_section && !trimmed.is_empty() && 
                 !trimmed.starts_with("(") && 
                 !trimmed.starts_with("gaussdb=") {
            // Collect plan lines (skip empty lines, predicate section, and prompts)
            if !trimmed.starts_with("SET") && !trimmed.starts_with("Time:") {
                plan_lines.push(line.trim());
            }
        }
    }
    
    if plan_lines.is_empty() {
        return Err("No execution plan found in tabular format".to_string());
    }
    
    // Parse the tabular format
    let plan_tree = parse_tabular_plan_lines(&plan_lines)?;
    
    // Create root node with SQL
    let mut node_details = plan_tree.node_details.clone();
    node_details.output = Some(vec![format!("SQL: {}", sql_part)]);
    
    Ok(ExecutionPlanNode {
        operation: "SQL+PLAN".to_string(),
        cost: plan_tree.cost,
        rows: plan_rows_from_tabular(&plan_lines),
        actual_rows: None,
        actual_time: None,
        width: None,
        children: vec![plan_tree],
        node_details,
        warnings: Vec::new(),
        suggestions: Vec::new(),
    })
}

/// Parse tabular plan lines into execution plan tree
fn parse_tabular_plan_lines(plan_lines: &[&str]) -> Result<ExecutionPlanNode, String> {
    let mut nodes: Vec<ExecutionPlanNode> = Vec::new();
    
    for line in plan_lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("(") {
            continue;
        }
        
        // Parse tabular format: "  1 | ->  Streaming (type: GATHER)            |     20 |      16 | 28.69"
        if let Some(id_pos) = trimmed.find('|') {
            let id_str = trimmed[..id_pos].trim();
            if id_str.parse::<i32>().is_ok() {
                // Parse operation from the tabular format
                let remaining = &trimmed[id_pos+1..];
                if let Some(op_pos) = remaining.find('|') {
                    let operation_part = remaining[..op_pos].trim().to_string();
                    let operation = extract_operation_from_tabular(&operation_part);
                    let cost = extract_cost_from_tabular(remaining);
                    let rows = extract_rows_from_tabular(remaining);
                    let table_name = extract_table_from_operation(&operation_part);
                    
                    let node = ExecutionPlanNode {
                        operation,
                        cost,
                        rows,
                        actual_rows: None,
                        actual_time: None,
                        width: None,
                        children: Vec::new(),
                        node_details: PlanNodeDetails {
                            output: Some(vec![operation_part]),
                            filter: None,
                            buffers: None,
                            join_type: None,
                            hash_keys: None,
                            index_name: None,
                            table_name,
                        },
                        warnings: Vec::new(),
                        suggestions: Vec::new(),
                    };
                    nodes.push(node);
                }
            }
        }
    }
    
    // Build tree structure based on indentation
    build_plan_tree_from_nodes(&nodes)
}

/// Extract operation name from operation part
fn extract_operation_name(op_part: &str) -> String {
    let cleaned = op_part.trim();
    // Remove leading dashes and arrows like "-> "
    let cleaned = if cleaned.starts_with("->") {
        &cleaned[2..].trim()
    } else {
        cleaned
    };
    
    // Extract the main operation type (before parenthesis or first space)
    if let Some(paren_pos) = cleaned.find('(') {
        cleaned[..paren_pos].trim().to_string()
    } else if let Some(space_pos) = cleaned.find(' ') {
        cleaned[..space_pos].to_string()
    } else {
        cleaned.to_string()
    }
}

/// Extract table name from operation
fn extract_table_name(op_part: &str) -> Option<String> {
    let cleaned = op_part.trim();
    if cleaned.starts_with("->") {
        let op_part = &cleaned[2..].trim();
    }
    
    // Look for patterns like "Seq Scan on t2" or "Index Scan using idx_name on table"
    if let Some(on_pos) = op_part.rfind(" on ") {
        let table = &op_part[on_pos+4..].trim();
        if !table.is_empty() {
            return Some(table.to_string());
        }
    }
    None
}

/// Extract cost from tabular format
fn extract_cost_from_tabular(line: &str) -> f64 {
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() >= 5 {
        if let Some(cost_str) = parts.get(4) {
            if let Ok(cost) = cost_str.trim().parse::<f64>() {
                return cost;
            }
        }
    }
    0.0
}

/// Extract rows from tabular format
fn extract_rows_from_tabular(line: &str) -> u64 {
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() >= 3 {
        if let Some(rows_str) = parts.get(2) {
            if let Ok(rows) = rows_str.trim().parse::<u64>() {
                return rows;
            }
        }
    }
    0
}

/// Estimate total rows from plan lines
fn plan_rows_from_tabular(plan_lines: &[&str]) -> u64 {
    if !plan_lines.is_empty() {
        if let Some(first_line) = plan_lines.first() {
            return extract_rows_from_tabular(first_line);
        }
    }
    0
}

/// Build tree structure from flat nodes list
fn build_plan_tree_from_nodes(nodes: &[ExecutionPlanNode]) -> Result<ExecutionPlanNode, String> {
    if nodes.is_empty() {
        return Err("No plan nodes found".to_string());
    }
    
    // For now, return the first node as root with others as children
    // A more sophisticated implementation would build proper hierarchy
    let mut root = nodes[0].clone();
    
    if nodes.len() > 1 {
        for node in &nodes[1..] {
            root.children.push(node.clone());
        }
    }
    
    Ok(root)
}

/// Parse the original text-based GaussDB format
fn parse_gaussdb_text_format(lines: &[&str]) -> Result<ExecutionPlanNode, String> {
    // ... existing code for text format
    let mut sql_end_index = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim().to_uppercase();
        if trimmed.starts_with("GAUSSDB=#") && trimmed.contains("EXPLAIN") {
            sql_end_index = Some(i);
            break;
        } else if trimmed.starts_with("EXPLAIN") || (trimmed.contains("EXPLAIN") && trimmed.ends_with(";")) {
            sql_end_index = Some(i);
            break;
        }
    }
    
    if sql_end_index.is_none() {
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.ends_with(";") && 
               (trimmed.to_uppercase().starts_with("SELECT ") ||
                trimmed.to_uppercase().starts_with("INSERT ") ||
                trimmed.to_uppercase().starts_with("UPDATE ") ||
                trimmed.to_uppercase().starts_with("DELETE ")) {
                sql_end_index = Some(i);
                break;
            }
        }
    }
    
    let sql_end_index = sql_end_index.unwrap_or(0);
    
    // Extract SQL part
    let sql_part = if sql_end_index < lines.len() {
        let sql_line = lines[sql_end_index].trim();
        let sql_line = sql_line.replace("gaussdb=# ", "").trim().to_string();
        sql_line
    } else {
        lines[0].trim().to_string()
    };
    
    // Find plan start
    let mut plan_start_index = sql_end_index + 1;
    while plan_start_index < lines.len() {
        let line = lines[plan_start_index].trim();
        let line_upper = line.to_uppercase();
        
        if !line.is_empty() && !line.chars().all(|c| c == '-') {
            if line_upper.contains("QUERY PLAN") || 
               line_upper.contains("STREAMING") ||
               line_upper.starts_with("->") ||
               line_upper.contains("HASH JOIN") ||
               line_upper.contains("SEQ SCAN") ||
               line_upper.contains("INDEX SCAN") {
                break;
            }
        }
        plan_start_index += 1;
    }
    
    if plan_start_index >= lines.len() {
        return Err("No execution plan found".to_string());
    }
    
    if lines[plan_start_index].trim().to_uppercase().contains("QUERY PLAN") {
        plan_start_index += 1;
        while plan_start_index < lines.len() && 
              (lines[plan_start_index].trim().is_empty() || 
               lines[plan_start_index].trim().chars().all(|c| c == '-')) {
            plan_start_index += 1;
        }
    }
    
    if plan_start_index >= lines.len() {
        return Err("No execution plan content found".to_string());
    }
    
    let explain_part = lines[plan_start_index..]
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("\n");
    
    if explain_part.is_empty() {
        return Err("Empty execution plan".to_string());
    }
    
    let plan_tree = parse_execution_plan_text(&explain_part)?;
    
    // Create a root node that includes the SQL statement
    let mut node_details = plan_tree.node_details.clone();
    node_details.output = Some(vec![format!("SQL: {}", sql_part)]);
    
    Ok(ExecutionPlanNode {
        operation: "SQL+PLAN".to_string(),
        cost: plan_tree.cost,
        rows: plan_tree.rows,
        actual_rows: plan_tree.actual_rows,
        actual_time: plan_tree.actual_time,
        width: plan_tree.width,
        children: vec![plan_tree],
        node_details,
        warnings: Vec::new(),
        suggestions: Vec::new(),
    })
}

/// Parse a single plan line into a node
fn parse_plan_line(line: &str, _depth: usize) -> Result<ExecutionPlanNode, String> {
    let trimmed = line.trim();

    // Skip empty lines
    if trimmed.is_empty() {
        return Err("Empty line".to_string());
    }

    // Parse operation type - format is like "Seq Scan on users (cost=...)"
    // or "Index Scan using idx_name on table (cost=...)"
    // We need to extract everything before the opening paren that starts the cost section
    let operation = if let Some(cost_start) = trimmed.find(" (cost=") {
        // Extract everything before " (cost=" and parse the operation type
        let before_cost = &trimmed[..cost_start];
        // The operation typically ends at " on " or " using " or is the entire string
        if let Some(on_pos) = before_cost.find(" on ") {
            // Include "Scan" part: "Seq Scan on users" -> "Seq Scan"
            before_cost[..on_pos].to_string()
        } else if let Some(using_pos) = before_cost.find(" using ") {
            // "Index Scan using idx_name" -> "Index Scan"
            before_cost[..using_pos].to_string()
        } else {
            // Just take the first word
            before_cost
                .split_whitespace()
                .next()
                .unwrap_or("Unknown")
                .to_string()
        }
    } else {
        // No cost info found, take first word
        trimmed
            .split_whitespace()
            .next()
            .unwrap_or("Unknown")
            .to_string()
    };

    // Parse cost information
    let (cost, rows, actual_rows, _actual_time) = parse_metrics_from_line(trimmed);

    // Parse node details
    let node_details = parse_node_details(trimmed);

    Ok(ExecutionPlanNode {
        operation,
        cost,
        rows,
        actual_rows,
        actual_time: None,
        width: None,
        children: Vec::new(),
        node_details,
        warnings: Vec::new(),
        suggestions: Vec::new(),
    })
}

/// Parse cost metrics from a plan line
fn parse_metrics_from_line(line: &str) -> (f64, u64, Option<u64>, Option<f64>) {
    let mut cost = 0.0;
    let mut rows = 0;

    // Parse cost=... (e.g., "cost=0.00..123.45")
    if let Some(cost_start) = line.find("cost=") {
        let cost_part = &line[cost_start + 5..];
        let cost_end = cost_part
            .find(|c: char| !c.is_digit(10) && c != '.')
            .unwrap_or(cost_part.len());
        let cost_str = &cost_part[..cost_end];

        // Parse the two-part cost (startup..total)
        if let Some(dot_dot) = cost_str.find("..") {
            let total_cost = &cost_str[dot_dot + 2..];
            cost = total_cost.parse().unwrap_or(0.0);
        } else {
            cost = cost_str.parse().unwrap_or(0.0);
        }
    }

    // Parse rows=...
    if let Some(rows_start) = line.find("rows=") {
        let rows_part = &line[rows_start + 5..];
        let rows_end = rows_part
            .find(|c: char| !c.is_digit(10))
            .unwrap_or(rows_part.len());
        let rows_str = &rows_part[..rows_end];
        rows = rows_str.parse().unwrap_or(0);
    }

    // Parse actual rows and time from ANALYZE output
    let (actual_rows, _actual_time) = parse_actual_metrics(line);

    (cost, rows, actual_rows, _actual_time)
}

/// Parse actual execution metrics from ANALYZE output
fn parse_actual_metrics(line: &str) -> (Option<u64>, Option<f64>) {
    let mut actual_rows = None;
    let mut actual_time = None;

    // Format: "actual rows=123 loops=1"
    if let Some(ar_start) = line.find("actual rows=") {
        let ar_part = &line[ar_start + 12..];
        let ar_end = ar_part
            .find(|c: char| !c.is_digit(10))
            .unwrap_or(ar_part.len());
        actual_rows = ar_part[..ar_end].parse().ok();
    }

    // Format: "actual time=0.123..456.789"
    if let Some(at_start) = line.find("actual time=") {
        let at_part = &line[at_start + 12..];
        // Find the end of the time value (after the second "..")
        let at_end = at_part
            .find(|c: char| c == ' ' || c == ')')
            .unwrap_or(at_part.len());

        // Parse total time (after "..")
        if let Some(dot_dot) = at_part[..at_end].find("..") {
            let total_time = &at_part[dot_dot + 2..at_end];
            actual_time = total_time.parse().ok();
        }
    }

    (actual_rows, actual_time)
}

/// Parse node-specific details from a plan line
fn parse_node_details(line: &str) -> PlanNodeDetails {
    let mut details = PlanNodeDetails {
        output: None,
        filter: None,
        buffers: None,
        join_type: None,
        hash_keys: None,
        index_name: None,
        table_name: None,
    };

    // Extract table name (common patterns)
    if line.contains(" on ") {
        let on_start = line.find(" on ").unwrap() + 4;
        let on_part = &line[on_start..];
        let table_end = on_part
            .find(|c: char| c == ' ' || c == '(')
            .unwrap_or(on_part.len());
        details.table_name = Some(on_part[..table_end].trim().to_string());
    } else if line.contains(" Scan ") {
        let scan_start = line.find(" Scan ").unwrap() + 6;
        let scan_part = &line[scan_start..];
        let table_end = scan_part
            .find(|c: char| c == ' ' || c == '(')
            .unwrap_or(scan_part.len());
        details.table_name = Some(scan_part[..table_end].trim().to_string());
    }

    // Extract index name
    if let Some(idx_start) = line.find(" using ") {
        let using_part = &line[idx_start + 7..];
        let idx_end = using_part
            .find(|c: char| c == ' ' || c == '(')
            .unwrap_or(using_part.len());
        details.index_name = Some(using_part[..idx_end].trim().to_string());
    }

    // Extract filter condition
    if let Some(filter_start) = line.find(" Filter: ") {
        let filter_part = &line[filter_start + 9..];
        let filter_end = filter_part
            .find(|c: char| c == ')')
            .unwrap_or(filter_part.len());
        details.filter = Some(filter_part[..filter_end].trim().to_string());
    }

    // Extract join type
    for join_type in &["Hash Join", "Merge Join", "Nested Loop"] {
        if line.contains(join_type) {
            details.join_type = Some(join_type.to_string());
            break;
        }
    }

    // Extract hash keys
    if let Some(hash_start) = line.find(" Hash Cond: ") {
        let hash_part = &line[hash_start + 12..];
        let hash_end = hash_part
            .find(|c: char| c == ')')
            .unwrap_or(hash_part.len());
        let hash_cond = hash_part[..hash_end].trim().to_string();
        details.hash_keys = Some(vec![hash_cond]);
    }

    // Extract buffer information
    if let Some(buf_start) = line.find(" Buffers: ") {
        let buf_part = &line[buf_start + 10..];
        let buf_end = buf_part.find(|c: char| c == ')').unwrap_or(buf_part.len());
        details.buffers = Some(buf_part[..buf_end].trim().to_string());
    }

    details
}

/// Parse a plan node from JSON
fn parse_plan_node(json: &Value) -> Result<ExecutionPlanNode, String> {
    let operation = json
        .get("Node Type")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let cost = json
        .get("Total Cost")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let rows = json.get("Plan Rows").and_then(|v| v.as_u64()).unwrap_or(0) as u64;

    let actual_rows = json
        .get("Actual Rows")
        .and_then(|v| v.as_u64())
        .map(|v| v as u64);

    let actual_time = json.get("Actual Total Time").and_then(|v| v.as_f64());

    let width = json
        .get("Plan Width")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let node_details = parse_json_node_details(json);

    // Parse child plans
    let mut children = Vec::new();
    if let Some(plans) = json.get("Plans").and_then(|v| v.as_array()) {
        for plan in plans {
            children.push(parse_plan_node(plan)?);
        }
    }

    Ok(ExecutionPlanNode {
        operation,
        cost,
        rows,
        actual_rows,
        actual_time,
        width,
        children,
        node_details,
        warnings: Vec::new(),
        suggestions: Vec::new(),
    })
}

/// Parse node details from JSON
fn parse_json_node_details(json: &Value) -> PlanNodeDetails {
    let mut details = PlanNodeDetails {
        output: None,
        filter: None,
        buffers: None,
        join_type: None,
        hash_keys: None,
        index_name: None,
        table_name: None,
    };

    if let Some(table) = json.get("Relation Name").and_then(|v| v.as_str()) {
        details.table_name = Some(table.to_string());
    }

    if let Some(idx) = json.get("Index Name").and_then(|v| v.as_str()) {
        details.index_name = Some(idx.to_string());
    }

    if let Some(filter) = json.get("Filter").and_then(|v| v.as_str()) {
        details.filter = Some(filter.to_string());
    }

    if let Some(hash_cond) = json.get("Hash Cond").and_then(|v| v.as_str()) {
        details.hash_keys = Some(vec![hash_cond.to_string()]);
    }

    if let Some(join_type) = json.get("Join Type").and_then(|v| v.as_str()) {
        details.join_type = Some(join_type.to_string());
    }

    details
}

/// Parse child nodes from text format
fn parse_children(
    lines: &[&str],
    start_idx: usize,
    parent_depth: usize,
    mut parent_node: ExecutionPlanNode,
) -> Result<(ExecutionPlanNode, usize), String> {
    let mut current_idx = start_idx;

    while current_idx < lines.len() {
        let line = lines[current_idx];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            current_idx += 1;
            continue;
        }

        // Calculate depth based on leading spaces
        let depth = line.len() - line.trim_start().len();

        // If we're back at parent level or higher, we're done with this branch
        if depth <= parent_depth {
            break;
        }

        // Parse this node
        let node = parse_plan_line(line, depth)?;

        // Recursively parse this node's children
        let (child_node, next_idx) = parse_children(lines, current_idx + 1, depth, node)?;
        parent_node.children.push(child_node);
        current_idx = next_idx;
    }

    Ok((parent_node, current_idx))
}

/// Analyze execution plan for performance issues
pub fn analyze_execution_plan(plan: &ExecutionPlanNode) -> ExecutionPlanResponse {
    let (total_cost, total_rows, max_depth, node_count, has_actual_stats) =
        calculate_plan_metadata(plan);

    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();

    analyze_node(plan, &mut warnings, &mut suggestions);

    let optimization_warnings = warnings.len() as u32;

    ExecutionPlanResponse {
        success: true,
        plan_tree: plan.clone(),
        plan_metadata: PlanMetadata {
            total_cost,
            total_rows,
            plan_depth: max_depth,
            node_count,
            optimization_warnings,
            estimated_time_ms: total_cost * 0.01, // Rough estimate
            gaussdb_format: true,
            has_actual_stats,
        },
        warnings,
        suggestions,
    }
}

/// Calculate metadata for a plan tree
fn calculate_plan_metadata(node: &ExecutionPlanNode) -> (f64, u64, u32, u32, bool) {
    let mut total_cost = node.cost;
    let mut total_rows = node.rows;
    let mut node_count = 1u32;
    let mut max_depth = 0u32;
    let mut has_actual_stats = node.actual_rows.is_some();

    for child in &node.children {
        let (child_cost, child_rows, child_depth, child_count, child_stats) =
            calculate_plan_metadata(child);
        total_cost = total_cost.max(child_cost);
        total_rows += child_rows;
        node_count += child_count;
        max_depth = max_depth.max(child_depth);
        has_actual_stats = has_actual_stats || child_stats;
    }

    (
        total_cost,
        total_rows,
        max_depth + 1,
        node_count,
        has_actual_stats,
    )
}

/// Recursively analyze a node for performance issues
fn analyze_node(
    node: &ExecutionPlanNode,
    warnings: &mut Vec<String>,
    suggestions: &mut Vec<String>,
) {
    // Check for full table scans on large tables
    if node.operation.contains("Seq Scan") {
        if node.rows > 100000 {
            warnings.push(format!(
                "Full table scan on '{}' will process {} rows",
                node.node_details.table_name.as_deref().unwrap_or("unknown"),
                node.rows
            ));
            if let Some(table) = &node.node_details.table_name {
                suggestions.push(format!(
                    "Consider creating an index on table '{}' to reduce full table scans",
                    table
                ));
            }
        }
    }

    // Check for nested loop with high row count
    if node.operation.contains("Nested Loop") && node.cost > 1000.0 {
        warnings.push(format!(
            "Nested Loop join has high cost ({:.2}), consider Hash Join instead",
            node.cost
        ));
        suggestions.push(
            "Consider increasing work_mem to enable Hash Join for better performance".to_string(),
        );
    }

    // Check for missing index usage
    if node.operation.contains("Seq Scan") && node.node_details.filter.is_some() {
        warnings.push(format!(
            "Sequential scan with filter condition may benefit from an index: {}",
            node.node_details.filter.as_deref().unwrap_or("")
        ));
    }

    // Check for sort operations
    if node.operation.contains("Sort") && node.rows > 10000 {
        warnings.push(format!(
            "Sort operation on {} rows may be expensive",
            node.rows
        ));
        suggestions
            .push("Consider adding an index with ORDER BY columns to avoid sorting".to_string());
    }

    // Check for hash join with high memory usage
    if node.operation.contains("Hash Join") && node.rows > 100000 {
        warnings.push(format!(
            "Hash Join on {} rows may require significant memory",
            node.rows
        ));
        suggestions.push("Ensure work_mem is sufficient for hash table size".to_string());
    }

    // Check for actual vs estimated row mismatch
    if let (Some(actual), Some(rows)) = (node.actual_rows, Some(node.rows)) {
        if actual as f64 > rows as f64 * 10.0 || (actual as f64) < rows as f64 * 0.1 {
            warnings.push(format!(
                "Row estimation error: estimated {} but actual {} rows ({}x difference)",
                rows,
                actual,
                if actual > rows {
                    actual as f64 / rows as f64
                } else {
                    rows as f64 / actual as f64
                }
            ));
            suggestions.push("Run ANALYZE on the involved tables to update statistics".to_string());
        }
    }

    // Recursively analyze children
    for child in &node.children {
        analyze_node(child, warnings, suggestions);
    }
}

/// Extract SQL text from a query (removes EXPLAIN prefix if present)
pub fn extract_sql_from_explain(query: &str) -> Result<String, String> {
    let query = query.trim();
    let query_upper = query.to_uppercase();

    // Check for EXPLAIN prefixes
    let explain_patterns = [
        "EXPLAIN (FORMAT JSON) ",
        "EXPLAIN (FORMAT TEXT) ",
        "EXPLAIN (ANALYZE, FORMAT JSON) ",
        "EXPLAIN (ANALYZE, BUFFERS) ",
        "EXPLAIN ANALYZE ",
        "EXPLAIN ",
    ];

    for pattern in &explain_patterns {
        if query_upper.starts_with(pattern) {
            // Extract the actual SQL after EXPLAIN
            let remaining = &query[pattern.len()..];
            return Ok(trim_semicolon(remaining.trim()));
        }
    }

    // No EXPLAIN prefix found, return as-is
    Ok(trim_semicolon(query))
}

/// Trim trailing semicolon from SQL
fn trim_semicolon(sql: &str) -> String {
    let trimmed = sql.trim();
    if trimmed.ends_with(';') {
        trimmed[..trimmed.len() - 1].trim().to_string()
    } else {
        trimmed.to_string()
    }
}

/// Generate EXPLAIN JSON query for a SQL statement
pub fn generate_explain_json(sql: &str) -> String {
    format!("EXPLAIN (FORMAT JSON) {}", sql.trim().trim_end_matches(';'))
}

/// Generate EXPLAIN ANALYZE query for a SQL statement
pub fn generate_explain_analyze(sql: &str) -> String {
    format!("EXPLAIN ANALYZE {}", sql.trim().trim_end_matches(';'))
}

/// Parse SQL to extract table names
pub fn extract_table_names(sql: &str) -> Vec<String> {
    let mut tables = Vec::new();
    let upper_sql = sql.to_uppercase();

    // Simple regex-like patterns for common SQL clauses
    // FROM clause
    if let Some(from_start) = upper_sql.find("FROM ") {
        let from_part = &sql[from_start + 5..];
        let table_end = find_table_end(from_part);
        tables.push(from_part[..table_end].trim().to_string());
    }

    // JOIN clauses
    let mut join_pos = 0;
    while let Some(join_start) = upper_sql[join_pos..].find("JOIN ") {
        let absolute_pos = join_pos + join_start;
        let join_part = &sql[absolute_pos + 5..];
        let table_end = find_table_end(join_part);
        tables.push(join_part[..table_end].trim().to_string());
        join_pos = absolute_pos + 5;
    }

    tables.dedup();
    tables
}

/// Find the end of a table reference in SQL
fn find_table_end(sql_part: &str) -> usize {
    let chars: Vec<char> = sql_part.chars().collect();
    let mut depth = 0usize;
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            ' ' | '\t' | '\n' | ',' => {
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        i += 1;
    }

    i.min(sql_part.len())
}

/// Validate if SQL syntax appears correct (basic check)
pub fn validate_sql_syntax(sql: &str) -> Result<(), String> {
    let trimmed = sql.trim();

    if trimmed.is_empty() {
        return Err("SQL is empty".to_string());
    }

    // Check for balanced parentheses
    let mut depth = 0i32;
    let mut in_string = false;
    let mut prev_char = ' ';
    let chars: Vec<char> = trimmed.chars().collect();

    for i in 0..chars.len() {
        let c = chars[i];

        // Handle string literals
        if (c == '\'' || c == '"') && prev_char != '\\' {
            in_string = !in_string;
        }

        if !in_string {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth < 0 {
                        return Err("Unbalanced parentheses: too many closing".to_string());
                    }
                }
                ';' => {
                    // Check if semicolon is in appropriate position
                    if i < chars.len() - 1 {
                        return Err("Semicolon should be at the end of SQL".to_string());
                    }
                }
                _ => {}
            }
        }

        prev_char = c;
    }

    if depth != 0 {
        return Err("Unbalanced parentheses: too many opening".to_string());
    }

    // Check for SELECT, INSERT, UPDATE, DELETE, etc.
    let upper = trimmed.to_uppercase();
    let valid_starts = [
        "SELECT", "INSERT", "UPDATE", "DELETE", "CREATE", "DROP", "ALTER", "TRUNCATE", "GRANT",
        "REVOKE", "WITH", "EXPLAIN",
    ];

    let starts_validly = valid_starts
        .iter()
        .any(|start| upper.starts_with(start) || upper.starts_with(&format!("{} ", start)));

    if !starts_validly {
        return Err(format!(
            "SQL must start with a valid statement (SELECT, INSERT, UPDATE, etc.)"
        ));
    }

    Ok(())
}

/// Extract operation name from tabular operation string
fn extract_operation_from_tabular(op_str: &str) -> String {
    let cleaned = op_str.trim();
    
    // Remove leading "-> " if present
    let cleaned = if cleaned.starts_with("->") {
        &cleaned[2..].trim()
    } else {
        cleaned
    };
    
    // Extract main operation (before parentheses or first space)
    if let Some(paren_pos) = cleaned.find('(') {
        cleaned[..paren_pos].trim().to_string()
    } else if let Some(space_pos) = cleaned.find(' ') {
        cleaned[..space_pos].to_string()
    } else {
        cleaned.to_string()
    }
}

/// Extract table name from operation string
fn extract_table_from_operation(op_str: &str) -> Option<String> {
    let cleaned = op_str.trim();
    
    // Remove leading "-> " if present
    let cleaned = if cleaned.starts_with("->") {
        &cleaned[2..].trim()
    } else {
        cleaned
    };
    
    // Look for " on table_name" pattern
    if let Some(on_pos) = cleaned.rfind(" on ") {
        let table_part = &cleaned[on_pos + 4..].trim();
        if !table_part.is_empty() {
            return Some(table_part.to_string());
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_sql_from_explain() {
        let sql = "EXPLAIN SELECT * FROM users WHERE id = 1";
        let result = extract_sql_from_explain(sql).unwrap();
        assert_eq!(result, "SELECT * FROM users WHERE id = 1");

        let sql = "EXPLAIN (FORMAT JSON) SELECT * FROM products";
        let result = extract_sql_from_explain(sql).unwrap();
        assert_eq!(result, "SELECT * FROM products");

        let sql = "EXPLAIN ANALYZE SELECT count(*) FROM orders";
        let result = extract_sql_from_explain(sql).unwrap();
        assert_eq!(result, "SELECT count(*) FROM orders");
    }

    #[test]
    fn test_generate_explain_json() {
        let sql = "SELECT * FROM users";
        let result = generate_explain_json(sql);
        assert_eq!(result, "EXPLAIN (FORMAT JSON) SELECT * FROM users");
    }

    #[test]
    fn test_extract_table_names() {
        let sql = "SELECT * FROM users u JOIN orders o ON u.id = o.user_id";
        let tables = extract_table_names(sql);
        assert_eq!(tables, vec!["users", "orders"]);
    }

    #[test]
    fn test_validate_sql_syntax() {
        assert!(validate_sql_syntax("SELECT * FROM users").is_ok());
        assert!(validate_sql_syntax("").is_err());
        assert!(validate_sql_syntax("SELECT * FROM (SELECT * FROM users").is_err());
        assert!(validate_sql_syntax("SELECT * FROM users WHERE id = 1; AND").is_err());
    }

    #[test]
    fn test_parse_execution_plan_json() {
        let json = r#"[{
            "Plan": {
                "Node Type": "Seq Scan",
                "Relation Name": "users",
                "Total Cost": 12.34,
                "Plan Rows": 100
            }
        }]"#;

        let result = parse_execution_plan_json(json);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.operation, "Seq Scan");
        assert_eq!(plan.node_details.table_name, Some("users".to_string()));
    }

    #[test]
    fn test_parse_execution_plan_text() {
        let text = "Seq Scan on users  (cost=0.00..12.34 rows=100 width=4)";
        let result = parse_execution_plan_text(text);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.operation, "Seq Scan");
        assert_eq!(plan.cost, 12.34);
        assert_eq!(plan.rows, 100);
    }
}
