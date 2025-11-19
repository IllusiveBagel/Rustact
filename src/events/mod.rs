use std::time::Duration;

use crossterm::event::{
    Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub enum FrameworkEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
}

#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<FrameworkEvent>,
}

impl EventBus {
    pub fn new(buffer: usize) -> Self {
        let (tx, _) = broadcast::channel(buffer);
        Self { tx }
    }

    pub fn publish(&self, event: FrameworkEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<FrameworkEvent> {
        self.tx.subscribe()
    }
}

pub fn map_terminal_event(event: CrosstermEvent) -> Option<FrameworkEvent> {
    match event {
        CrosstermEvent::Key(key) => Some(FrameworkEvent::Key(key)),
        CrosstermEvent::Mouse(mouse) => Some(FrameworkEvent::Mouse(mouse)),
        CrosstermEvent::Resize(cols, rows) => Some(FrameworkEvent::Resize(cols, rows)),
        CrosstermEvent::FocusGained | CrosstermEvent::FocusLost | CrosstermEvent::Paste(_) => None,
    }
}

pub fn is_ctrl_c(event: &FrameworkEvent) -> bool {
    match event {
        FrameworkEvent::Key(key) => match key.code {
            KeyCode::Char('c') | KeyCode::Char('C') => {
                key.modifiers.contains(KeyModifiers::CONTROL)
            }
            _ => false,
        },
        _ => false,
    }
}

pub fn is_mouse_click(event: &FrameworkEvent, button: MouseButton) -> bool {
    matches!(
        event,
        FrameworkEvent::Mouse(mouse)
            if matches!(mouse.kind, MouseEventKind::Down(btn) if btn == button)
    )
}

pub fn mouse_scroll_delta(event: &FrameworkEvent) -> i32 {
    if let FrameworkEvent::Mouse(mouse) = event {
        match mouse.kind {
            MouseEventKind::ScrollUp => 1,
            MouseEventKind::ScrollDown => -1,
            _ => 0,
        }
    } else {
        0
    }
}

pub fn mouse_position(event: &FrameworkEvent) -> Option<(u16, u16)> {
    if let FrameworkEvent::Mouse(mouse) = event {
        Some((mouse.column, mouse.row))
    } else {
        None
    }
}

pub const DEFAULT_TICK_RATE: Duration = Duration::from_millis(250);
