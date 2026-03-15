use tokio::sync::mpsc::Sender;

use crate::ui::state::{GoalItemSearchAction, PlanTabAction};

pub fn create_goal_item_search_input_on_confirm(
    plan_tab_requester: Sender<PlanTabAction>,
) -> impl FnMut(&str) + Send + Sync + 'static {
    move |_| {
        let _ = plan_tab_requester.try_send(PlanTabAction::SwitchFocusToNext);
    }
}

pub fn create_goal_item_search_input_on_update(
    goal_item_search_requester: Sender<GoalItemSearchAction>,
) -> impl FnMut(&str) + Send + Sync + 'static {
    move |pattern| {
        let _ = goal_item_search_requester.try_send(GoalItemSearchAction::UpdateItemNamePattern(
            pattern.to_string(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::*;

    #[tokio::test]
    async fn test_on_confirm_switches_focus() {
        let (plan_tab_requester, mut plan_tab_actions) = mpsc::channel(32);

        let mut on_confirm = create_goal_item_search_input_on_confirm(plan_tab_requester);
        on_confirm("test_pattern");

        assert!(matches!(
            plan_tab_actions.recv().await.unwrap(),
            PlanTabAction::SwitchFocusToNext,
        ));
    }

    #[tokio::test]
    async fn test_on_update_updates_pattern() {
        let (goal_item_search_requester, mut goal_item_search_actions) = mpsc::channel(32);

        let mut on_confirm = create_goal_item_search_input_on_update(goal_item_search_requester);
        on_confirm("test_pattern");

        assert!(matches!(
            goal_item_search_actions.recv().await.unwrap(),
            GoalItemSearchAction::UpdateItemNamePattern(_),
        ));
    }
}
