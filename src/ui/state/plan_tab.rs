use getset::{CopyGetters, Getters};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::domain::item::model::Item;
use crate::domain::recipe::model::Flow;
use crate::infrastructure::util::state::{State, StateManager, StateManagerContext, StateSource};
use crate::ui::state::AppAction;

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters)]
pub struct PlanTabState {
    #[getset(get_copy = "pub")]
    focus: PlanTabFocus,
    #[getset(get = "pub")]
    expected_goal_item: Option<Item>,
    #[getset(get_copy = "pub")]
    expected_goal_flow: Option<Flow>,
}

impl Default for PlanTabState {
    fn default() -> Self {
        Self {
            focus: PlanTabFocus::default(),
            expected_goal_item: None,
            expected_goal_flow: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlanTabFocus {
    #[default]
    GoalFlowInput,
    GoalItemSearchInput,
    GoalItemSearchList,
    PlanTreeList,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanTabAction {
    UpdateExpectedGoalItem(Item),
    UpdateExpectedGoalFlow(Flow),
    SwitchFocusToNext,
    SwitchFocusToPrevious,
}

pub struct PlanTabStateManager {
    source: StateSource<PlanTabState>,
    actions: Receiver<PlanTabAction>,
    app_requester: Sender<AppAction>,
}

impl PlanTabStateManager {
    pub fn context(app_requester: Sender<AppAction>) -> StateManagerContext<Self> {
        let (requester, actions) = mpsc::channel(32);
        let (source, _) = StateSource::new(PlanTabState::default());
        let manager = Self {
            source,
            actions,
            app_requester,
        };
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

    fn handle_action_update_expected_goal_item(&mut self, item: Item) {
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
        let new_focus = match self.source.get().focus {
            PlanTabFocus::GoalFlowInput => PlanTabFocus::GoalItemSearchInput,
            PlanTabFocus::GoalItemSearchInput => PlanTabFocus::GoalItemSearchList,
            PlanTabFocus::GoalItemSearchList => PlanTabFocus::PlanTreeList,
            PlanTabFocus::PlanTreeList => PlanTabFocus::GoalFlowInput,
        };
        self.source.modify(|state| PlanTabState {
            focus: new_focus,
            ..state.clone()
        });
        let _ =
            self.app_requester
                .try_send(AppAction::SetFocus(crate::ui::state::AppFocus::PlanTab(
                    new_focus,
                )));
    }

    fn handle_action_switch_focus_to_previous(&mut self) {
        let new_focus = match self.source.get().focus {
            PlanTabFocus::GoalFlowInput => PlanTabFocus::PlanTreeList,
            PlanTabFocus::GoalItemSearchInput => PlanTabFocus::GoalFlowInput,
            PlanTabFocus::GoalItemSearchList => PlanTabFocus::GoalItemSearchInput,
            PlanTabFocus::PlanTreeList => PlanTabFocus::GoalItemSearchList,
        };
        self.source.modify(|state| PlanTabState {
            focus: new_focus,
            ..state.clone()
        });
        let _ =
            self.app_requester
                .try_send(AppAction::SetFocus(crate::ui::state::AppFocus::PlanTab(
                    new_focus,
                )));
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
