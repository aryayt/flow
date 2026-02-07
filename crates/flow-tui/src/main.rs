use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Determine database path from CLI args or default
    let db_path = std::env::args().nth(1).map(PathBuf::from);

    let mut app = flow_tui::app::App::with_db_path(db_path);
    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut flow_tui::app::App,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| app.render(frame))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if app.handle_key(key) {
                        return Ok(());
                    }
                }
                Event::Resize(w, h) => {
                    app.resize(w, h);
                }
                _ => {}
            }
        } else {
            // Auto-refresh on timeout (no key pressed)
            app.auto_refresh();
        }
    }
}
