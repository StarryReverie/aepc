use std::sync::Arc;

use getset::{CopyGetters, Getters};
use ratatui::widgets::ListState;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::application::query::plan::{
    DynPlanQueryService, PlanDetail, PlanQueryService, QueryPlanError, QueryPlanRequest,
};
use crate::domain::item::model::ItemId;
use crate::domain::recipe::model::Flow;
use crate::infrastructure::util::state::{State, StateManager, StateManagerContext, StateSource};
use crate::ui::state::{AppAction, PlanTabState, StatusLevel};

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters)]
pub struct PlanTreeListState {
    #[getset(get = "pub")]
    plan_detail: Option<PlanDetail>,
    #[getset(get_copy = "pub")]
    list_state: ListState,
    #[getset(get = "pub")]
    last_goal_item_id: Option<ItemId>,
    #[getset(get_copy = "pub")]
    last_flow: Option<Flow>,
}

impl Default for PlanTreeListState {
    fn default() -> Self {
        Self {
            plan_detail: None,
            list_state: ListState::default().with_selected(Some(0)),
            last_goal_item_id: None,
            last_flow: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanTreeListAction {
    SelectNext,
    SelectPrevious,
    SetListState(ListState),
}

#[derive(Debug)]
pub enum PlanTreeListResponse {
    PlanLoaded(Result<PlanDetail, QueryPlanError>),
}

pub struct PlanTreeListStateManager {
    source: StateSource<PlanTreeListState>,
    actions: Receiver<PlanTreeListAction>,
    responses: Receiver<PlanTreeListResponse>,
    reporter: Sender<PlanTreeListResponse>,
    app_requester: Sender<AppAction>,
    plan_tab_state: State<PlanTabState>,
    plan_query_service: Arc<DynPlanQueryService<'static>>,
}

impl PlanTreeListStateManager {
    pub fn context(
        plan_tab_state: State<PlanTabState>,
        app_requester: Sender<AppAction>,
        plan_query_service: Arc<DynPlanQueryService<'static>>,
    ) -> StateManagerContext<Self> {
        let (requester, actions) = mpsc::channel(32);
        let (reporter, responses) = mpsc::channel(32);
        let (source, _) = StateSource::new(PlanTreeListState::default());
        let manager = Self {
            source,
            actions,
            responses,
            reporter,
            app_requester,
            plan_tab_state,
            plan_query_service,
        };
        StateManagerContext::new(manager, requester)
    }

    fn handle_action(&mut self, action: PlanTreeListAction) {
        match action {
            PlanTreeListAction::SelectNext => {
                self.handle_action_select_next();
            }
            PlanTreeListAction::SelectPrevious => {
                self.handle_action_select_previous();
            }
            PlanTreeListAction::SetListState(list_state) => {
                self.handle_action_set_list_state(list_state)
            }
        }
    }

    fn handle_action_select_next(&mut self) {
        let mut state = self.source.get().clone();
        state.list_state.select_next();
        self.source.set(state);
    }

    fn handle_action_select_previous(&mut self) {
        let mut state = self.source.get().clone();
        state.list_state.select_previous();
        self.source.set(state);
    }

    fn handle_action_set_list_state(&mut self, list_state: ListState) {
        self.source.modify(|state| PlanTreeListState {
            list_state,
            ..state.clone()
        });
    }

    fn handle_response(&mut self, response: PlanTreeListResponse) {
        match response {
            PlanTreeListResponse::PlanLoaded(result) => {
                self.handle_response_plan_loaded(result);
            }
        }
    }

    fn handle_response_plan_loaded(&mut self, result: Result<PlanDetail, QueryPlanError>) {
        match result {
            Ok(plan_detail) => {
                self.source.modify(|state| PlanTreeListState {
                    plan_detail: Some(plan_detail),
                    list_state: ListState::default().with_selected(Some(0)),
                    ..state.clone()
                });
                let _ = self.app_requester.try_send(AppAction::SetStatus {
                    text: "Plan generated successfully".to_string(),
                    level: StatusLevel::Info,
                });
            }
            Err(err) => {
                let _ = self.app_requester.try_send(AppAction::SetStatus {
                    text: err.to_string(),
                    level: StatusLevel::Error,
                });
            }
        }
    }

    fn handle_plan_tab_state_changed(&mut self) {
        let plan_tab_state = self.plan_tab_state.get();
        let goal_item = plan_tab_state.expected_goal_item();
        let flow = plan_tab_state.expected_goal_flow();
        let goal_item_id = goal_item.as_ref().map(|i| i.id());

        let changed = {
            let current = self.source.get();
            let goal_changed = match (current.last_goal_item_id(), goal_item_id) {
                (Some(cached), Some(id)) => cached != id,
                (None, None) => false,
                _ => true,
            };
            let flow_changed = current.last_flow() != flow;
            goal_changed || flow_changed
        };
        if !changed {
            return;
        }

        self.source.modify(|state| PlanTreeListState {
            last_goal_item_id: goal_item_id.cloned(),
            last_flow: flow,
            ..state.clone()
        });

        if let (Some(goal_item), Some(flow)) = (goal_item, flow) {
            let request = QueryPlanRequest {
                goal: goal_item.id().clone(),
                expected_flow: flow,
            };
            let plan_query_service = self.plan_query_service.clone();
            let reporter = self.reporter.clone();
            tokio::spawn(async move {
                let res = plan_query_service.query_plan(request).await;
                let _ = reporter
                    .send(PlanTreeListResponse::PlanLoaded(res.map(|r| r.plan)))
                    .await;
            });
        }
    }
}

impl StateManager for PlanTreeListStateManager {
    type State = PlanTreeListState;

    type Action = PlanTreeListAction;

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
                    Some(response) = self.responses.recv() => {
                        self.handle_response(response);
                    }
                    Ok(_) = self.plan_tab_state.watch() => {
                        self.handle_plan_tab_state_changed();
                    }
                }
            }
        });
    }
}
