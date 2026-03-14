use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};

use crate::infrastructure::util::component::Component;
use crate::infrastructure::util::state::State;
use crate::ui::state::{PlanTabFocus, PlanTabState};

use super::GoalFlowInputComponent;

pub struct GoalSelectionPanelComponent {
    goal_flow_input: GoalFlowInputComponent,
    plan_tab_state: State<PlanTabState>,
}

impl GoalSelectionPanelComponent {
    pub fn new(
        goal_flow_input: GoalFlowInputComponent,
        plan_tab_state: State<PlanTabState>,
    ) -> Self {
        Self {
            goal_flow_input,
            plan_tab_state,
        }
    }
}

impl Component for GoalSelectionPanelComponent {
    fn handle_input(&self, input: &Event) {
        #[expect(clippy::single_match)]
        match self.plan_tab_state.get().focus() {
            PlanTabFocus::GoalFlowInput => {
                self.goal_flow_input.handle_input(input);
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

        let paragraph = Paragraph::new("").block(
            Block::new()
                .border_type(BorderType::Plain)
                .borders(Borders::all()),
        );
        paragraph.render(layout[1], buf);
    }
}
