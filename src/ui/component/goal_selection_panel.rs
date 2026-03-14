use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::Widget;

use crate::infrastructure::util::component::Component;
use crate::infrastructure::util::state::State;
use crate::ui::state::{PlanTabFocus, PlanTabState};

use super::{GoalFlowInputComponent, GoalItemSearchComponent};

pub struct GoalSelectionPanelComponent {
    goal_flow_input: GoalFlowInputComponent,
    goal_item_search: GoalItemSearchComponent,
    plan_tab_state: State<PlanTabState>,
}

impl GoalSelectionPanelComponent {
    pub fn new(
        goal_flow_input: GoalFlowInputComponent,
        goal_item_search: GoalItemSearchComponent,
        plan_tab_state: State<PlanTabState>,
    ) -> Self {
        Self {
            goal_flow_input,
            goal_item_search,
            plan_tab_state,
        }
    }
}

impl Component for GoalSelectionPanelComponent {
    fn handle_input(&self, input: &Event) {
        match self.plan_tab_state.get().focus() {
            PlanTabFocus::GoalFlowInput => {
                self.goal_flow_input.handle_input(input);
            }
            PlanTabFocus::GoalItemSearchInput | PlanTabFocus::GoalItemSearchList => {
                self.goal_item_search.handle_input(input);
            }
            _ => {}
        }
    }
}

impl Widget for &GoalSelectionPanelComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Fill(1)],
        )
        .split(area);

        self.goal_flow_input.render(layout[0], buf);
        self.goal_item_search.render(layout[1], buf);
    }
}
