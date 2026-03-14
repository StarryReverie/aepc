use tokio::sync::mpsc::Sender;

use crate::domain::recipe::model::Flow;
use crate::ui::state::{AppAction, PlanTabAction, StatusLevel};

pub fn create_goal_flow_on_confirm(
    plan_tab_requester: Sender<PlanTabAction>,
    app_requester: Sender<AppAction>,
) -> impl FnMut(&str) + Send + 'static {
    move |text| match text.parse::<f64>() {
        Ok(value) => match Flow::new(value) {
            Ok(flow) => {
                let _ = plan_tab_requester.try_send(PlanTabAction::UpdateExpectedGoalFlow(flow));
                let _ = plan_tab_requester.try_send(PlanTabAction::SwitchFocusToNext);
            }
            Err(e) => {
                let _ = app_requester.try_send(AppAction::SetStatus {
                    text: e.to_string(),
                    level: StatusLevel::Error,
                });
            }
        },
        Err(_) => {
            let _ = app_requester.try_send(AppAction::SetStatus {
                text: if text.is_empty() {
                    "Flow value cannot be empty".to_string()
                } else {
                    format!("'{}' is not a valid number", text)
                },
                level: StatusLevel::Error,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::*;

    #[tokio::test]
    async fn test_on_confirm_valid_flow() {
        let (plan_tab_requester, mut plan_tab_actions) = mpsc::channel(32);
        let (app_requester, mut app_actions) = mpsc::channel(32);

        let mut on_confirm =
            create_goal_flow_on_confirm(plan_tab_requester.clone(), app_requester.clone());

        on_confirm("10.5");

        assert!(matches!(
            plan_tab_actions.recv().await.unwrap(),
            PlanTabAction::UpdateExpectedGoalFlow(_),
        ));
        assert!(matches!(
            plan_tab_actions.recv().await.unwrap(),
            PlanTabAction::SwitchFocusToNext,
        ));

        let result = app_actions.try_recv();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_on_confirm_negative_flow() {
        let (plan_tab_requester, mut plan_tab_actions) = mpsc::channel(32);
        let (app_requester, mut app_actions) = mpsc::channel(32);

        let mut on_confirm =
            create_goal_flow_on_confirm(plan_tab_requester.clone(), app_requester.clone());

        on_confirm("-10.5");

        assert!(matches!(
            app_actions.recv().await.unwrap(),
            AppAction::SetStatus {
                level: StatusLevel::Error,
                ..
            },
        ));

        let result = plan_tab_actions.try_recv();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_on_confirm_empty_input() {
        let (plan_tab_requester, mut plan_tab_actions) = mpsc::channel(32);
        let (app_requester, mut app_actions) = mpsc::channel(32);

        let mut on_confirm =
            create_goal_flow_on_confirm(plan_tab_requester.clone(), app_requester.clone());

        on_confirm("");

        assert!(matches!(
            app_actions.recv().await.unwrap(),
            AppAction::SetStatus {
                level: StatusLevel::Error,
                ..
            },
        ));

        let result = plan_tab_actions.try_recv();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_on_confirm_invalid_number() {
        let (plan_tab_requester, mut plan_tab_actions) = mpsc::channel(32);
        let (app_requester, mut app_actions) = mpsc::channel(32);

        let mut on_confirm =
            create_goal_flow_on_confirm(plan_tab_requester.clone(), app_requester.clone());

        on_confirm("abc");

        assert!(matches!(
            app_actions.recv().await.unwrap(),
            AppAction::SetStatus {
                level: StatusLevel::Error,
                ..
            },
        ));

        let result = plan_tab_actions.try_recv();
        assert!(result.is_err());
    }
}
