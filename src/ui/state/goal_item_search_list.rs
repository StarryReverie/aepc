use std::sync::Arc;

use getset::{CopyGetters, Getters};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::application::query::item::{DynItemQueryService, ItemQueryService, SearchItemsError};
use crate::domain::item::model::Item;
use crate::infrastructure::util::state::{State, StateManager, StateManagerContext, StateSource};
use crate::ui::state::{AppAction, GoalItemSearchState, PlanTabAction, StatusLevel};

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters)]
pub struct GoalItemSearchListState {
    #[getset(get = "pub")]
    filtered_items: Vec<Item>,
    #[getset(get_copy = "pub")]
    selected_index: Option<usize>,
}

impl Default for GoalItemSearchListState {
    fn default() -> Self {
        Self {
            filtered_items: Vec::new(),
            selected_index: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoalItemSearchListAction {
    SelectNext,
    SelectPrevious,
    ConfirmSelection,
}

#[derive(Debug)]
pub enum GoalItemSearchListResponse {
    ItemsLoaded(Result<Vec<Item>, SearchItemsError>),
}

pub struct GoalItemSearchListStateManager {
    source: StateSource<GoalItemSearchListState>,
    actions: Receiver<GoalItemSearchListAction>,
    responses: Receiver<GoalItemSearchListResponse>,
    reporter: Sender<GoalItemSearchListResponse>,
    plan_tab_requester: Sender<PlanTabAction>,
    app_requester: Sender<AppAction>,
    goal_item_search_state: State<GoalItemSearchState>,
    item_query_service: Arc<DynItemQueryService<'static>>,
}

impl GoalItemSearchListStateManager {
    pub fn context(
        goal_item_search_state: State<GoalItemSearchState>,
        plan_tab_requester: Sender<PlanTabAction>,
        app_requester: Sender<AppAction>,
        item_query_service: Arc<DynItemQueryService<'static>>,
    ) -> StateManagerContext<Self> {
        let (requester, actions) = mpsc::channel(32);
        let (reporter, responses) = mpsc::channel(32);
        let (source, _) = StateSource::new(GoalItemSearchListState::default());
        let manager = Self {
            source,
            actions,
            responses,
            reporter,
            plan_tab_requester,
            app_requester,
            goal_item_search_state,
            item_query_service,
        };
        StateManagerContext::new(manager, requester)
    }
}

impl GoalItemSearchListStateManager {
    fn handle_action(&mut self, action: GoalItemSearchListAction) {
        match action {
            GoalItemSearchListAction::SelectNext => {
                self.handle_action_select_next();
            }
            GoalItemSearchListAction::SelectPrevious => {
                self.handle_action_select_previous();
            }
            GoalItemSearchListAction::ConfirmSelection => {
                self.handle_action_confirm_selection();
            }
        }
    }

    fn handle_action_select_next(&mut self) {
        self.source.modify(|state| {
            let len = state.filtered_items().len();
            let selected_index = match state.selected_index() {
                None if len > 0 => Some(0),
                Some(idx) if idx < len - 1 => Some(idx + 1),
                otherwise => otherwise,
            };
            GoalItemSearchListState {
                selected_index,
                ..state.clone()
            }
        });
    }

    fn handle_action_select_previous(&mut self) {
        self.source.modify(|state| {
            let len = state.filtered_items().len();
            let selected_index = match state.selected_index() {
                None if len > 0 => Some(len - 1),
                Some(idx) if idx > 0 => Some(idx - 1),
                otherwise => otherwise,
            };
            GoalItemSearchListState {
                selected_index,
                ..state.clone()
            }
        });
    }

    fn handle_action_confirm_selection(&mut self) {
        let state = self.source.get();
        if let Some(index) = state.selected_index()
            && let Some(item) = state.filtered_items().get(index)
        {
            let _ = self
                .plan_tab_requester
                .try_send(PlanTabAction::UpdateExpectedGoalItem(item.clone()));
            let _ = self
                .plan_tab_requester
                .try_send(PlanTabAction::SwitchFocusToNext);
        }
    }

    fn handle_response(&mut self, response: GoalItemSearchListResponse) {
        match response {
            GoalItemSearchListResponse::ItemsLoaded(filtered_items) => {
                self.handle_response_items_loaded(filtered_items);
            }
        }
    }

    fn handle_response_items_loaded(
        &mut self,
        filtered_items: Result<Vec<Item>, SearchItemsError>,
    ) {
        match filtered_items {
            Ok(filtered_items) => {
                self.source.modify(|state| {
                    let selected_item_id = state.selected_index().and_then(|idx| {
                        state
                            .filtered_items()
                            .get(idx)
                            .map(|item| item.id().clone())
                    });
                    let selected_index = selected_item_id
                        .and_then(|idx| filtered_items.iter().position(|item| item.id() == &idx));
                    GoalItemSearchListState {
                        filtered_items,
                        selected_index,
                        ..state.clone()
                    }
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

    fn handle_goal_item_search_state_changed(&mut self) {
        let pattern = self
            .goal_item_search_state
            .get()
            .item_name_pattern()
            .clone();
        let item_query_service = self.item_query_service.clone();
        let reporter = self.reporter.clone();

        tokio::spawn(async move {
            let res = item_query_service.search_items(&pattern).await;
            let _ = reporter
                .send(GoalItemSearchListResponse::ItemsLoaded(res))
                .await;
        });
    }
}

impl StateManager for GoalItemSearchListStateManager {
    type State = GoalItemSearchListState;

    type Action = GoalItemSearchListAction;

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
                    Ok(_) = self.goal_item_search_state.watch() => {
                        self.handle_goal_item_search_state_changed();
                    }
                }
            }
        });
    }
}
