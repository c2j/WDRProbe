use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Gauge, LineGauge};
use ratatui::Frame;

pub fn render_metric_gauge(frame: &mut Frame, area: Rect, label: &str, percent: u16, style: Style) {
    let gauge = Gauge::default()
        .block(Block::bordered().title(label))
        .gauge_style(style)
        .percent(percent.min(100));
    frame.render_widget(gauge, area);
}

pub fn render_line_gauge(frame: &mut Frame, area: Rect, label: &str, ratio: f64, style: Style) {
    let gauge = LineGauge::default()
        .label(label)
        .ratio(ratio.clamp(0.0, 1.0))
        .filled_style(style);
    frame.render_widget(gauge, area);
}
