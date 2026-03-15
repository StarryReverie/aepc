use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Widget};

use crate::infrastructure::util::component::Component;
use crate::ui::component::PlanGoalLabelComponent;

pub struct PlanDisplayPanelComponent {
    plan_goal_label: PlanGoalLabelComponent,
}

impl PlanDisplayPanelComponent {
    pub fn new(plan_goal_label: PlanGoalLabelComponent) -> Self {
        Self { plan_goal_label }
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

        self.plan_goal_label.render(layout[0], buf);
        Block::bordered().render(layout[1], buf);
    }
}
