use getset::CopyGetters;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::infrastructure::util::state::{State, StateSource};

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters)]
pub struct AppState {
    #[getset(get_copy = "pub")]
    running: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self { running: true }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppAction {
    Quit,
}

pub struct AppStateManager {
    source: StateSource<AppState>,
    actions: Receiver<AppAction>,
}

impl AppStateManager {
    pub fn new() -> (Self, Sender<AppAction>) {
        let (requester, actions) = mpsc::channel(32);
        let (source, _) = StateSource::new(AppState::default());
        let res = Self { source, actions };
        (res, requester)
    }

    pub fn state(&self) -> State<AppState> {
        self.source.subscribe()
    }

    pub fn run(mut self) {
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

    fn handle_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => {
                self.handle_action_quit();
            }
        }
    }

    fn handle_action_quit(&mut self) {
        self.source.modify(|state| AppState {
            running: false,
            ..state.clone()
        });
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_quit_action() {
        let (manager, requester) = AppStateManager::new();
        let mut state = manager.state();

        manager.run();
        requester.send(AppAction::Quit).await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        let new_state = state.watch().await.unwrap();
        assert!(!new_state.running());
    }
}
