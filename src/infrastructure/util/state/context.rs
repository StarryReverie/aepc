use tokio::sync::mpsc::Sender;

use super::{State, StateManager};

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
