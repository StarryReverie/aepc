use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use tokio::sync::mpsc::Sender;

use crate::infrastructure::util::component::{Component, TextInputUtilComponent};
use crate::infrastructure::util::state::State;
use crate::ui::state::{self, GoalItemSearchAction, PlanTabAction, PlanTabFocus, PlanTabState};
use crate::ui::style;

pub struct GoalItemSearchInputComponent {
    text_input: TextInputUtilComponent,
}

impl GoalItemSearchInputComponent {
    pub fn new(
        goal_item_search_requester: Sender<GoalItemSearchAction>,
        plan_tab_requester: Sender<PlanTabAction>,
        plan_tab_state: State<PlanTabState>,
    ) -> Self {
        let text_input = TextInputUtilComponent::new(
            |_| true,
            state::create_goal_item_search_input_on_confirm(plan_tab_requester),
            state::create_goal_item_search_input_on_update(goal_item_search_requester),
            move || plan_tab_state.get().focus() == PlanTabFocus::GoalItemSearchInput,
            || " Expected Item Search ".to_string(),
            style::block_with_focused,
        );
        Self { text_input }
    }
}

impl Component for GoalItemSearchInputComponent {
    fn handle_input(&self, input: &Event) {
        self.text_input.handle_input(input);
    }
}

impl Widget for &GoalItemSearchInputComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.text_input.render(area, buf);
    }
}
