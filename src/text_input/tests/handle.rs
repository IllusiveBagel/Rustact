use crate::events::EventBus;
use crate::runtime::{Dispatcher, FormFieldStatus};
use crate::text_input::{TextInputHandle, TextInputs};
use tokio::sync::mpsc;

fn test_dispatcher() -> Dispatcher {
    let (tx, _rx) = mpsc::channel(8);
    let bus = EventBus::new(8);
    Dispatcher::new(tx, bus)
}

#[test]
fn handle_updates_value_cursor_and_status() {
    let dispatcher = test_dispatcher();
    let handle = TextInputHandle::new("field".into(), "hi".into(), dispatcher.clone());
    assert_eq!(handle.id(), "field");
    assert_eq!(handle.value(), "hi");

    handle.set_value("hello");
    assert_eq!(handle.value(), "hello");
    assert_eq!(
        handle.cursor(),
        2,
        "cursor should clamp to previous position"
    );

    handle.set_cursor(2);
    assert_eq!(handle.cursor(), 2);

    assert!(handle.status().is_none());
    handle.set_status(FormFieldStatus::Warning);
    assert_eq!(handle.status(), Some(FormFieldStatus::Warning));
    handle.clear_status();
    assert!(handle.status().is_none());

    TextInputs::unregister_binding(handle.id());
}

#[test]
fn handle_focuses_registered_input() {
    let dispatcher = test_dispatcher();
    let handle = TextInputHandle::new("field.focus".into(), String::new(), dispatcher);
    handle.focus();
    assert!(TextInputs::is_focused(handle.id()));
    TextInputs::unregister_binding(handle.id());
}
