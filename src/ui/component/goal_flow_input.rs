use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use tokio::sync::mpsc::Sender;

use crate::infrastructure::util::component::{Component, TextInputUtilComponent};
use crate::infrastructure::util::state::State;
use crate::ui::state::{self, AppAction, PlanTabAction, PlanTabState};
use crate::ui::style;

pub struct GoalFlowInputComponent {
    text_input: TextInputUtilComponent,
}

impl GoalFlowInputComponent {
    pub fn new(
        plan_tab_requester: Sender<PlanTabAction>,
        app_requester: Sender<AppAction>,
        plan_tab_state: State<PlanTabState>,
    ) -> Self {
        let text_input = TextInputUtilComponent::new(
            |c| c.is_ascii_digit() || c == '.',
            state::create_goal_flow_input_on_confirm(plan_tab_requester, app_requester),
            |_| {},
            state::create_goal_flow_input_is_focused(plan_tab_state),
            || " Expected Flow (items/min) ".to_string(),
            style::block_with_focused,
        );

        Self { text_input }
    }
}

impl Component for GoalFlowInputComponent {
    fn handle_input(&self, input: &Event) {
        self.text_input.handle_input(input);
    }
}

impl Widget for &GoalFlowInputComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.text_input.render(area, buf);
    }
}
