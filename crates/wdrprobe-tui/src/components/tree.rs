use std::collections::HashSet;

use ratatui::layout::Rect;
use ratatui::widgets::{Block, List, ListItem, ListState};
use ratatui::Frame;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: usize,
    pub text: String,
    pub children: Vec<TreeNode>,
    pub severity: Option<Severity>,
}

#[derive(Debug, Clone)]
pub struct FlatNode {
    pub id: usize,
    pub depth: usize,
    pub text: String,
    pub has_children: bool,
    pub expanded: bool,
    pub severity: Option<Severity>,
}

#[derive(Debug, Clone)]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

/// Convert a tree structure into a flat vec of visible nodes.
/// depth tracks the indentation level. expanded_set controls which nodes are expanded.
pub fn flatten_tree(
    nodes: &[TreeNode],
    expanded: &HashSet<usize>,
    result: &mut Vec<FlatNode>,
    depth: usize,
) {
    for node in nodes {
        let is_expanded = expanded.contains(&node.id);
        result.push(FlatNode {
            id: node.id,
            depth,
            text: node.text.clone(),
            has_children: !node.children.is_empty(),
            expanded: is_expanded,
            severity: node.severity.clone(),
        });
        if is_expanded {
            flatten_tree(&node.children, expanded, result, depth + 1);
        }
    }
}

/// Convert a plan ExecutionPlanNode (from wdrprobe-core) into our TreeNode
pub fn plan_node_to_tree_node(
    plan_node: &wdrprobe_core::models::ExecutionPlanNode,
    id_counter: &mut usize,
) -> TreeNode {
    let node_id = {
        let id = *id_counter;
        *id_counter += 1;
        id
    };

    // Build text from operation + cost/rows
    let mut text = plan_node.operation.clone();
    if plan_node.width.is_some() || plan_node.rows > 0 || plan_node.cost > 0.0 {
        text.push_str(&format!(
            " (rows={} cost={:.2})",
            plan_node.rows, plan_node.cost
        ));
    }
    if let Some(actual_rows) = plan_node.actual_rows {
        text.push_str(&format!(" actual_rows={}", actual_rows));
    }
    if let Some(actual_time) = plan_node.actual_time {
        text.push_str(&format!(" time={:.3}ms", actual_time));
    }
    if let Some(ref filter) = plan_node.node_details.filter {
        text.push_str(&format!(" Filter: {}", filter));
    }

    // Determine severity from warnings/suggestions
    let severity = if !plan_node.warnings.is_empty() {
        // Check for critical-style warnings
        let combined = plan_node.warnings.join(" ").to_lowercase();
        if combined.contains("full table scan")
            || combined.contains("sequential scan")
            || combined.contains("missing index")
        {
            Some(Severity::Warning)
        } else {
            Some(Severity::Info)
        }
    } else {
        None
    };

    let children: Vec<TreeNode> = plan_node
        .children
        .iter()
        .map(|c| plan_node_to_tree_node(c, id_counter))
        .collect();

    TreeNode {
        id: node_id,
        text,
        children,
        severity,
    }
}

/// Render a flattened tree as a list widget
pub fn render_tree(
    frame: &mut Frame,
    area: Rect,
    flat_nodes: &[FlatNode],
    selected: usize,
    title: &str,
) {
    let items: Vec<ListItem> = flat_nodes
        .iter()
        .map(|node| {
            let indent = "  ".repeat(node.depth);
            let icon = if node.has_children {
                if node.expanded {
                    "▾ "
                } else {
                    "▸ "
                }
            } else {
                "· "
            };
            let sev_str = match &node.severity {
                Some(Severity::Critical) => " !!",
                Some(Severity::Warning) => " !",
                Some(Severity::Info) => " *",
                None => "",
            };
            ListItem::new(format!("{}{}{}{}", indent, icon, node.text, sev_str))
        })
        .collect();

    let list = List::new(items)
        .block(Block::bordered().title(title))
        .highlight_style(crate::theme::Theme::selected());

    let mut state = ListState::default();
    if selected < flat_nodes.len() {
        state.select(Some(selected));
    }
    frame.render_stateful_widget(list, area, &mut state);
}
