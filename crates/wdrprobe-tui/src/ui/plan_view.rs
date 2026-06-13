use std::collections::HashSet;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::components::tree::{self, FlatNode};
use crate::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    if app.selected_report_id.is_none() {
        let msg = Paragraph::new("Select a report first (Tab to Reports page).")
            .style(Theme::dim())
            .block(Block::bordered().title(" Plan View "));
        frame.render_widget(msg, area);
        return;
    }

    if app.plan_nodes.is_empty() {
        let msg =
            Paragraph::new("No execution plans found for this report.").style(Theme::dim());
        frame.render_widget(msg, area);
        return;
    }

    // Vertical split: plan tree (top 70%) + node detail (bottom 30%)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    // Render the plan tree
    tree::render_tree(
        frame,
        chunks[0],
        &app.plan_nodes,
        app.plan_selected.min(app.plan_nodes.len().saturating_sub(1)),
        " Execution Plan ",
    );

    // Show selected node detail in bottom pane
    if !app.plan_nodes.is_empty() && app.plan_selected < app.plan_nodes.len() {
        let node = &app.plan_nodes[app.plan_selected];
        let mut detail_lines = Vec::new();
        detail_lines.push(format!("Node: {}", node.text));

        // Find the corresponding TreeNode to show warnings/suggestions
        // Actually, we store FlatNode, so we'd need to track that separately.
        // For now, show basic info and let the user browse.

        let detail_text = detail_lines.join("\n");
        let detail = Paragraph::new(detail_text)
            .block(Block::bordered().title(" Node Details "))
            .style(Theme::info())
            .wrap(Wrap { trim: false });
        frame.render_widget(detail, chunks[1]);
    } else {
        let detail = Paragraph::new("Select a node to see details.")
            .block(Block::bordered().title(" Node Details "))
            .style(Theme::dim());
        frame.render_widget(detail, chunks[1]);
    }
}

/// Build the flat node list from execution plans stored in the DB.
/// This is called when loading a report's plans.
pub fn build_plan_nodes(
    plans: &[wdrprobe_core::models::SqlExecutionPlan],
) -> Vec<FlatNode> {
    if plans.is_empty() {
        return Vec::new();
    }

    // Use the first plan's tree (most relevant)
    let plan = &plans[0];
    let mut id_counter = 1;
    let root = tree::plan_node_to_tree_node(&plan.plan_tree, &mut id_counter);

    let mut expanded = HashSet::new();
    // Expand the root level by default
    expanded.insert(root.id);

    let mut flat_nodes = Vec::new();
    tree::flatten_tree(&[root], &expanded, &mut flat_nodes, 0);
    flat_nodes
}

/// Toggle expand/collapse on the selected node in the flat list.
/// Returns the new expanded set.
pub fn toggle_node(flat_nodes: &[FlatNode], selected: usize, expanded: &mut HashSet<usize>) {
    if selected >= flat_nodes.len() {
        return;
    }
    let node = &flat_nodes[selected];
    if !node.has_children {
        return;
    }
    if expanded.contains(&node.id) {
        expanded.remove(&node.id);
    } else {
        expanded.insert(node.id);
    }
}

/// Expand all nodes
pub fn expand_all(flat_nodes: &[FlatNode], expanded: &mut HashSet<usize>) {
    for node in flat_nodes {
        if node.has_children {
            expanded.insert(node.id);
        }
    }
}

/// Collapse all nodes except root
pub fn collapse_all(flat_nodes: &[FlatNode], expanded: &mut HashSet<usize>) {
    expanded.clear();
    // Keep root expanded if it has children
    if let Some(first) = flat_nodes.first() {
        if first.has_children {
            expanded.insert(first.id);
        }
    }
}
