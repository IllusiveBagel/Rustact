use std::sync::Arc;

use crate::events::{EventBus, FrameworkEvent};
use crate::styles::Stylesheet;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tracing::trace;

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
        match self.tx.try_send(AppMessage::RequestRender) {
            Ok(_) => trace!("render request queued"),
            Err(TrySendError::Full(_)) => {
                trace!("render request dropped because channel is full")
            }
            Err(TrySendError::Closed(_)) => trace!("render request dropped because channel closed"),
        }
    }

    pub fn events(&self) -> EventBus {
        self.event_bus.clone()
    }
}

#[derive(Clone, Debug)]
pub enum AppMessage {
    RequestRender,
    ExternalEvent(FrameworkEvent),
    Shutdown,
    StylesheetUpdated(Arc<Stylesheet>),
}
