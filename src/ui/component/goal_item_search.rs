use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::Widget;

use crate::infrastructure::util::component::Component;
use crate::infrastructure::util::state::State;
use crate::ui::state::{PlanTabFocus, PlanTabState};

use super::GoalItemSearchInputComponent;

pub struct GoalItemSearchComponent {
    goal_item_search_input: GoalItemSearchInputComponent,
    plan_tab_state: State<PlanTabState>,
}

impl GoalItemSearchComponent {
    pub fn new(
        goal_item_search_input: GoalItemSearchInputComponent,
        plan_tab_state: State<PlanTabState>,
    ) -> Self {
        Self {
            goal_item_search_input,
            plan_tab_state,
        }
    }
}

impl Component for GoalItemSearchComponent {
    fn handle_input(&self, input: &Event) {
        match self.plan_tab_state.get().focus() {
            PlanTabFocus::GoalItemSearchInput => {
                self.goal_item_search_input.handle_input(input);
            }
            PlanTabFocus::PlanTreeList => {
                todo!("needs to forward inputs to goal_item_search_list");
            }
            _ => {}
        }
    }
}

impl Widget for &GoalItemSearchComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Fill(1)],
        )
        .split(area);

        self.goal_item_search_input.render(layout[0], buf);
    }
}
