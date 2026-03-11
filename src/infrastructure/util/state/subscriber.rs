use std::sync::Arc;

use tokio::sync::watch::Receiver as WatchReceiver;
use tokio::sync::watch::error::RecvError;

pub struct State<S> {
    subscriber: WatchReceiver<Arc<S>>,
}

impl<T> State<T> {
    #[inline]
    pub(super) fn new(mut subscriber: WatchReceiver<Arc<T>>) -> Self {
        subscriber.mark_changed();
        Self { subscriber }
    }

    #[inline]
    pub fn get(&self) -> Arc<T> {
        Arc::clone(&*self.subscriber.borrow())
    }

    #[inline]
    pub async fn watch(&mut self) -> Result<Arc<T>, RecvError> {
        self.subscriber.changed().await?;
        Ok(Arc::clone(&*self.subscriber.borrow_and_update()))
    }
}

impl<T> Clone for State<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.subscriber.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::infrastructure::util::state::StateSource;

    #[tokio::test]
    async fn test_state_get_and_watch() {
        let (mut source, state) = StateSource::new(1);
        let mut state2 = state.clone();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            assert_eq!(*state2.watch().await.unwrap(), 2);
        });

        source.set(2);

        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(*state.get(), 2);
    }
}
