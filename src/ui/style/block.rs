use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders};

pub fn block_default() -> Block<'static> {
    let default_border = Style::default().fg(Color::Gray);
    block_base().border_style(default_border)
}

pub fn block_focused() -> Block<'static> {
    let focused_border = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    block_base().border_style(focused_border)
}

pub fn block_with_focused(focused: bool) -> Block<'static> {
    if focused {
        block_focused()
    } else {
        block_default()
    }
}

fn block_base() -> Block<'static> {
    Block::new()
        .border_type(BorderType::Plain)
        .borders(Borders::all())
}
