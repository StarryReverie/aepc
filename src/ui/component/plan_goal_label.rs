use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};

use crate::infrastructure::util::component::Component;
use crate::infrastructure::util::state::State;
use crate::ui::state::PlanTabState;

pub struct PlanGoalLabelComponent {
    plan_tab_state: State<PlanTabState>,
}

impl PlanGoalLabelComponent {
    pub fn new(plan_tab_state: State<PlanTabState>) -> Self {
        Self { plan_tab_state }
    }
}

impl Component for PlanGoalLabelComponent {
    fn handle_input(&self, _input: &Event) {}
}

impl Widget for &PlanGoalLabelComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let plan_tab_state = self.plan_tab_state.get();
        let expected_goal_item = plan_tab_state.expected_goal_item();
        let expected_goal_flow = plan_tab_state.expected_goal_flow();

        let line = Line::from(vec![
            Span::from(" Expected Goal = "),
            expected_goal_item
                .as_ref()
                .map(|item| Span::from(item.name().value()))
                .unwrap_or(Span::from("(None)")),
            Span::from(" x "),
            expected_goal_flow
                .map(|flow| Span::from(flow.to_string()))
                .unwrap_or(Span::from("(None)")),
        ]);

        let paragraph = Paragraph::new(line).block(
            Block::new()
                .border_type(BorderType::Plain)
                .borders(Borders::all())
                .title(" Goal "),
        );

        paragraph.render(area, buf);
    }
}
