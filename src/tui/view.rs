use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::{
    input::HELP,
    layout::{self, Mode},
    theme::Theme,
};
use crate::{
    app::{AppState, Focus},
    domain::SOURCE,
};

fn panel(title: &str, focused: bool, theme: Theme) -> Block<'_> {
    Block::default()
        .title(format!("{} {title} ", if focused { ">" } else { " " }))
        .borders(Borders::ALL)
        .border_type(if focused {
            BorderType::Thick
        } else {
            BorderType::Plain
        })
        .border_style(Style::default().fg(if focused { theme.cyan } else { theme.border }))
        .style(theme.base().bg(theme.surface))
}

/// Remove terminal controls before handing any display text to widgets.
fn safe(text: &str) -> String {
    text.chars()
        .filter(|ch| *ch == '\n' || !ch.is_control())
        .collect()
}

fn inspector_text(app: &AppState) -> String {
    match app.selected_record() {
        Some(record) => safe(&format!(
            "{}\n{}\n\nID: {}\nSource: {}\nCategory: {}\n\n{}",
            record.title,
            "FICTIONAL • display fixture",
            record.id,
            SOURCE,
            record.category,
            record.content
        )),
        None => "No record selected.\nOnly fictional in-memory fixtures are available.".into(),
    }
}

pub fn scroll_limit(app: &AppState, area: Rect) -> u16 {
    let regions = layout::regions(area);
    if regions.mode == Mode::Small {
        return 0;
    }
    let area = regions.inspector.unwrap_or(regions.body);
    let width = area.width.saturating_sub(2);
    let height = area.height.saturating_sub(2);
    let count = Paragraph::new(inspector_text(app))
        .wrap(Wrap { trim: false })
        .line_count(width);
    count
        .saturating_sub(usize::from(height))
        .min(usize::from(u16::MAX)) as u16
}

/// Return a grapheme-aligned suffix and cursor column; reserve one cursor cell.
fn search_window(query: &str, cursor: usize, width: u16) -> (&str, u16) {
    let available = usize::from(width.saturating_sub(1));
    let mut start = 0;
    for (index, _) in query[..cursor].grapheme_indices(true) {
        if query[index..cursor].width() <= available {
            break;
        }
        start = index + query[index..].graphemes(true).next().map_or(0, str::len);
    }
    (
        &query[start..],
        query[start..cursor].width().min(available) as u16,
    )
}

pub fn draw(frame: &mut Frame, app: &AppState, theme: Theme) {
    let area = frame.area();
    frame.render_widget(Block::default().style(theme.base()), area);
    let r = layout::regions(area);
    if r.mode == Mode::Small {
        frame.render_widget(Paragraph::new("Resize to at least 60 x 16\nCtrl+C quits • Esc then q quits\nFICTIONAL fixtures • no database")
            .style(theme.base()).wrap(Wrap { trim: false }), area);
        return;
    }

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    " FERRISFIND ",
                    Style::default().fg(theme.cyan).add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("v{}  /  LOCAL WORKSHOP", env!("CARGO_PKG_VERSION"))),
            ]),
            Line::styled(
                format!(
                    " FICTIONAL DATA  •  {} records  •  memory only",
                    app.records.len()
                ),
                Style::default().fg(theme.violet),
            ),
        ]),
        r.header,
    );

    let search = panel(
        "SEARCH • Enter: preview only",
        app.focus == Focus::Search,
        theme,
    );
    let inner = search.inner(r.search);
    frame.render_widget(search, r.search);
    let (text, column) = search_window(&app.query, app.cursor, inner.width);
    let hint = if app.query.is_empty() && app.focus != Focus::Search {
        "/ to type • search is not connected"
    } else {
        text
    };
    frame.render_widget(Paragraph::new(hint).style(theme.base()), inner);
    if app.focus == Focus::Search && inner.width > 0 && inner.height > 0 {
        frame.set_cursor_position((inner.x + column, inner.y));
    }

    if let Some(area) = r.sources {
        sources(frame, app, theme, area);
    }
    results(frame, app, theme, r.results);
    if let Some(area) = r.inspector {
        inspector(frame, app, theme, area);
    }
    if r.mode == Mode::Narrow && app.focus == Focus::Sources {
        frame.render_widget(Clear, r.body);
        sources(frame, app, theme, r.body);
    }
    if r.mode != Mode::Wide && app.focus == Focus::Inspector {
        frame.render_widget(Clear, r.body);
        inspector(frame, app, theme, r.body);
    }

    frame.render_widget(
        Paragraph::new(app.status).style(Style::default().fg(theme.muted)),
        r.status,
    );
    let footer = if app.focus == Focus::Search {
        " Esc: results | Enter: preview | Tab: focus | F1: help | ^C: quit"
    } else {
        " /: search | j/k: move | Enter: inspect | F1: help | q: quit"
    };
    frame.render_widget(
        Paragraph::new(footer).style(Style::default().fg(theme.cyan)),
        r.footer,
    );

    if app.focus == Focus::Help {
        let popup = layout::centered(area, 76, 16);
        frame.render_widget(Clear, popup);
        frame.render_widget(
            Paragraph::new(HELP)
                .block(panel("HELP • F1 / Esc / Enter closes", true, theme))
                .wrap(Wrap { trim: false }),
            popup,
        );
    }
}

