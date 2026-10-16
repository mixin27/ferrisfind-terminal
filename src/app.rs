use unicode_segmentation::UnicodeSegmentation;

use crate::domain::Record;

mod event_loop;
pub use event_loop::run;

pub const QUERY_LIMIT: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Search,
    Sources,
    Results,
    Inspector,
    Help,
}

#[derive(Debug)]
pub enum Action {
    Quit,
    Search,
    Help,
    NextFocus,
    PreviousFocus,
    Escape,
    Enter,
    Up,
    Down,
    Home,
    End,
    Left,
    Right,
    Backspace,
    Delete,
    Insert(String),
    Clear,
    Refresh,
    ScrollLimit(u16),
}

pub struct AppState {
    pub records: Vec<Record>,
    pub selected: Option<usize>,
    pub focus: Focus,
    pub query: String,
    /// Byte offset, always on a grapheme boundary.
    pub cursor: usize,
    pub inspector_scroll: u16,
    pub max_scroll: u16,
    pub status: &'static str,
    pub quit: bool,
    previous_focus: Focus,
}

impl AppState {
    pub fn new(records: Vec<Record>) -> Self {
        let mut state = Self {
            records: Vec::new(),
            selected: None,
            focus: Focus::Results,
            query: String::new(),
            cursor: 0,
            inspector_scroll: 0,
            max_scroll: 0,
            status: "FICTIONAL fixtures • memory only • search not connected",
            quit: false,
            previous_focus: Focus::Results,
        };
        state.replace_records(records);
        state
    }

    pub fn selected_record(&self) -> Option<&Record> {
        self.selected.and_then(|index| self.records.get(index))
    }

    pub fn replace_records(&mut self, records: Vec<Record>) {
        self.records = records;
        self.selected = if self.records.is_empty() {
            None
        } else {
            Some(self.selected.unwrap_or(0).min(self.records.len() - 1))
        };
        self.inspector_scroll = 0;
        self.max_scroll = 0;
    }

    fn previous_boundary(&self) -> usize {
        self.query
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .take_while(|&i| i < self.cursor)
            .last()
            .unwrap_or(0)
    }

    fn next_boundary(&self) -> usize {
        self.query
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .find(|&i| i > self.cursor)
            .unwrap_or(self.query.len())
    }

    fn clamp_cursor(&mut self) {
        self.cursor = self
            .query
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .find(|&i| i >= self.cursor)
            .unwrap_or(self.query.len());
    }

