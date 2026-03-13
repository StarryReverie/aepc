use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};

use super::Component;

pub struct PlanTabComponent {}

impl PlanTabComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for PlanTabComponent {
    fn handle_input(&self, key: &KeyEvent) {
        match &key.code {
            _ => {}
        }
    }
}

impl Widget for &PlanTabComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let paragraph = Paragraph::new("Hello World").block(
            Block::new()
                .border_type(BorderType::Plain)
                .borders(Borders::all()),
        );
        paragraph.render(area, buf);
    }
}
