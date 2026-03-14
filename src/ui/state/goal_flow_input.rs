use getset::Getters;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::domain::recipe::model::Flow;
use crate::infrastructure::util::state::{State, StateSource};

use super::{AppAction, PlanTabAction, StateManager, StateManagerContext, StatusLevel};

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct GoalFlowInputState {
    #[getset(get = "pub")]
    input_text: String,
}

impl Default for GoalFlowInputState {
    fn default() -> Self {
        Self {
            input_text: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoalFlowInputAction {
    Input(char),
    Backspace,
    Confirm,
    Clear,
}

pub struct GoalFlowInputStateManager {
    source: StateSource<GoalFlowInputState>,
    actions: Receiver<GoalFlowInputAction>,
    plan_tab_requester: Sender<PlanTabAction>,
    app_requester: Sender<AppAction>,
}

impl GoalFlowInputStateManager {
    pub fn context(
        plan_tab_requester: Sender<PlanTabAction>,
        app_requester: Sender<AppAction>,
    ) -> StateManagerContext<Self> {
        let (requester, actions) = mpsc::channel(32);
        let (source, _) = StateSource::new(GoalFlowInputState::default());
        let manager = Self {
            source,
            actions,
            plan_tab_requester,
            app_requester,
        };
        StateManagerContext::new(manager, requester)
    }

    fn handle_action(&mut self, action: GoalFlowInputAction) {
        match action {
            GoalFlowInputAction::Input(c) => self.handle_action_input(c),
            GoalFlowInputAction::Backspace => self.handle_action_backspace(),
            GoalFlowInputAction::Confirm => self.handle_action_confirm(),
            GoalFlowInputAction::Clear => self.handle_action_clear(),
        }
    }

    fn handle_action_input(&mut self, c: char) {
        if c.is_ascii_digit() || c == '.' {
            self.source.modify(|state| {
                let mut text = state.input_text.clone();
                text.push(c);
                GoalFlowInputState {
                    input_text: text,
                    ..state.clone()
                }
            });
        }
    }

    fn handle_action_backspace(&mut self) {
        self.source.modify(|state| {
            let mut text = state.input_text.clone();
            text.pop();
            GoalFlowInputState {
                input_text: text,
                ..state.clone()
            }
        });
    }

    fn handle_action_confirm(&mut self) {
        let input_text = self.source.get().input_text().clone();
        match input_text.parse::<f64>() {
            Ok(value) => match Flow::new(value) {
                Ok(flow) => {
                    let _ = self
                        .plan_tab_requester
                        .try_send(PlanTabAction::UpdateExpectedGoalFlow(flow));
                    let _ = self
                        .plan_tab_requester
                        .try_send(PlanTabAction::SwitchFocusToNext);
                }
                Err(err) => {
                    let _ = self.app_requester.try_send(AppAction::SetStatus {
                        text: err.to_string(),
                        level: StatusLevel::Error,
                    });
                }
            },
            Err(_) => {
                let _ = self.app_requester.try_send(AppAction::SetStatus {
                    text: if input_text.is_empty() {
                        "Flow value cannot be empty".to_string()
                    } else {
                        format!("'{}' is not a valid number", input_text)
                    },
                    level: StatusLevel::Error,
                });
            }
        }
    }

    fn handle_action_clear(&mut self) {
        self.source.modify(|state| GoalFlowInputState {
            input_text: String::new(),
            ..state.clone()
        });
    }
}

impl StateManager for GoalFlowInputStateManager {
    type State = GoalFlowInputState;

    type Action = GoalFlowInputAction;

    fn state(&self) -> State<GoalFlowInputState> {
        self.source.subscribe()
    }

    fn run(mut self) {
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(action) = self.actions.recv() => {
                        self.handle_action(action);
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_confirm() {
        let (plan_tab_requester, mut plan_tab_actions) = mpsc::channel(32);
        let (app_requester, mut app_actions) = mpsc::channel(32);
        let context =
            GoalFlowInputStateManager::context(plan_tab_requester.clone(), app_requester.clone());

        let requester = context.requester();
        context.run();

        let _ = requester.try_send(GoalFlowInputAction::Input('1'));
        let _ = requester.try_send(GoalFlowInputAction::Input('.'));
        let _ = requester.try_send(GoalFlowInputAction::Input('2'));
        let _ = requester.try_send(GoalFlowInputAction::Confirm);

        assert!(matches!(
            plan_tab_actions.recv().await.unwrap(),
            PlanTabAction::UpdateExpectedGoalFlow(_),
        ));
        assert!(matches!(
            plan_tab_actions.recv().await.unwrap(),
            PlanTabAction::SwitchFocusToNext,
        ));

        let _ = requester.try_send(GoalFlowInputAction::Clear);
        let _ = requester.try_send(GoalFlowInputAction::Confirm);

        assert!(matches!(
            app_actions.recv().await.unwrap(),
            AppAction::SetStatus {
                level: StatusLevel::Error,
                ..
            },
        ));

        let _ = requester.try_send(GoalFlowInputAction::Input('a'));
        let _ = requester.try_send(GoalFlowInputAction::Confirm);

        assert!(matches!(
            app_actions.recv().await.unwrap(),
            AppAction::SetStatus {
                level: StatusLevel::Error,
                ..
            },
        ));
    }
}
