mod app;
mod config;
mod ui;

use std::io::{self, Write};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

fn main() -> io::Result<()> {
    // tui_start_init: load (or create) ~/.config/confcosmos/confcosmos.toml
    let (cfg, config_path, needs_setup) = config::Config::load()?;
    let home = std::env::var("HOME").unwrap_or_default();

    let mut terminal = ratatui::init();
    // Make the terminal cursor white so it stays visible on the colored
    // input boxes (restored again before leaving).
    set_cursor_color_white();
    let mut app = app::App::new(cfg, config_path, home, needs_setup);

    let result = run(&mut terminal, &mut app);

    restore_cursor_color();
    ratatui::restore();
    result
}

/// OSC 12: set the terminal cursor color to white (xterm compatible).
fn set_cursor_color_white() {
    let _ = write!(io::stdout(), "\x1b]12;#FFFFFF\x07");
    let _ = io::stdout().flush();
}

/// OSC 112: reset the terminal cursor color to its default.
fn restore_cursor_color() {
    let _ = write!(io::stdout(), "\x1b]112\x07");
    let _ = io::stdout().flush();
}

fn run<B: ratatui::backend::Backend>(
    terminal: &mut ratatui::Terminal<B>,
    app: &mut app::App,
) -> io::Result<()> {
    while !app.quit {
        terminal.draw(|frame| ui::draw(frame, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                // Always allow Ctrl+C to quit
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    app.quit = true;
                } else {
                    app::handle(app, key);
                }
            }
        }
    }
    Ok(())
}
