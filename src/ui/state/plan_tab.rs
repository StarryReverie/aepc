use getset::{CopyGetters, Getters};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::domain::item::model::ItemId;
use crate::domain::recipe::model::Flow;
use crate::infrastructure::util::state::{State, StateSource};

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
            focus: PlanTabFocus::GoalSelectionPanel,
            expected_goal_item: None,
            expected_goal_flow: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanTabFocus {
    GoalSelectionPanel,
    PlanDisplayPanel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanTabAction {
    UpdateExpectedGoalItem(ItemId),
    UpdateExpectedGoalFlow(Flow),
    SwitchFocus,
}

pub struct PlanTabStateManager {
    source: StateSource<PlanTabState>,
    actions: Receiver<PlanTabAction>,
}

impl PlanTabStateManager {
    pub fn new() -> (Self, Sender<PlanTabAction>) {
        let (requester, actions) = mpsc::channel(32);
        let (source, _) = StateSource::new(PlanTabState::default());
        let res = Self { source, actions };
        (res, requester)
    }

    pub fn state(&self) -> State<PlanTabState> {
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

    fn handle_action(&mut self, action: PlanTabAction) {
        match action {
            PlanTabAction::UpdateExpectedGoalItem(item) => {
                self.handle_action_update_expected_goal_item(item);
            }
            PlanTabAction::UpdateExpectedGoalFlow(flow) => {
                self.handle_action_update_expected_goal_flow(flow);
            }
            PlanTabAction::SwitchFocus => {
                self.handle_action_switch_focus();
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

    fn handle_action_switch_focus(&mut self) {
        self.source.modify(|state| PlanTabState {
            focus: match state.focus {
                PlanTabFocus::GoalSelectionPanel => PlanTabFocus::PlanDisplayPanel,
                PlanTabFocus::PlanDisplayPanel => PlanTabFocus::GoalSelectionPanel,
            },
            ..state.clone()
        });
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_update_expected_goal_item() {
        let (manager, requester) = PlanTabStateManager::new();
        let mut state = manager.state();

        manager.run();

        let item_id = ItemId::new("test_item".to_string()).unwrap();
        requester
            .send(PlanTabAction::UpdateExpectedGoalItem(item_id.clone()))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        let new_state = state.watch().await.unwrap();
        assert_eq!(new_state.expected_goal_item(), &Some(item_id));
    }

    #[tokio::test]
    async fn test_update_expected_goal_flow() {
        let (manager, requester) = PlanTabStateManager::new();
        let mut state = manager.state();

        manager.run();

        let flow = Flow::new(100.0).unwrap();
        requester
            .send(PlanTabAction::UpdateExpectedGoalFlow(flow))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        let new_state = state.watch().await.unwrap();
        assert_eq!(new_state.expected_goal_flow(), Some(flow));
    }

    #[tokio::test]
    async fn test_switch_focus() {
        let (manager, requester) = PlanTabStateManager::new();
        let mut state = manager.state();

        manager.run();

        requester.send(PlanTabAction::SwitchFocus).await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        let new_state = state.watch().await.unwrap();
        assert_eq!(new_state.focus(), PlanTabFocus::PlanDisplayPanel);

        requester.send(PlanTabAction::SwitchFocus).await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        let new_state = state.watch().await.unwrap();
        assert_eq!(new_state.focus(), PlanTabFocus::GoalSelectionPanel);
    }
}
