use ratatui::style::{Color, Modifier, Style};

pub fn highlight_default() -> Style {
    Style::default()
        .bg(Color::Rgb(80, 84, 92))
        .add_modifier(Modifier::BOLD)
}

pub fn highlight_non_focused() -> Style {
    Style::default()
}

pub fn highlight_with_focused(focused: bool) -> Style {
    if focused {
        highlight_default()
    } else {
        highlight_non_focused()
    }
}
