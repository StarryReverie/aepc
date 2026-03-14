use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Widget};
use tokio::sync::mpsc::Sender;

use crate::infrastructure::util::state::State;
use crate::ui::component::GoalSelectionPanelComponent;
use crate::ui::state::{PlanTabAction, PlanTabFocus, PlanTabState};

use super::Component;

pub struct PlanTabComponent {
    goal_selection_panel: GoalSelectionPanelComponent,
    plan_tab_requester: Sender<PlanTabAction>,
    plan_tab_state: State<PlanTabState>,
}

impl PlanTabComponent {
    pub fn new(
        goal_selection_panel: GoalSelectionPanelComponent,
        plan_tab_requester: Sender<PlanTabAction>,
        plan_tab_state: State<PlanTabState>,
    ) -> Self {
        Self {
            goal_selection_panel,
            plan_tab_requester,
            plan_tab_state,
        }
    }
}

impl Component for PlanTabComponent {
    fn handle_input(&self, key: &KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                let _ = self
                    .plan_tab_requester
                    .try_send(PlanTabAction::SwitchFocusToNext);
            }
            KeyCode::BackTab => {
                let _ = self
                    .plan_tab_requester
                    .try_send(PlanTabAction::SwitchFocusToPrevious);
            }
            _ => match self.plan_tab_state.get().focus() {
                PlanTabFocus::GoalFlowInput
                | PlanTabFocus::GoalItemSearchInput
                | PlanTabFocus::GoalItemSearchList => {
                    self.goal_selection_panel.handle_input(key);
                }
                _ => {}
            },
        }
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
