//! Alizarin Explorer - TUI for exploring Arches data structures
//!
//! A Ratatui-based dashboard for inspecting prebuild directories.

mod app;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;

use app::App;

#[derive(Parser, Debug)]
#[command(name = "alizarin-explorer")]
#[command(about = "TUI explorer for Alizarin/Arches data structures")]
#[command(version)]
struct Args {
    /// Path to the prebuild directory
    #[arg(value_name = "PREBUILD_DIR")]
    prebuild_path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new(args.prebuild_path)?;
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Tab => app.next_tab(),
                    KeyCode::BackTab => app.prev_tab(),
                    KeyCode::Down | KeyCode::Char('j') => app.on_down(),
                    KeyCode::Up | KeyCode::Char('k') => app.on_up(),
                    KeyCode::Right | KeyCode::Char('l') => app.on_right(),
                    KeyCode::Left | KeyCode::Char('h') => app.on_left(),
                    KeyCode::Enter => app.on_enter(),
                    KeyCode::Esc => app.on_escape(),
                    _ => {}
                }
            }
        }
    }
}
