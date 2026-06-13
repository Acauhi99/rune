use ratatui::style::{Color, Modifier, Style};

pub const BG: Color = Color::Rgb(20, 20, 28);
pub const SURFACE: Color = Color::Rgb(30, 30, 42);

pub const TEXT: Color = Color::Rgb(220, 220, 235);
pub const TEXT_DIM: Color = Color::Rgb(140, 140, 158);

pub const BORDER_FOCUSED: Color = Color::Rgb(130, 130, 230);
pub const BORDER_UNFOCUSED: Color = Color::Rgb(65, 65, 78);

pub const GREEN: Color = Color::Rgb(80, 210, 120);
pub const RED: Color = Color::Rgb(240, 85, 85);
pub const YELLOW: Color = Color::Rgb(245, 210, 80);
pub const CYAN: Color = Color::Rgb(90, 210, 240);
pub const MAGENTA: Color = Color::Rgb(210, 130, 210);
pub const GRAY: Color = Color::Rgb(110, 110, 130);

pub const SELECTION_BG: Color = Color::Rgb(55, 55, 75);

pub const STATUS_BG: Color = Color::Rgb(24, 24, 34);
pub const STATUS_TEXT: Color = Color::Rgb(170, 170, 190);

pub const DIFF_HEADER: Color = Color::Rgb(90, 90, 180);

pub fn focused_border() -> Style {
    Style::default().fg(BORDER_FOCUSED)
}

pub fn unfocused_border() -> Style {
    Style::default().fg(BORDER_UNFOCUSED)
}

pub fn selected_item() -> Style {
    Style::default()
        .fg(TEXT)
        .bg(SELECTION_BG)
        .add_modifier(Modifier::BOLD)
}
