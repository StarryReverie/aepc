use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};

use super::Component;

pub struct GoalSelectionPanelComponent {}

impl GoalSelectionPanelComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for GoalSelectionPanelComponent {
    fn handle_input(&self, key: &KeyEvent) {
        #[expect(clippy::match_single_binding)]
        match &key.code {
            _ => {}
        }
    }
}

impl Widget for &GoalSelectionPanelComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let paragraph = Paragraph::new("goal selection panel").block(
            Block::new()
                .border_type(BorderType::Plain)
                .borders(Borders::all()),
        );
        paragraph.render(area, buf);
    }
}
