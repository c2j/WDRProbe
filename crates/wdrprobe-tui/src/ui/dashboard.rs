use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::components::gauge;
use crate::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    // If no data, show centered message
    if app.reports.is_empty() {
        let msg = Paragraph::new("No data. Import WDR reports first.")
            .style(Theme::info())
            .block(Block::bordered().title(" Dashboard "));
        frame.render_widget(msg, area);
        return;
    }

    // Two-column layout: left for efficiency, right for load profile
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left column: Efficiency metrics
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(chunks[0]);

    if let Some(ref eff) = app.detail_efficiency {
        let buf_hit_pct = eff.buffer_hit_percent as u16;
        let cpu_eff_pct = eff.cpu_efficiency_percent as u16;
        let soft_parse_pct = eff.soft_parse_rate_percent as u16;
        let exec_eff_pct = eff.execution_efficiency_percent as u16;

        gauge::render_metric_gauge(
            frame,
            left_chunks[0],
            "Buffer Hit %",
            buf_hit_pct,
            gauge_style(buf_hit_pct),
        );
        gauge::render_metric_gauge(
            frame,
            left_chunks[1],
            "CPU Efficiency %",
            cpu_eff_pct,
            gauge_style(cpu_eff_pct),
        );
        gauge::render_metric_gauge(
            frame,
            left_chunks[2],
            "Soft Parse %",
            soft_parse_pct,
            gauge_style(soft_parse_pct),
        );
        gauge::render_metric_gauge(
            frame,
            left_chunks[3],
            "Execution Efficiency %",
            exec_eff_pct,
            gauge_style(exec_eff_pct),
        );
    } else {
        // Use dashboard metrics fallback
        let placeholder = Paragraph::new("No efficiency metrics available for this selection.")
            .style(Theme::dim());
        frame.render_widget(placeholder, left_chunks[0]);
    }

    // Right column: Load profile items rendered in sub-chunks
    if let Some(ref lp) = app.detail_load_profile {
        let items = vec![
            format!("DB Time/sec:      {:.2} ms", lp.db_time_per_sec),
            format!("CPU Time/sec:     {:.2} ms", lp.cpu_time_per_sec),
            format!("IO Requests/sec:  {:.2}", lp.io_requests_per_sec),
            format!("Total Xacts:      {}", lp.total_transactions),
            format!("Commits/sec:      {:.2}", lp.commits_per_sec),
            format!("Rollbacks/sec:    {:.2}", lp.rollbacks_per_sec),
        ];

        let text = items.join("\n");
        let paragraph = Paragraph::new(text)
            .block(Block::bordered().title(" Load Profile "))
            .style(Theme::header());
        frame.render_widget(paragraph, chunks[1]);
    } else {
        let placeholder = Paragraph::new("No load profile data available for this selection.")
            .style(Theme::dim())
            .block(Block::bordered().title(" Load Profile "));
        frame.render_widget(placeholder, chunks[1]);
    }
}

fn gauge_style(pct: u16) -> ratatui::style::Style {
    if pct >= 90 {
        Theme::gauge_good()
    } else if pct >= 70 {
        Theme::gauge_warn()
    } else {
        Theme::gauge_bad()
    }
}
