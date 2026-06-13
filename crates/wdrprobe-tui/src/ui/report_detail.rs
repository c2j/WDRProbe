use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::components::gauge;
use crate::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let report = match &app.detail_report {
        Some(r) => r,
        None => {
            let msg = Paragraph::new("Select a report from the Reports page first.")
                .style(Theme::dim())
                .block(Block::bordered().title(" Report Detail "));
            frame.render_widget(msg, area);
            return;
        }
    };

    // Vertical split: metadata (3) + efficiency (6) + load profile (5) + top sqls (rest)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .split(area);

    // Metadata block
    let meta_text = format!(
        " Instance: {}  |  Snap: {} → {}  |  Status: {}  |  Generated: {}",
        report.instance_name,
        truncate_datetime(&report.snapshot_start),
        truncate_datetime(&report.snapshot_end),
        report.status,
        truncate_datetime(&report.generation_time),
    );
    let meta = Paragraph::new(meta_text)
        .block(Block::bordered().title(" Report Metadata "))
        .style(Theme::info());
    frame.render_widget(meta, chunks[0]);

    // Efficiency metrics (2 rows x 2 cols = 4 gauges)
    let eff_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let left_eff = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .split(eff_chunks[0]);
    let right_eff = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .split(eff_chunks[1]);

    if let Some(ref eff) = app.detail_efficiency {
        gauge::render_metric_gauge(
            frame,
            left_eff[0],
            " Buffer Hit % ",
            eff.buffer_hit_percent as u16,
            gauge_style(eff.buffer_hit_percent),
        );
        gauge::render_metric_gauge(
            frame,
            left_eff[1],
            " CPU Efficiency % ",
            eff.cpu_efficiency_percent as u16,
            gauge_style(eff.cpu_efficiency_percent),
        );
        gauge::render_metric_gauge(
            frame,
            right_eff[0],
            " Soft Parse % ",
            eff.soft_parse_rate_percent as u16,
            gauge_style(eff.soft_parse_rate_percent),
        );
        gauge::render_metric_gauge(
            frame,
            right_eff[1],
            " Execution Efficiency % ",
            eff.execution_efficiency_percent as u16,
            gauge_style(eff.execution_efficiency_percent),
        );
    } else {
        let no_eff = Paragraph::new("No efficiency metrics")
            .style(Theme::dim())
            .block(Block::bordered().title(" Efficiency Metrics "));
        frame.render_widget(no_eff, chunks[1]);
    }

    // Load profile
    let load_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    if let Some(ref lp) = app.detail_load_profile {
        let left_lp = format!(
            "DB Time/sec:     {:.2} ms\nCPU Time/sec:    {:.2} ms\nIO Req/sec:      {:.2}",
            lp.db_time_per_sec, lp.cpu_time_per_sec, lp.io_requests_per_sec
        );
        let right_lp = format!(
            "Total Xacts:     {}\nCommits/sec:     {:.2}\nRollbacks/sec:   {:.2}",
            lp.total_transactions, lp.commits_per_sec, lp.rollbacks_per_sec
        );

        let left_p = Paragraph::new(left_lp)
            .block(Block::bordered().title(" Load Profile (Left) "))
            .style(Theme::info());
        let right_p = Paragraph::new(right_lp)
            .block(Block::bordered().title(" Load Profile (Right) "))
            .style(Theme::info());

        frame.render_widget(left_p, load_chunks[0]);
        frame.render_widget(right_p, load_chunks[1]);
    } else {
        let no_lp = Paragraph::new("No load profile data")
            .style(Theme::dim())
            .block(Block::bordered().title(" Load Profile "));
        frame.render_widget(no_lp, chunks[2]);
    }

    // Top SQLs (scrollable)
    let sqls = &app.detail_top_sqls;
    if sqls.is_empty() {
        let no_sqls = Paragraph::new("No Top SQL data for this report.")
            .style(Theme::dim())
            .block(Block::bordered().title(" Top SQLs "));
        frame.render_widget(no_sqls, chunks[3]);
        return;
    }

    let sql_text: Vec<String> = sqls
        .iter()
        .map(|s| {
            let truncated = if s.sql_text.len() > 60 {
                format!("{}...", &s.sql_text[..57])
            } else {
                s.sql_text.clone()
            };
            let rank = s.rank_by_time.unwrap_or(0);
            format!(
                " #{:2} | SQL_ID: {:8} | Elapsed: {:>8.1} | CPU: {:>8.1} | Execs: {:>6} | {}",
                rank,
                s.sql_id.as_deref().unwrap_or("N/A"),
                s.total_elapsed_time,
                s.cpu_time,
                s.executions,
                truncated,
            )
        })
        .collect();

    let display_text = sql_text.join("\n");
    let sql_para = Paragraph::new(display_text)
        .block(Block::bordered().title(format!(
            " Top SQLs ({} items, j/k scroll) ",
            sqls.len()
        )))
        .style(Theme::info())
        .scroll((app.detail_scroll as u16, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(sql_para, chunks[3]);
}

fn gauge_style(pct: f64) -> ratatui::style::Style {
    if pct >= 90.0 {
        Theme::gauge_good()
    } else if pct >= 70.0 {
        Theme::gauge_warn()
    } else {
        Theme::gauge_bad()
    }
}

fn truncate_datetime(dt: &str) -> &str {
    if dt.len() > 16 {
        &dt[..16]
    } else {
        dt
    }
}
