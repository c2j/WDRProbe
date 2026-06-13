use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    if app.audit_issues.is_empty() {
        let msg = Paragraph::new("No audit issues found for the selected report.")
            .style(Theme::dim())
            .block(Block::bordered().title(" SQL Audit Issues "));
        frame.render_widget(msg, area);
        return;
    }

    // Vertical layout: header + list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(area);

    let total = app.audit_issues.len();
    let selected = app.audit_selected.min(total.saturating_sub(1));

    // Build display text for issues, highlighting the selected one
    let mut display_lines = Vec::new();
    for (i, issue) in app.audit_issues.iter().enumerate() {
        let prefix = if i == selected { ">" } else { " " };
        let severity_str = format!("{:?}", issue.severity);
        let severity_color = match severity_str.as_str() {
            "Critical" => "!!CRITICAL",
            "High" => "!HIGH",
            "Medium" => "MEDIUM",
            "Low" => "LOW",
            _ => "INFO",
        };
        let line = format!(
            "{} {}  {}",
            prefix, severity_color, issue.title
        );
        display_lines.push(line);

        // Show description on next line (indented)
        if i == selected || app.show_help {
            let desc = format!("   {}", issue.description);
            display_lines.push(desc);
            if let Some(ref sql) = issue.problematic_sql {
                let truncated = if sql.len() > 80 {
                    format!("{}...", &sql[..77])
                } else {
                    sql.clone()
                };
                display_lines.push(format!("   SQL: {}", truncated));
            }
            display_lines.push(format!("   Recommendation: {}", issue.recommendation));
            display_lines.push(format!("   Status: {:?} | Detected: {}", issue.status, issue.detected_at));
            display_lines.push(String::new());
        }
    }

    let text = display_lines.join("\n");
    let block = Block::bordered().title(format!(
        " SQL Audit Issues ({} total, j/k navigate) ",
        total
    ));
    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Theme::info())
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, chunks[0]);
}

/// Severity color mapping (for possible future use with styled spans)
pub fn severity_style(severity_str: &str) -> Style {
    match severity_str {
        "Critical" => Theme::critical(),
        "High" => Theme::warning(),
        "Medium" => Theme::info(),
        "Low" => Theme::dim(),
        _ => Theme::info(),
    }
}
