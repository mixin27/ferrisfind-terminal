use std::io;
use std::time::Duration;

use crossterm::event::{self, Event};
use ratatui::{Terminal, backend::CrosstermBackend};

use super::{Action, AppState, Focus};
use crate::{
    domain::Record,
    error::AppError,
    tui::{input, interrupt::Interrupt, terminal, theme::Theme, view},
};

pub fn run(records: Vec<Record>) -> Result<(), AppError> {
    terminal::install_panic_hook();
    // Register before entering raw mode and unregister after terminal restoration.
    let interrupt = Interrupt::new().map_err(AppError::Terminal)?;
    terminal::with_terminal(|| {
        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        terminal.clear()?;
        let mut app = AppState::new(records);
        let theme = Theme::from_env();
        while !app.quit && !interrupt.requested() {
            terminal.draw(|frame| {
                app.reduce(Action::ScrollLimit(view::scroll_limit(&app, frame.area())));
                view::draw(frame, &app, theme);
            })?;
            // Poll only to observe termination signals. Idle polling never redraws.
            while !event::poll(Duration::from_millis(100))? {
                if interrupt.requested() {
                    return Ok(());
                }
            }
            if interrupt.requested() {
                return Ok(());
            }
            match event::read()? {
                Event::Key(key) => {
                    if let Some(action) = input::action(key, app.focus) {
                        app.reduce(action);
                    }
                }
                Event::Paste(text) if app.focus == Focus::Search => {
                    app.reduce(Action::Insert(text))
                }
                Event::Resize(_, _) => {} // Terminal::draw re-queries dimensions.
                _ => {}
            }
        }
        Ok(())
    })
}
