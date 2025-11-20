use crate::events::{EventBus, FrameworkEvent};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct Dispatcher {
    tx: mpsc::Sender<AppMessage>,
    event_bus: EventBus,
}

impl Dispatcher {
    pub(crate) fn new(tx: mpsc::Sender<AppMessage>, event_bus: EventBus) -> Self {
        Self { tx, event_bus }
    }

    pub fn request_render(&self) {
        let _ = self.tx.try_send(AppMessage::RequestRender);
    }

    pub fn events(&self) -> EventBus {
        self.event_bus.clone()
    }
}

#[derive(Clone, Debug)]
pub(crate) enum AppMessage {
    RequestRender,
    ExternalEvent(FrameworkEvent),
    Shutdown,
}
