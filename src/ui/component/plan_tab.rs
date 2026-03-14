use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Widget};
use tokio::sync::mpsc::Sender;

use crate::ui::component::GoalSelectionPanelComponent;
use crate::ui::state::PlanTabAction;

use super::Component;

pub struct PlanTabComponent {
    goal_selection_panel: GoalSelectionPanelComponent,
    #[expect(dead_code)]
    plan_tab_requester: Sender<PlanTabAction>,
}

impl PlanTabComponent {
    pub fn new(
        goal_selection_panel: GoalSelectionPanelComponent,
        plan_tab_requester: Sender<PlanTabAction>,
    ) -> Self {
        Self {
            goal_selection_panel,
            plan_tab_requester,
        }
    }
}

impl Component for PlanTabComponent {
    fn handle_input(&self, key: &KeyEvent) {
        self.goal_selection_panel.handle_input(key);
    }
}

impl Widget for &PlanTabComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Fill(1), Constraint::Fill(2)],
        )
        .split(area);

        self.goal_selection_panel.render(layout[0], buf);
        Block::bordered().render(layout[1], buf);
    }
}
