use tokio::sync::mpsc;

use super::super::dispatcher::{AppMessage, Dispatcher};
use crate::events::{EventBus, FrameworkEvent};

#[test]
fn request_render_queues_app_message() {
    let (tx, mut rx) = mpsc::channel(1);
    let dispatcher = Dispatcher::new(tx, EventBus::new(2));

    dispatcher.request_render();

    match rx.try_recv().expect("render request enqueued") {
        AppMessage::RequestRender => {}
        other => panic!("unexpected message: {other:?}"),
    }
}

#[test]
fn events_accessor_returns_shared_bus() {
    let (tx, _) = mpsc::channel(1);
    let bus = EventBus::new(2);
    let dispatcher = Dispatcher::new(tx, bus.clone());
    let mut subscriber = dispatcher.events().subscribe();

    bus.publish(FrameworkEvent::Tick);

    match subscriber.try_recv().expect("event broadcasted") {
        FrameworkEvent::Tick => {}
        other => panic!("unexpected event: {other:?}"),
    }
}