fn sources(frame: &mut Frame, app: &AppState, theme: Theme, area: Rect) {
    frame.render_widget(Paragraph::new(format!(
        "> {SOURCE}\n  {} records\n\nFICTIONAL\nMemory only\n\nNo database\nNo imports\n\nTab: next focus\nEnter/Esc: results", app.records.len()
    )).block(panel("SOURCES", app.focus == Focus::Sources, theme)).wrap(Wrap { trim: false }), area);
}

fn results(frame: &mut Frame, app: &AppState, theme: Theme, area: Rect) {
    let title = format!("RECORDS • {} fictional", app.records.len());
    let block = panel(&title, app.focus == Focus::Results, theme);
    if app.records.is_empty() {
        frame.render_widget(
            Paragraph::new("No fictional records available.").block(block),
            area,
        );
        return;
    }
    let items: Vec<_> = app
        .records
        .iter()
        .enumerate()
        .map(|(index, record)| {
            ListItem::new(vec![
                Line::from(format!("{:02}  {}", index + 1, safe(record.title))),
                Line::styled(
                    format!(
                        "    [{}] {}",
                        safe(record.category),
                        safe(record.content.lines().next().unwrap_or_default())
                    ),
                    Style::default().fg(theme.muted),
                ),
            ])
        })
        .collect();
    let list = List::new(items)
        .block(block)
        .highlight_symbol("> ")
        .highlight_style(
            Style::default()
                .fg(theme.cyan)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
        );
    // View-only scrolling state: the reducer remains the sole selection owner.
    let mut state = ListState::default().with_selected(app.selected);
    frame.render_stateful_widget(list, area, &mut state);
}

fn inspector(frame: &mut Frame, app: &AppState, theme: Theme, area: Rect) {
    let title = format!("INSPECTOR • line {}", app.inspector_scroll + 1);
    frame.render_widget(
        Paragraph::new(inspector_text(app))
            .block(panel(&title, app.focus == Focus::Inspector, theme))
            .wrap(Wrap { trim: false })
            .scroll((app.inspector_scroll, 0)),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{app::Action, domain::fictional_records};
    use ratatui::{Terminal, backend::TestBackend};

    fn render(app: &AppState, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| draw(frame, app, Theme::new(true, false)))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    #[test]
    fn all_focus_states_render_at_every_breakpoint_and_tiny_sizes() {
        let mut app = AppState::new(fictional_records());
        for (width, height) in [
            (0, 0),
            (1, 1),
            (40, 10),
            (59, 24),
            (60, 16),
            (79, 24),
            (80, 24),
            (119, 24),
            (120, 30),
            (180, 40),
        ] {
            for focus in [
                Focus::Search,
                Focus::Sources,
                Focus::Results,
                Focus::Inspector,
                Focus::Help,
            ] {
                app.focus = focus;
                let text = render(&app, width, height);
                if width >= 60 && height >= 16 {
                    assert!(text.contains("FICTIONAL"));
                }
            }
        }
    }

    #[test]
    fn selection_scrolls_into_view_and_empty_state_renders() {
        let mut app = AppState::new(fictional_records());
        app.reduce(Action::End);
        assert!(render(&app, 80, 24).contains("End of the map"));
        app.replace_records(vec![]);
        assert!(render(&app, 80, 24).contains("No fictional records"));
    }

    #[test]
    fn inspector_wrap_scroll_limit_and_sanitization() {
        let app = AppState::new(fictional_records());
        assert!(scroll_limit(&app, Rect::new(0, 0, 120, 24)) > 0);
        assert_eq!(scroll_limit(&app, Rect::new(0, 0, 10, 10)), 0);
        assert_eq!(safe("a\x1b[31m\0b\n"), "a[31mb\n");
    }

    #[test]
    fn help_fits_minimum_supported_size_and_narrow_views_are_accessible() {
        let mut app = AppState::new(fictional_records());
        app.reduce(Action::Help);
        let text = render(&app, 60, 16);
        assert!(text.contains("Ctrl+C: always quit"));
        assert!(text.contains("Submitting leaves the fictional records unchanged."));
        app.reduce(Action::Help);
        app.reduce(Action::Enter);
        assert!(render(&app, 60, 16).contains("INSPECTOR"));
        app.reduce(Action::PreviousFocus);
        app.reduce(Action::PreviousFocus);
        assert!(render(&app, 60, 16).contains("SOURCES"));
    }

    #[test]
    fn search_cursor_uses_display_width_and_keeps_graphemes_intact() {
        let query = "a界e\u{301}🦀";
        assert_eq!(search_window(query, query.len(), 4), ("e\u{301}🦀", 3));
        for cursor in query
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain([query.len()])
        {
            for width in 1..12 {
                let (text, column) = search_window(query, cursor, width);
                assert!(column < width);
                assert!(query.ends_with(text));
            }
        }
    }
}
