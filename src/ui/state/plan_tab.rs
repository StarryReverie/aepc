use getset::{CopyGetters, Getters};
use tokio::sync::mpsc::{self, Receiver};

use crate::domain::item::model::ItemId;
use crate::domain::recipe::model::Flow;
use crate::infrastructure::util::state::{State, StateManager, StateManagerContext, StateSource};

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters)]
pub struct PlanTabState {
    #[getset(get_copy = "pub")]
    focus: PlanTabFocus,
    #[getset(get = "pub")]
    expected_goal_item: Option<ItemId>,
    #[getset(get_copy = "pub")]
    expected_goal_flow: Option<Flow>,
}

impl Default for PlanTabState {
    fn default() -> Self {
        Self {
            focus: PlanTabFocus::GoalFlowInput,
            expected_goal_item: None,
            expected_goal_flow: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanTabFocus {
    GoalFlowInput,
    GoalItemSearchInput,
    GoalItemSearchList,
    PlanTreeList,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanTabAction {
    UpdateExpectedGoalItem(ItemId),
    UpdateExpectedGoalFlow(Flow),
    SwitchFocusToNext,
    SwitchFocusToPrevious,
}

pub struct PlanTabStateManager {
    source: StateSource<PlanTabState>,
    actions: Receiver<PlanTabAction>,
}

impl PlanTabStateManager {
    pub fn context() -> StateManagerContext<Self> {
        let (requester, actions) = mpsc::channel(32);
        let (source, _) = StateSource::new(PlanTabState::default());
        let manager = Self { source, actions };
        StateManagerContext::new(manager, requester)
    }

    fn handle_action(&mut self, action: PlanTabAction) {
        match action {
            PlanTabAction::UpdateExpectedGoalItem(item) => {
                self.handle_action_update_expected_goal_item(item);
            }
            PlanTabAction::UpdateExpectedGoalFlow(flow) => {
                self.handle_action_update_expected_goal_flow(flow);
            }
            PlanTabAction::SwitchFocusToNext => {
                self.handle_action_switch_focus_to_next();
            }
            PlanTabAction::SwitchFocusToPrevious => {
                self.handle_action_switch_focus_to_previous();
            }
        }
    }

    fn handle_action_update_expected_goal_item(&mut self, item: ItemId) {
        self.source.modify(|state| PlanTabState {
            expected_goal_item: Some(item),
            ..state.clone()
        });
    }

    fn handle_action_update_expected_goal_flow(&mut self, flow: Flow) {
        self.source.modify(|state| PlanTabState {
            expected_goal_flow: Some(flow),
            ..state.clone()
        });
    }

    fn handle_action_switch_focus_to_next(&mut self) {
        self.source.modify(|state| PlanTabState {
            focus: match state.focus {
                PlanTabFocus::GoalFlowInput => PlanTabFocus::GoalItemSearchInput,
                PlanTabFocus::GoalItemSearchInput => PlanTabFocus::GoalItemSearchList,
                PlanTabFocus::GoalItemSearchList => PlanTabFocus::PlanTreeList,
                PlanTabFocus::PlanTreeList => PlanTabFocus::GoalFlowInput,
            },
            ..state.clone()
        });
    }

    fn handle_action_switch_focus_to_previous(&mut self) {
        self.source.modify(|state| PlanTabState {
            focus: match state.focus {
                PlanTabFocus::GoalFlowInput => PlanTabFocus::PlanTreeList,
                PlanTabFocus::GoalItemSearchInput => PlanTabFocus::GoalFlowInput,
                PlanTabFocus::GoalItemSearchList => PlanTabFocus::GoalItemSearchInput,
                PlanTabFocus::PlanTreeList => PlanTabFocus::GoalItemSearchList,
            },
            ..state.clone()
        });
    }
}

impl StateManager for PlanTabStateManager {
    type State = PlanTabState;

    type Action = PlanTabAction;

    fn state(&self) -> State<PlanTabState> {
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
