use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    pub fn title_bar() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }

    pub fn status_bar() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    pub fn selected() -> Style {
        Style::default().add_modifier(Modifier::REVERSED)
    }

    pub fn header() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn critical() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn warning() -> Style {
        Style::default().fg(Color::Yellow)
    }

    pub fn info() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn good() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn bad() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn border() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    pub fn gauge_normal() -> Style {
        Style::default().fg(Color::Blue)
    }

    pub fn gauge_good() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn gauge_warn() -> Style {
        Style::default().fg(Color::Yellow)
    }

    pub fn gauge_bad() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn highlight() -> Style {
        Style::default().fg(Color::Cyan)
    }

    pub fn dim() -> Style {
        Style::default().fg(Color::DarkGray)
    }
}
