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
    use crossterm::event::KeyModifiers;
    use std::time::Duration;

    loop {
        // Poll the background loader for new data
        app.bd_poll_loader();

        terminal.draw(|f| ui::draw(f, app))?;

        // Use poll with timeout so we can keep polling the loader
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Handle graph tree search mode input
                    if app.search_mode {
                        match key.code {
                            KeyCode::Esc => app.exit_search_mode(),
                            KeyCode::Enter => {
                                // Exit search mode but keep the filtered view
                                app.search_mode = false;
                            }
                            KeyCode::Backspace => app.search_backspace(),
                            KeyCode::Char('i') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.search_toggle_case();
                            }
                            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.search_next();
                            }
                            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.search_prev();
                            }
                            KeyCode::Down => app.search_next(),
                            KeyCode::Up => app.search_prev(),
                            KeyCode::Char(c) => app.search_input(c),
                            _ => {}
                        }
                    }
                    // Handle business data search mode input
                    else if app.bd_search_mode {
                        match key.code {
                            KeyCode::Esc => app.bd_exit_search_mode(),
                            KeyCode::Enter => {
                                // Exit search mode but keep filter
                                app.bd_search_mode = false;
                            }
                            KeyCode::Backspace => app.bd_search_backspace(),
                            KeyCode::Char('i') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.bd_search_toggle_case();
                            }
                            KeyCode::Down => app.on_down(),
                            KeyCode::Up => app.on_up(),
                            KeyCode::Char(c) => app.bd_search_input(c),
                            _ => {}
                        }
                    } else {
                        // Normal mode
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('/') => {
                                // Enter search mode in tree view or business data resource list
                                if app.current_tab == app::Tab::Graphs
                                    && app.graphs_view == app::GraphsView::Tree
                                {
                                    app.enter_search_mode();
                                } else if app.current_tab == app::Tab::BusinessData
                                    && app.bd_view == app::BusinessDataView::ResourceList
                                {
                                    app.bd_enter_search_mode();
                                }
                            }
                            KeyCode::Char('s') => {
                                // Toggle data source in Business Data tab
                                if app.current_tab == app::Tab::BusinessData {
                                    app.bd_toggle_source();
                                }
                            }
                            KeyCode::Tab => app.next_tab(),
                            KeyCode::BackTab => app.prev_tab(),
                            KeyCode::Down | KeyCode::Char('j') => app.on_down(),
                            KeyCode::Up | KeyCode::Char('k') => app.on_up(),
                            KeyCode::Right | KeyCode::Char('l') => app.on_right(),
                            KeyCode::Left | KeyCode::Char('h') => app.on_left(),
                            KeyCode::Enter => app.on_enter(),
                            KeyCode::Esc => app.on_escape(),
                            KeyCode::Char('n') => app.search_next(),
                            KeyCode::Char('N') => app.search_prev(),
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
