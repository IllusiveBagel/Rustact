use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind};
use parking_lot::{Mutex, RwLock};
use std::sync::OnceLock;

use crate::events::{FrameworkEvent, mouse_position};
use crate::interactions::Hitbox;
use crate::runtime::{Dispatcher, FormFieldStatus};

#[derive(Clone, Debug)]
pub struct TextInputState {
    pub value: String,
    pub cursor: usize,
    pub status: Option<FormFieldStatus>,
}

impl TextInputState {
    pub fn new(initial: String) -> Self {
        let cursor = initial.len();
        Self {
            value: initial,
            cursor,
            status: None,
        }
    }
}

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

#[derive(Clone, Debug)]
pub struct TextInputSnapshot {
    pub id: Arc<String>,
    pub value: String,
    pub cursor: usize,
    pub status: Option<FormFieldStatus>,
}

struct TextInputRegistry {
    bindings: RwLock<HashMap<String, Arc<Mutex<TextInputState>>>>,
    order: RwLock<Vec<String>>,
    hitboxes: RwLock<HashMap<String, Hitbox>>,
    focused: Mutex<Option<String>>,
    cursor_visible: Mutex<bool>,
}

impl TextInputRegistry {
    fn new() -> Self {
        Self {
            bindings: RwLock::new(HashMap::new()),
            order: RwLock::new(Vec::new()),
            hitboxes: RwLock::new(HashMap::new()),
            focused: Mutex::new(None),
            cursor_visible: Mutex::new(true),
        }
    }

    fn singleton() -> &'static Self {
        static REGISTRY: OnceLock<TextInputRegistry> = OnceLock::new();
        REGISTRY.get_or_init(Self::new)
    }

    fn register_binding(id: &str, state: Arc<Mutex<TextInputState>>) {
        let registry = Self::singleton();
        registry.bindings.write().insert(id.to_string(), state);
        let mut order = registry.order.write();
        if !order.iter().any(|existing| existing == id) {
            order.push(id.to_string());
        }
    }

    fn unregister_binding(id: &str) {
        let registry = Self::singleton();
        registry.bindings.write().remove(id);
        registry.hitboxes.write().remove(id);
        let mut order = registry.order.write();
        if let Some(index) = order.iter().position(|existing| existing == id) {
            order.remove(index);
        }
        let mut focused = registry.focused.lock();
        if focused.as_deref() == Some(id) {
            *focused = None;
        }
    }

    fn register_hitbox(id: &str, hitbox: Hitbox) {
        let registry = Self::singleton();
        registry.hitboxes.write().insert(id.to_string(), hitbox);
    }

    fn reset_hitboxes() {
        let registry = Self::singleton();
        registry.hitboxes.write().clear();
    }

    fn hitbox_contains(&self, column: u16, row: u16) -> Option<String> {
        self.hitboxes.read().iter().find_map(|(id, hitbox)| {
            if column >= hitbox.x
                && column < hitbox.x.saturating_add(hitbox.width)
                && row >= hitbox.y
                && row < hitbox.y.saturating_add(hitbox.height)
            {
                Some(id.clone())
            } else {
                None
            }
        })
    }

    fn focus(&self, id: Option<&str>, dispatcher: &Dispatcher) {
        let mut guard = self.focused.lock();
        let next = id.map(|value| value.to_string());
        if guard.as_ref() != next.as_ref() {
            *guard = next;
            *self.cursor_visible.lock() = true;
            dispatcher.request_render();
        }
    }

    fn focused(&self) -> Option<String> {
        self.focused.lock().clone()
    }

    fn binding(&self, id: &str) -> Option<Arc<Mutex<TextInputState>>> {
        self.bindings.read().get(id).cloned()
    }

    fn focus_next(&self, reverse: bool, dispatcher: &Dispatcher) {
        let order = self.order.read();
        if order.is_empty() {
            return;
        }
        let current = self.focused();
        let next_index = if current.is_none() {
            if reverse {
                order.len().saturating_sub(1)
            } else {
                0
            }
        } else {
            let current_index = current
                .as_ref()
                .and_then(|id| order.iter().position(|existing| existing == id))
                .unwrap_or(0);
            if reverse {
                if current_index == 0 {
                    order.len() - 1
                } else {
                    current_index - 1
                }
            } else {
                (current_index + 1) % order.len()
            }
        };
        if let Some(next_id) = order.get(next_index) {
            self.focus(Some(next_id), dispatcher);
        }
    }

    fn cursor_visible(&self, id: &str) -> bool {
        if self.focused().as_deref() != Some(id) {
            return false;
        }
        *self.cursor_visible.lock()
    }

    fn tick(&self, dispatcher: &Dispatcher) {
        if self.focused().is_none() {
            let mut visible = self.cursor_visible.lock();
            if *visible {
                *visible = false;
                dispatcher.request_render();
            }
            return;
        }
        {
            let mut visible = self.cursor_visible.lock();
            *visible = !*visible;
        }
        dispatcher.request_render();
    }
}

