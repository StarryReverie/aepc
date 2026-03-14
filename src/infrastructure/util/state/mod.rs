mod context;
mod source;
mod subscriber;

pub use context::StateManagerContext;
pub use source::{StateRef, StateSource};
pub use subscriber::State;

pub trait StateManager: Sized {
    type State;

    type Action;

    fn state(&self) -> State<Self::State>;

    fn run(self);
}
