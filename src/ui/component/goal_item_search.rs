use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use crate::infrastructure::util::component::Component;
use crate::infrastructure::util::state::State;
use crate::ui::state::{PlanTabFocus, PlanTabState};

pub struct GoalItemSearchComponent {
    plan_tab_state: State<PlanTabState>,
}

impl GoalItemSearchComponent {
    pub fn new(plan_tab_state: State<PlanTabState>) -> Self {
        Self { plan_tab_state }
    }
}

impl Component for GoalItemSearchComponent {
    #[expect(unused)]
    fn handle_input(&self, input: &Event) {
        match self.plan_tab_state.get().focus() {
            PlanTabFocus::GoalItemSearchInput => {}
            PlanTabFocus::PlanTreeList => {}
            _ => {}
        }
    }
}

impl Widget for &GoalItemSearchComponent {
    #[expect(unused)]
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
    }
}