pub struct TextInputs;

impl TextInputs {
    pub(crate) fn register_binding(id: &str, state: Arc<Mutex<TextInputState>>) {
        TextInputRegistry::register_binding(id, state);
    }

    pub(crate) fn unregister_binding(id: &str) {
        TextInputRegistry::unregister_binding(id);
    }

    pub fn register_hitbox(id: &str, hitbox: Hitbox) {
        TextInputRegistry::register_hitbox(id, hitbox);
    }

    pub fn reset_hitboxes() {
        TextInputRegistry::reset_hitboxes();
    }

    pub fn is_focused(id: &str) -> bool {
        let registry = TextInputRegistry::singleton();
        registry.focused().as_deref() == Some(id)
    }

    pub fn cursor_visible(id: &str) -> bool {
        let registry = TextInputRegistry::singleton();
        registry.cursor_visible(id)
    }

    pub fn focus(id: Option<&str>, dispatcher: &Dispatcher) {
        let registry = TextInputRegistry::singleton();
        registry.focus(id, dispatcher);
    }

    pub fn handle_event(event: &FrameworkEvent, dispatcher: &Dispatcher) {
        match event {
            FrameworkEvent::Mouse(mouse)
                if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) =>
            {
                let registry = TextInputRegistry::singleton();
                if let Some((col, row)) = mouse_position(event) {
                    if let Some(id) = registry.hitbox_contains(col, row) {
                        registry.focus(Some(&id), dispatcher);
                    } else {
                        registry.focus(None, dispatcher);
                    }
                }
            }
            FrameworkEvent::Key(key) => Self::handle_key(key, dispatcher),
            FrameworkEvent::Tick => {
                let registry = TextInputRegistry::singleton();
                registry.tick(dispatcher);
            }
            _ => {}
        }
    }

    fn handle_key(key: &KeyEvent, dispatcher: &Dispatcher) {
        let registry = TextInputRegistry::singleton();
        if matches!(key.code, KeyCode::Tab) {
            let reverse = key.modifiers.contains(KeyModifiers::SHIFT);
            registry.focus_next(reverse, dispatcher);
            return;
        }
        let Some(focused_id) = registry.focused() else {
            return;
        };
        if let Some(binding) = registry.binding(&focused_id) {
            let mut state = binding.lock();
            match key.code {
                KeyCode::Char(c) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        || key.modifiers.contains(KeyModifiers::ALT)
                    {
                        return;
                    }
                    let cursor = state.cursor;
                    state.value.insert(cursor, c);
                    state.cursor = cursor + c.len_utf8();
                }
                KeyCode::Backspace => {
                    if state.cursor > 0 {
                        let cursor = state.cursor;
                        let prev = prev_char_boundary(&state.value, cursor);
                        if let Some(prev_index) = prev {
                            state.value.replace_range(prev_index..cursor, "");
                            state.cursor = prev_index;
                        }
                    }
                }
                KeyCode::Delete => {
                    if state.cursor < state.value.len() {
                        let cursor = state.cursor;
                        if let Some(next_index) = next_char_boundary(&state.value, cursor) {
                            state.value.replace_range(cursor..next_index, "");
                        }
                    }
                }
                KeyCode::Left => {
                    if let Some(prev) = prev_char_boundary(&state.value, state.cursor) {
                        state.cursor = prev;
                    }
                }
                KeyCode::Right => {
                    if let Some(next) = next_char_boundary(&state.value, state.cursor) {
                        state.cursor = next;
                    }
                }
                KeyCode::Home => state.cursor = 0,
                KeyCode::End => state.cursor = state.value.len(),
                KeyCode::Esc => {
                    registry.focus(None, dispatcher);
                    return;
                }
                _ => return,
            }
            dispatcher.request_render();
        }
    }
}

fn prev_char_boundary(value: &str, index: usize) -> Option<usize> {
    value[..index].char_indices().last().map(|(idx, _)| idx)
}

fn next_char_boundary(value: &str, index: usize) -> Option<usize> {
    if index >= value.len() {
        return None;
    }
    let mut chars = value[index..].chars();
    let ch = chars.next()?;
    Some(index + ch.len_utf8())
}
