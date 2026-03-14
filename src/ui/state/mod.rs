mod app;
mod goal_flow_input;
mod plan_tab;

pub use app::{AppAction, AppState, AppStateManager, StatusLevel};
pub use goal_flow_input::create_goal_flow_on_confirm;
pub use plan_tab::{PlanTabAction, PlanTabFocus, PlanTabState, PlanTabStateManager};

use tokio::sync::mpsc::Sender;

use crate::infrastructure::util::state::State;

pub trait StateManager: Sized {
    type State;

    type Action;

    fn state(&self) -> State<Self::State>;

    fn run(self);
}

pub struct StateManagerContext<M>
where
    M: StateManager,
{
    manager: M,
    requester: Sender<M::Action>,
}

impl<M> StateManagerContext<M>
where
    M: StateManager,
{
    pub fn new(manager: M, requester: Sender<M::Action>) -> Self {
        Self { manager, requester }
    }

    pub fn state(&self) -> State<M::State> {
        self.manager.state()
    }

    pub fn requester(&self) -> Sender<M::Action> {
        self.requester.clone()
    }

    pub fn run(self) {
        self.manager.run();
    }
}
