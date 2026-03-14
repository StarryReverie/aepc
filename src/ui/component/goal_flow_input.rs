use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};
use tokio::sync::mpsc::Sender;

use crate::infrastructure::util::state::State;
use crate::ui::state::{GoalFlowInputAction, GoalFlowInputState, PlanTabFocus, PlanTabState};

use super::Component;

pub struct GoalFlowInputComponent {
    goal_flow_input_requester: Sender<GoalFlowInputAction>,
    goal_flow_input_state: State<GoalFlowInputState>,
    plan_tab_state: State<PlanTabState>,
}

impl GoalFlowInputComponent {
    pub fn new(
        goal_flow_input_requester: Sender<GoalFlowInputAction>,
        goal_flow_input_state: State<GoalFlowInputState>,
        plan_tab_state: State<PlanTabState>,
    ) -> Self {
        Self {
            goal_flow_input_requester,
            goal_flow_input_state,
            plan_tab_state,
        }
    }
}

impl Component for GoalFlowInputComponent {
    fn handle_input(&self, key: &KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                let _ = self
                    .goal_flow_input_requester
                    .try_send(GoalFlowInputAction::Input(c));
            }
            KeyCode::Backspace => {
                let _ = self
                    .goal_flow_input_requester
                    .try_send(GoalFlowInputAction::Backspace);
            }
            KeyCode::Enter => {
                let _ = self
                    .goal_flow_input_requester
                    .try_send(GoalFlowInputAction::Confirm);
            }
            KeyCode::Esc => {
                let _ = self
                    .goal_flow_input_requester
                    .try_send(GoalFlowInputAction::Clear);
            }
            _ => {}
        }
    }
}

impl Widget for &GoalFlowInputComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let goal_flow_input_state = self.goal_flow_input_state.get();
        let plan_tab_state = self.plan_tab_state.get();

        let is_focused = plan_tab_state.focus() == PlanTabFocus::GoalFlowInput;

        let line = Line::from(vec![
            Span::from(goal_flow_input_state.input_text()),
            Span::raw(if is_focused { "█" } else { "" }),
        ]);

        let border_style = if is_focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let paragraph = Paragraph::new(line).block(
            Block::new()
                .border_type(BorderType::Plain)
                .borders(Borders::all())
                .border_style(border_style)
                .title("Expected Flow (items/min)"),
        );

        paragraph.render(area, buf);
    }
}
