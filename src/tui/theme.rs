use ratatui::style::{Color, Style};

#[derive(Clone, Copy)]
pub struct Theme {
    pub background: Color,
    pub surface: Color,
    pub border: Color,
    pub foreground: Color,
    pub muted: Color,
    pub cyan: Color,
    pub violet: Color,
}

impl Theme {
    pub fn from_env() -> Self {
        Self::new(
            std::env::var_os("NO_COLOR").is_some()
                || std::env::var("TERM").is_ok_and(|s| s == "dumb"),
            std::env::var("COLORTERM").is_ok_and(|s| s == "truecolor" || s == "24bit"),
        )
    }

    pub fn new(monochrome: bool, truecolor: bool) -> Self {
        let color = |rgb, indexed| {
            if monochrome {
                Color::Reset
            } else if truecolor {
                rgb
            } else {
                indexed
            }
        };
        Self {
            background: color(Color::Rgb(8, 14, 27), Color::Black),
            surface: color(Color::Rgb(16, 28, 48), Color::Black),
            border: color(Color::Rgb(38, 54, 83), Color::DarkGray),
            foreground: color(Color::Rgb(227, 237, 250), Color::White),
            muted: color(Color::Rgb(145, 164, 190), Color::Gray),
            cyan: color(Color::Rgb(57, 228, 237), Color::Cyan),
            violet: color(Color::Rgb(156, 139, 251), Color::Magenta),
        }
    }

    pub fn base(self) -> Style {
        Style::default().fg(self.foreground).bg(self.background)
    }
}
