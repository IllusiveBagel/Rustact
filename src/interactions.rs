use std::collections::HashMap;
use std::sync::OnceLock;

use parking_lot::RwLock;

use crate::events::{FrameworkEvent, mouse_position};
use crossterm::event::{MouseButton, MouseEventKind};

#[derive(Clone, Copy, Debug, Default)]
pub struct Hitbox {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

pub struct ButtonRegistry {
    hitboxes: RwLock<HashMap<String, Hitbox>>,
}

impl ButtonRegistry {
    fn new() -> Self {
        Self {
            hitboxes: RwLock::new(HashMap::new()),
        }
    }

    fn global() -> &'static Self {
        static REGISTRY: OnceLock<ButtonRegistry> = OnceLock::new();
        REGISTRY.get_or_init(Self::new)
    }

    pub fn reset() {
        let registry = Self::global();
        registry.hitboxes.write().clear();
    }

    pub fn record(id: &str, hitbox: Hitbox) {
        let registry = Self::global();
        registry.hitboxes.write().insert(id.to_string(), hitbox);
    }

    pub fn contains(id: &str, column: u16, row: u16) -> bool {
        let registry = Self::global();
        let boxes = registry.hitboxes.read();
        if let Some(hitbox) = boxes.get(id) {
            return column >= hitbox.x
                && column < hitbox.x.saturating_add(hitbox.width)
                && row >= hitbox.y
                && row < hitbox.y.saturating_add(hitbox.height);
        }
        false
    }
}

pub(crate) fn register_button_hitbox(id: &str, hitbox: Hitbox) {
    ButtonRegistry::record(id, hitbox);
}

pub(crate) fn reset_button_hitboxes() {
    ButtonRegistry::reset();
}

pub fn is_button_click(event: &FrameworkEvent, button_id: &str) -> bool {
    if let FrameworkEvent::Mouse(mouse) = event {
        if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            if let Some((column, row)) = mouse_position(event) {
                return ButtonRegistry::contains(button_id, column, row);
            }
        }
    }
    false
}

#[cfg(test)]
mod tests;
