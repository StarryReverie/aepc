use std::convert::Infallible;

use getset::{CopyGetters, Getters};
use tokio::sync::mpsc::{self, Sender};
use tokio::time::Duration;

use crate::infrastructure::util::state::{State, StateManager, StateManagerContext, StateSource};
use crate::ui::state::AppState;

const SCROLL_INTERVAL_MS: u64 = 200;
const INITIAL_DELAY_MS: u64 = 3000;

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters)]
pub struct StatusBarState {
    #[getset(get = "pub")]
    text: String,
    #[getset(get_copy = "pub")]
    scroll_offset: usize,
    #[getset(get_copy = "pub")]
    scroll_delay_counter: u64,
}

impl Default for StatusBarState {
    fn default() -> Self {
        Self {
            text: String::new(),
            scroll_offset: 0,
            scroll_delay_counter: 0,
        }
    }
}

pub struct StatusBarStateManager {
    source: StateSource<StatusBarState>,
    app_state: State<AppState>,
}

impl StatusBarStateManager {
    pub fn context(app_state: State<AppState>) -> StateManagerContext<Self> {
        let (source, _) = StateSource::new(StatusBarState::default());
        let manager = Self { source, app_state };
        let (requester, _): (Sender<Infallible>, _) = mpsc::channel(1);
        StateManagerContext::new(manager, requester)
    }

    fn handle_app_state_change(&mut self, app_state: &AppState) {
        let new_text = app_state.status_text().clone();
        let text_changed = {
            let state = self.source.get();
            state.text() != &new_text
        };

        if text_changed {
            self.source.modify(|state| StatusBarState {
                text: new_text,
                scroll_offset: 0,
                scroll_delay_counter: 0,
                ..state.clone()
            });
        }
    }

    fn handle_action_scroll(&mut self) {
        self.source.modify(|state| {
            let len = state.text().chars().count();
            if state.scroll_delay_counter() < INITIAL_DELAY_MS {
                StatusBarState {
                    scroll_delay_counter: state.scroll_delay_counter() + SCROLL_INTERVAL_MS,
                    ..state.clone()
                }
            } else if state.scroll_offset() < len {
                StatusBarState {
                    scroll_offset: state.scroll_offset() + 1,
                    ..state.clone()
                }
            } else {
                StatusBarState {
                    scroll_offset: 0,
                    scroll_delay_counter: 0,
                    ..state.clone()
                }
            }
        });
    }
}

impl StateManager for StatusBarStateManager {
    type State = StatusBarState;

    type Action = Infallible;

    fn state(&self) -> State<StatusBarState> {
        self.source.subscribe()
    }

    fn run(mut self) {
        let initial_text = self.app_state.get().status_text().clone();
        self.source.modify(|state| StatusBarState {
            text: initial_text,
            ..state.clone()
        });

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(new_app_state) = self.app_state.watch() => {
                        self.handle_app_state_change(&new_app_state);
                    }
                    _ = tokio::time::sleep(Duration::from_millis(SCROLL_INTERVAL_MS)) => {
                        self.handle_action_scroll();
                    }
                }
            }
        });
    }
}