    pub fn reduce(&mut self, action: Action) {
        use Action::*;
        match action {
            Quit => self.quit = true,
            ScrollLimit(limit) => {
                self.max_scroll = limit;
                self.inspector_scroll = self.inspector_scroll.min(limit);
            }
            Help => {
                if self.focus == Focus::Help {
                    self.focus = self.previous_focus;
                } else {
                    self.previous_focus = self.focus;
                    self.focus = Focus::Help;
                }
            }
            Escape | Enter if self.focus == Focus::Help => self.focus = self.previous_focus,
            _ if self.focus == Focus::Help => {}
            Search => self.focus = Focus::Search,
            Clear => {
                self.query.clear();
                self.cursor = 0;
            }
            Refresh => self.status = "FICTIONAL source count refreshed from memory",
            NextFocus | PreviousFocus => {
                let order = [
                    Focus::Search,
                    Focus::Sources,
                    Focus::Results,
                    Focus::Inspector,
                ];
                let index = order
                    .iter()
                    .position(|&focus| focus == self.focus)
                    .unwrap_or(0);
                let step = if matches!(action, NextFocus) {
                    1
                } else {
                    order.len() - 1
                };
                self.focus = order[(index + step) % order.len()];
            }
            Escape => self.focus = Focus::Results,
            Enter => match self.focus {
                Focus::Search => {
                    self.status =
                        "Search is not connected in Phase 1; fictional records are unchanged";
                    self.focus = Focus::Results;
                }
                Focus::Results if self.selected.is_some() => self.focus = Focus::Inspector,
                Focus::Inspector | Focus::Sources => self.focus = Focus::Results,
                _ => {}
            },
            Insert(text) if self.focus == Focus::Search => {
                let room = QUERY_LIMIT.saturating_sub(self.query.chars().count());
                let text: String = text
                    .chars()
                    .filter(|ch| !ch.is_control())
                    .take(room)
                    .collect();
                self.query.insert_str(self.cursor, &text);
                self.cursor += text.len();
                self.clamp_cursor();
                if self.query.chars().count() == QUERY_LIMIT {
                    self.status = "Search field limit: 256 characters";
                }
            }
            Left if self.focus == Focus::Search => self.cursor = self.previous_boundary(),
            Right if self.focus == Focus::Search => self.cursor = self.next_boundary(),
            Home if self.focus == Focus::Search => self.cursor = 0,
            End if self.focus == Focus::Search => self.cursor = self.query.len(),
            Backspace if self.focus == Focus::Search => {
                let start = self.previous_boundary();
                self.query.replace_range(start..self.cursor, "");
                self.cursor = start;
                self.clamp_cursor();
            }
            Delete if self.focus == Focus::Search => {
                self.query
                    .replace_range(self.cursor..self.next_boundary(), "");
                self.clamp_cursor();
            }
            Up | Down | Home | End if self.focus == Focus::Inspector => {
                self.inspector_scroll = match action {
                    Up => self.inspector_scroll.saturating_sub(1),
                    Down => self.inspector_scroll.saturating_add(1).min(self.max_scroll),
                    Home => 0,
                    _ => self.max_scroll,
                };
            }
            Up | Down | Home | End if self.focus == Focus::Results => {
                if let Some(index) = self.selected {
                    self.selected = Some(match action {
                        Up => index.saturating_sub(1),
                        Down => index.saturating_add(1).min(self.records.len() - 1),
                        Home => 0,
                        _ => self.records.len() - 1,
                    });
                    self.inspector_scroll = 0;
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::fictional_records;

    #[test]
    fn selection_clamps_after_replacement_and_empty_navigation() {
        let mut app = AppState::new(fictional_records());
        app.reduce(Action::End);
        assert_eq!(app.selected, Some(23));
        app.reduce(Action::Down);
        assert_eq!(app.selected, Some(23));
        app.replace_records(fictional_records()[..2].to_vec());
        assert_eq!(app.selected, Some(1));
        app.replace_records(vec![]);
        for action in [
            Action::Up,
            Action::Down,
            Action::Home,
            Action::End,
            Action::Enter,
        ] {
            app.reduce(action);
        }
        assert_eq!(app.selected, None);
        assert_eq!(app.focus, Focus::Results);
        app.replace_records(fictional_records());
        assert_eq!(app.selected, Some(0));
    }

    #[test]
    fn focus_cycle_and_modal_return() {
        let mut app = AppState::new(fictional_records());
        for focus in [
            Focus::Inspector,
            Focus::Search,
            Focus::Sources,
            Focus::Results,
        ] {
            app.reduce(Action::NextFocus);
            assert_eq!(app.focus, focus);
        }
        app.reduce(Action::PreviousFocus);
        assert_eq!(app.focus, Focus::Sources);
        app.reduce(Action::Search);
        app.reduce(Action::Help);
        app.reduce(Action::Insert("ignored".into()));
        app.reduce(Action::NextFocus);
        assert_eq!(app.focus, Focus::Help);
        assert!(app.query.is_empty());
        app.reduce(Action::Escape);
        assert_eq!(app.focus, Focus::Search);
        app.reduce(Action::Enter);
        assert_eq!(app.focus, Focus::Results);
        assert_eq!(app.records.len(), 24);
        assert!(app.status.contains("not connected"));
        app.reduce(Action::Enter);
        assert_eq!(app.focus, Focus::Inspector);
        app.reduce(Action::Escape);
        assert_eq!(app.focus, Focus::Results);
    }

    #[test]
    fn editing_uses_graphemes_and_bounds_pasted_input() {
        let mut app = AppState::new(vec![]);
        app.reduce(Action::Search);
        app.reduce(Action::Insert("q cafe\u{301}🦀".into()));
        app.reduce(Action::Left);
        app.reduce(Action::Backspace);
        assert_eq!(app.query, "q caf🦀");
        app.reduce(Action::Delete);
        assert_eq!(app.query, "q caf");
        app.reduce(Action::Home);
        app.reduce(Action::Insert("မြန်မာ".into()));
        assert!(app.query.is_char_boundary(app.cursor));
        app.reduce(Action::Clear);
        app.reduce(Action::Insert(format!("\n\x1b{}", "界".repeat(300))));
        assert_eq!(app.query.chars().count(), QUERY_LIMIT);
        assert!(!app.query.chars().any(char::is_control));
        assert!(!app.quit);
    }

    #[test]
    fn inspector_scroll_is_bounded_and_reclamped_on_resize() {
        let mut app = AppState::new(fictional_records());
        app.reduce(Action::Enter);
        app.reduce(Action::ScrollLimit(10));
        app.reduce(Action::End);
        app.reduce(Action::Down);
        assert_eq!(app.inspector_scroll, 10);
        app.reduce(Action::ScrollLimit(2));
        assert_eq!(app.inspector_scroll, 2);
        app.reduce(Action::Home);
        app.reduce(Action::Up);
        assert_eq!(app.inspector_scroll, 0);
    }
}
