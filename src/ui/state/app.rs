use getset::{CopyGetters, Getters};
use tokio::sync::mpsc::{self, Receiver};

use crate::infrastructure::util::state::{State, StateSource};

use super::{StateManager, StateManagerContext};

const INIT_STATUS_TEXT: &str = "Welcome to aepc ('Arknights: Endfield' Pipeline Calculator)";

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters, Getters)]
pub struct AppState {
    #[getset(get_copy = "pub")]
    running: bool,
    #[getset(get = "pub")]
    status_text: String,
    #[getset(get_copy = "pub")]
    status_level: StatusLevel,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            running: true,
            status_text: String::from(INIT_STATUS_TEXT),
            status_level: StatusLevel::Info,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    Info,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppAction {
    Quit,
    SetStatus { text: String, level: StatusLevel },
}

pub struct AppStateManager {
    source: StateSource<AppState>,
    actions: Receiver<AppAction>,
}

impl AppStateManager {
    pub fn context() -> StateManagerContext<Self> {
        let (requester, actions) = mpsc::channel(32);
        let (source, _) = StateSource::new(AppState::default());
        let manager = Self { source, actions };
        StateManagerContext::new(manager, requester)
    }

    fn handle_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => {
                self.handle_action_quit();
            }
            AppAction::SetStatus { text, level } => {
                self.handle_action_set_status(text, level);
            }
        }
    }

    fn handle_action_quit(&mut self) {
        self.source.modify(|state| AppState {
            running: false,
            ..state.clone()
        });
    }

    fn handle_action_set_status(&mut self, text: String, level: StatusLevel) {
        self.source.modify(|state| AppState {
            status_text: text,
            status_level: level,
            ..state.clone()
        });
    }
}

impl StateManager for AppStateManager {
    type State = AppState;

    type Action = AppAction;

    fn state(&self) -> State<AppState> {
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
    use tokio::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_quit_action() {
        let context = AppStateManager::context();
        let mut state = context.state();
        let requester = context.requester();

        context.run();
        requester.send(AppAction::Quit).await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        let new_state = state.watch().await.unwrap();
        assert!(!new_state.running());
    }

    #[tokio::test]
    async fn test_set_status_info() {
        let context = AppStateManager::context();
        let mut state = context.state();
        let requester = context.requester();

        context.run();
        requester
            .send(AppAction::SetStatus {
                text: "Info message".to_string(),
                level: StatusLevel::Info,
            })
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        let new_state = state.watch().await.unwrap();
        assert_eq!(new_state.status_text(), "Info message");
        assert_eq!(new_state.status_level(), StatusLevel::Info);
    }

    #[tokio::test]
    async fn test_set_status_error() {
        let context = AppStateManager::context();
        let mut state = context.state();
        let requester = context.requester();

        context.run();
        requester
            .send(AppAction::SetStatus {
                text: "Error message".to_string(),
                level: StatusLevel::Error,
            })
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        let new_state = state.watch().await.unwrap();
        assert_eq!(new_state.status_text(), "Error message");
        assert_eq!(new_state.status_level(), StatusLevel::Error);
    }
}
