use std::sync::Arc;

use parking_lot::Mutex;

use crate::runtime::Dispatcher;

pub type ReducerFn<S, A> = dyn Fn(&mut S, A) + Send + Sync + 'static;

#[derive(Clone)]
pub struct StateHandle<T: Send + 'static> {
    pub(crate) shared: Arc<Mutex<T>>,
    dispatcher: Dispatcher,
}

impl<T: Send + 'static> StateHandle<T> {
    pub(crate) fn new(shared: Arc<Mutex<T>>, dispatcher: Dispatcher) -> Self {
        Self { shared, dispatcher }
    }

    pub fn set(&self, next: T) {
        *self.shared.lock() = next;
        self.dispatcher.request_render();
    }

    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        f(&mut *self.shared.lock());
        self.dispatcher.request_render();
    }
}

#[derive(Clone)]
pub struct ReducerDispatch<S: Send + 'static, A: Send + 'static> {
    pub(crate) shared: Arc<Mutex<S>>,
    pub(crate) reducer: Arc<ReducerFn<S, A>>,
    dispatcher: Dispatcher,
}

impl<S: Send + 'static, A: Send + 'static> ReducerDispatch<S, A> {
    pub(crate) fn new(
        shared: Arc<Mutex<S>>,
        reducer: Arc<ReducerFn<S, A>>,
        dispatcher: Dispatcher,
    ) -> Self {
        Self {
            shared,
            reducer,
            dispatcher,
        }
    }

    pub fn dispatch(&self, action: A) {
        {
            let mut state = self.shared.lock();
            (self.reducer)(&mut state, action);
        }
        self.dispatcher.request_render();
    }

    pub fn with_state<R>(&self, f: impl FnOnce(&S) -> R) -> R {
        let state = self.shared.lock();
        f(&state)
    }
}

#[derive(Clone)]
pub struct RefHandle<T: Send + 'static> {
    shared: Arc<Mutex<T>>,
}

impl<T: Send + 'static> RefHandle<T> {
    pub(crate) fn new(shared: Arc<Mutex<T>>) -> Self {
        Self { shared }
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let value = self.shared.lock();
        f(&value)
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut value = self.shared.lock();
        f(&mut value)
    }

    pub fn set(&self, next: T) {
        *self.shared.lock() = next;
    }
}
