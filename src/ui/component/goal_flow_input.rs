use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use tokio::sync::mpsc::Sender;

use crate::infrastructure::util::component::TextInputUtilComponent;
use crate::infrastructure::util::state::State;
use crate::ui::state::{self, AppAction, PlanTabAction, PlanTabFocus, PlanTabState};

use super::Component;

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
            state::create_goal_flow_on_confirm(plan_tab_requester, app_requester),
            move || plan_tab_state.get().focus() == PlanTabFocus::GoalFlowInput,
            || " Expected Flow (items/min) ".to_string(),
        );

        Self { text_input }
    }
}

impl Component for GoalFlowInputComponent {
    fn handle_input(&self, key: &KeyEvent) {
        self.text_input.handle_input(key);
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
