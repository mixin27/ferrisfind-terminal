use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::{Action, Focus};

pub const HELP: &str = "FICTIONAL fixtures | memory only | no database\n\
Tab / Shift+Tab: next / previous focus\n\
/: search | Enter: submit / inspect / close\n\
Esc: return | F1: toggle help\n\
Up/Down or j/k: browse / scroll inspector\n\
Home / End: first / last record or scroll line\n\
Left/Right: move search cursor by grapheme\n\
Backspace/Delete: edit search text\n\
Ctrl+L: clear field | Ctrl+R: refresh fixture count\n\
q: quit outside search and help | Ctrl+C: always quit\n\
Search is not connected in Phase 1.\n\
Submitting leaves the fictional records unchanged.";

pub fn action(key: KeyEvent, focus: Focus) -> Option<Action> {
    if key.kind == KeyEventKind::Release {
        return None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c' | 'C') => Some(Action::Quit),
            KeyCode::Char('l' | 'L') => Some(Action::Clear),
            KeyCode::Char('r' | 'R') => Some(Action::Refresh),
            _ => None,
        };
    }
    if key
        .modifiers
        .intersects(KeyModifiers::ALT | KeyModifiers::SUPER)
    {
        return None;
    }
    let action = match key.code {
        KeyCode::F(1) => Action::Help,
        KeyCode::Esc => Action::Escape,
        KeyCode::Enter => Action::Enter,
        _ if focus == Focus::Help => return None,
        KeyCode::BackTab => Action::PreviousFocus,
        KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => Action::PreviousFocus,
        KeyCode::Tab => Action::NextFocus,
        KeyCode::Char(ch) if focus == Focus::Search => Action::Insert(ch.to_string()),
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('/') => Action::Search,
        KeyCode::Char('k') | KeyCode::Up => Action::Up,
        KeyCode::Char('j') | KeyCode::Down => Action::Down,
        KeyCode::Home => Action::Home,
        KeyCode::End => Action::End,
        KeyCode::Left => Action::Left,
        KeyCode::Right => Action::Right,
        KeyCode::Backspace => Action::Backspace,
        KeyCode::Delete => Action::Delete,
        _ => return None,
    };
    Some(action)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{app::AppState, domain::fictional_records};

    fn press(app: &mut AppState, code: KeyCode, modifiers: KeyModifiers) {
        if let Some(action) = action(KeyEvent::new(code, modifiers), app.focus) {
            app.reduce(action);
        }
    }

    #[test]
    fn shortcuts_are_text_while_searching_and_ctrl_c_always_quits() {
        let mut app = AppState::new(fictional_records());
        press(&mut app, KeyCode::Char('/'), KeyModifiers::NONE);
        for ch in "qjk/".chars() {
            press(&mut app, KeyCode::Char(ch), KeyModifiers::NONE);
        }
        assert_eq!(app.query, "qjk/");
        assert!(!app.quit);
        press(&mut app, KeyCode::Esc, KeyModifiers::NONE);
        press(&mut app, KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(app.quit);
        for focus in [
            Focus::Search,
            Focus::Results,
            Focus::Sources,
            Focus::Inspector,
            Focus::Help,
        ] {
            let mut app = AppState::new(vec![]);
            app.focus = focus;
            press(&mut app, KeyCode::Char('c'), KeyModifiers::CONTROL);
            assert!(app.quit);
        }
    }

    #[test]
    fn help_does_not_leak_keys_and_release_events_are_ignored() {
        let mut app = AppState::new(fictional_records());
        press(&mut app, KeyCode::F(1), KeyModifiers::NONE);
        press(&mut app, KeyCode::Char('q'), KeyModifiers::NONE);
        press(&mut app, KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(!app.quit);
        assert_eq!(app.selected, Some(0));
        press(&mut app, KeyCode::F(1), KeyModifiers::NONE);
        assert_eq!(app.focus, Focus::Results);
        let key = KeyEvent::new_with_kind(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        );
        assert!(action(key, app.focus).is_none());
        press(&mut app, KeyCode::BackTab, KeyModifiers::SHIFT);
        assert_eq!(app.focus, Focus::Sources);
    }
}
