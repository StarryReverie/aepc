use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::Widget;

use crate::infrastructure::util::component::Component;
use crate::ui::component::{PlanGoalLabelComponent, PlanTreeListComponent};

pub struct PlanDisplayPanelComponent {
    plan_goal_label: PlanGoalLabelComponent,
    plan_tree_list: PlanTreeListComponent,
}

impl PlanDisplayPanelComponent {
    pub fn new(
        plan_goal_label: PlanGoalLabelComponent,
        plan_tree_list: PlanTreeListComponent,
    ) -> Self {
        Self {
            plan_goal_label,
            plan_tree_list,
        }
    }
}

impl Component for PlanDisplayPanelComponent {
    fn handle_input(&self, input: &Event) {
        self.plan_tree_list.handle_input(input);
    }
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
        self.plan_tree_list.render(layout[1], buf);
    }
}
