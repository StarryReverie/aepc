use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Widget};

use crate::infrastructure::util::component::Component;

pub struct PlanDisplayPanelComponent {}

impl PlanDisplayPanelComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for PlanDisplayPanelComponent {
    #[expect(unused)]
    fn handle_input(&self, input: &Event) {}
}

impl Widget for &PlanDisplayPanelComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Fill(1)],
        )
        .split(area);

        Block::bordered().render(layout[0], buf);
        Block::bordered().render(layout[1], buf);
    }
}
