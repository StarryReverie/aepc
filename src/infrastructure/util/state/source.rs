use std::ops::Deref;
use std::sync::Arc;

use tokio::sync::watch::{self, Receiver as WatchReceiver, Ref, Sender as WatchSender};

use super::State;

pub struct StateSource<T> {
    state: WatchReceiver<Arc<T>>,
    publisher: WatchSender<Arc<T>>,
}

impl<T> StateSource<T> {
    #[inline]
    pub fn new(state: T) -> (Self, State<T>) {
        let (publisher, state) = watch::channel(Arc::new(state));
        let source = Self { state, publisher };
        let subscriber = source.subscribe();
        (source, subscriber)
    }

    #[inline]
    pub fn subscribe(&self) -> State<T> {
        State::new(self.publisher.subscribe())
    }

    #[inline]
    pub fn get(&self) -> StateRef<'_, T> {
        StateRef(self.state.borrow())
    }

    #[inline]
    pub fn set(&mut self, state: T) {
        let _ = self.publisher.send(Arc::new(state));
    }

    #[inline]
    pub fn modify(&mut self, func: impl FnOnce(&T) -> T) {
        let state = func(&*self.get());
        self.set(state)
    }
}

impl<T> From<T> for StateSource<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value).0
    }
}

pub struct StateRef<'a, T>(Ref<'a, Arc<T>>);

impl<'a, T> Deref for StateRef<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_source_set_and_modify() {
        let (mut source, _) = StateSource::new(1);
        assert_eq!(*source.get(), 1);

        source.set(2);
        assert_eq!(*source.get(), 2);

        source.modify(|x| x + 1);
        assert_eq!(*source.get(), 3);
    }
}
