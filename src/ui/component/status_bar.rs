use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};

use crate::infrastructure::util::state::State;
use crate::ui::state::{AppState, StatusLevel};

use super::Component;

pub struct StatusBarComponent {
    app_state: State<AppState>,
}

impl StatusBarComponent {
    pub fn new(app_state: State<AppState>) -> Self {
        Self { app_state }
    }
}

impl Component for StatusBarComponent {
    fn handle_input(&self, _input: &Event) {}
}

impl Widget for &StatusBarComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let state = self.app_state.get();

        let color = match state.status_level() {
            StatusLevel::Info => Color::Green,
            StatusLevel::Error => Color::Red,
        };

        let text = Span::styled(state.status_text().as_str(), Style::new().fg(color));

        let paragraph = Paragraph::new(text).block(
            Block::new()
                .border_type(BorderType::Plain)
                .borders(Borders::all()),
        );

        paragraph.render(area, buf);
    }
}
