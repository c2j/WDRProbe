use ratatui::layout::Rect;
use ratatui::widgets::{Block, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    if app.reports.is_empty() {
        let msg = Paragraph::new("No WDR reports found. Import WDR reports first via the desktop app.")
            .style(Theme::dim())
            .block(Block::bordered().title(" Reports "));
        frame.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = app
        .reports
        .iter()
        .map(|r| {
            let line = format!(
                " #{}  {}  |  {} → {}  |  {} ",
                r.id,
                r.instance_name,
                truncate_datetime(&r.snapshot_start),
                truncate_datetime(&r.snapshot_end),
                r.status
            );
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::bordered().title(" Reports "))
        .highlight_style(Theme::selected());

    frame.render_stateful_widget(list, area, &mut app.reports_state);
}

fn truncate_datetime(dt: &str) -> &str {
    // Trim to "YYYY-MM-DD HH:MM" (16 chars) if longer
    if dt.len() > 16 {
        &dt[..16]
    } else {
        dt
    }
}
