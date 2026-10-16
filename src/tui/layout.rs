use ratatui::layout::{Constraint, Layout, Rect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Small,
    Narrow,
    Medium,
    Wide,
}

pub struct Regions {
    pub mode: Mode,
    pub header: Rect,
    pub search: Rect,
    pub body: Rect,
    pub status: Rect,
    pub footer: Rect,
    pub sources: Option<Rect>,
    pub results: Rect,
    pub inspector: Option<Rect>,
}

pub fn regions(area: Rect) -> Regions {
    let mode = if area.width < 60 || area.height < 16 {
        Mode::Small
    } else if area.width < 80 {
        Mode::Narrow
    } else if area.width < 120 {
        Mode::Medium
    } else {
        Mode::Wide
    };
    let [header, search, body, status, footer] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);
    let (sources, results, inspector) = match mode {
        Mode::Wide => {
            let [sources, results, inspector] = Layout::horizontal([
                Constraint::Length(24),
                Constraint::Fill(3),
                Constraint::Fill(2),
            ])
            .areas(body);
            (Some(sources), results, Some(inspector))
        }
        Mode::Medium => {
            let [sources, results] =
                Layout::horizontal([Constraint::Length(20), Constraint::Min(0)]).areas(body);
            (Some(sources), results, None)
        }
        _ => (None, body, None),
    };
    Regions {
        mode,
        header,
        search,
        body,
        status,
        footer,
        sources,
        results,
        inspector,
    }
}

pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakpoints_and_minimum_height() {
        for (width, height, mode) in [
            (59, 24, Mode::Small),
            (60, 16, Mode::Narrow),
            (79, 24, Mode::Narrow),
            (80, 24, Mode::Medium),
            (119, 24, Mode::Medium),
            (120, 24, Mode::Wide),
            (160, 15, Mode::Small),
        ] {
            assert_eq!(regions(Rect::new(0, 0, width, height)).mode, mode);
        }
    }

    #[test]
    fn rectangles_stay_within_resized_area_including_zero() {
        for width in [0, 1, 20, 59, 60, 79, 80, 119, 120, 180] {
            for height in [0, 1, 5, 15, 16, 24, 50] {
                let area = Rect::new(3, 7, width, height);
                let r = regions(area);
                for child in [
                    r.header,
                    r.search,
                    r.body,
                    r.status,
                    r.footer,
                    r.results,
                    r.sources.unwrap_or(r.body),
                    r.inspector.unwrap_or(r.body),
                    centered(area, 70, 20),
                ] {
                    assert!(child.x >= area.x && child.y >= area.y);
                    assert!(child.right() <= area.right() && child.bottom() <= area.bottom());
                }
            }
        }
    }
}
