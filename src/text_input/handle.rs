use std::fmt;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::runtime::{Dispatcher, FormFieldStatus};

use super::registry::TextInputs;
use super::state::{TextInputSnapshot, TextInputState};

#[derive(Clone)]
pub struct TextInputHandle {
    id: Arc<String>,
    state: Arc<Mutex<TextInputState>>,
    dispatcher: Dispatcher,
}

impl TextInputHandle {
    pub(crate) fn new(id: String, initial: String, dispatcher: Dispatcher) -> Self {
        let state = Arc::new(Mutex::new(TextInputState::new(initial)));
        TextInputs::register_binding(&id, state.clone());
        Self {
            id: Arc::new(id),
            state,
            dispatcher,
        }
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn value(&self) -> String {
        self.state.lock().value.clone()
    }

    pub fn set_value(&self, next: impl Into<String>) {
        let mut guard = self.state.lock();
        guard.value = next.into();
        guard.cursor = guard.value.len().min(guard.cursor);
        self.dispatcher.request_render();
    }

    pub fn cursor(&self) -> usize {
        self.state.lock().cursor
    }

    pub fn set_cursor(&self, cursor: usize) {
        let mut guard = self.state.lock();
        guard.cursor = cursor.min(guard.value.len());
        self.dispatcher.request_render();
    }

    pub fn snapshot(&self) -> TextInputSnapshot {
        let guard = self.state.lock();
        TextInputSnapshot {
            id: self.id.clone(),
            value: guard.value.clone(),
            cursor: guard.cursor,
            status: guard.status,
        }
    }

    pub fn status(&self) -> Option<FormFieldStatus> {
        self.state.lock().status
    }

    pub fn set_status(&self, status: FormFieldStatus) {
        let mut guard = self.state.lock();
        if guard.status == Some(status) {
            return;
        }
        guard.status = Some(status);
        self.dispatcher.request_render();
    }

    pub fn clear_status(&self) {
        let mut guard = self.state.lock();
        if guard.status.take().is_some() {
            self.dispatcher.request_render();
        }
    }

    pub fn focus(&self) {
        TextInputs::focus(Some(self.id()), &self.dispatcher);
    }
}

impl fmt::Debug for TextInputHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextInputHandle")
            .field("id", &self.id)
            .finish()
    }
}
