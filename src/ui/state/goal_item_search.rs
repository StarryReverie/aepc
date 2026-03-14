use getset::Getters;
use tokio::sync::mpsc::{self, Receiver};

use crate::infrastructure::util::state::{State, StateSource};
use crate::ui::state::{StateManager, StateManagerContext};

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct GoalItemSearchState {
    #[getset(get = "pub")]
    item_name_pattern: String,
}

impl Default for GoalItemSearchState {
    fn default() -> Self {
        Self {
            item_name_pattern: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoalItemSearchAction {
    UpdateItemNamePattern(String),
}

pub struct GoalItemSearchStateManager {
    source: StateSource<GoalItemSearchState>,
    actions: Receiver<GoalItemSearchAction>,
}

impl GoalItemSearchStateManager {
    pub fn context() -> StateManagerContext<Self> {
        let (requester, actions) = mpsc::channel(32);
        let (source, _) = StateSource::new(GoalItemSearchState::default());
        let manager = Self { source, actions };
        StateManagerContext::new(manager, requester)
    }

    fn handle_action(&mut self, action: GoalItemSearchAction) {
        match action {
            GoalItemSearchAction::UpdateItemNamePattern(pattern) => {
                self.handle_action_update_item_name_pattern(pattern);
            }
        }
    }

    fn handle_action_update_item_name_pattern(&mut self, pattern: String) {
        self.source.modify(|state| GoalItemSearchState {
            item_name_pattern: pattern,
            ..state.clone()
        });
    }
}

impl StateManager for GoalItemSearchStateManager {
    type State = GoalItemSearchState;

    type Action = GoalItemSearchAction;

    fn state(&self) -> State<Self::State> {
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
