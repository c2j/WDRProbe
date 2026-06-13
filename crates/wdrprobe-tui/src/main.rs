mod app;
mod components;
mod theme;
mod ui;

use clap::Parser;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;

#[derive(Parser)]
#[command(name = "wdrprobe-tui", about = "WDRProbe TUI — Interactive terminal browser")]
struct Cli {
    /// Path to the SQLite database
    #[arg(long, default_value = "./wdrprobe.db")]
    db: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Install panic hook to restore terminal on crash
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
        default_hook(info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let result = app::run_app(&mut terminal, &cli.db);

    // Restore terminal (MUST happen even on error)
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result?;
    Ok(())
}
