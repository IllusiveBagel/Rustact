use super::*;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers, MouseButton,
                       MouseEvent, MouseEventKind};

#[test]
fn map_terminal_event_converts_supported_inputs() {
    let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let mouse_event = MouseEvent {
        kind: MouseEventKind::Moved,
        column: 10,
        row: 5,
        modifiers: KeyModifiers::NONE,
    };
    let resize_event = CrosstermEvent::Resize(80, 24);

    assert!(matches!(
        map_terminal_event(CrosstermEvent::Key(key_event)),
        Some(FrameworkEvent::Key(_))
    ));
    assert!(matches!(
        map_terminal_event(CrosstermEvent::Mouse(mouse_event)),
        Some(FrameworkEvent::Mouse(_))
    ));
    assert!(matches!(
        map_terminal_event(resize_event),
        Some(FrameworkEvent::Resize(80, 24))
    ));
    assert!(map_terminal_event(CrosstermEvent::FocusLost).is_none());
}

#[test]
fn ctrl_c_and_mouse_helpers_behave_as_expected() {
    let ctrl_c = FrameworkEvent::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    let plain_c = FrameworkEvent::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
    assert!(is_ctrl_c(&ctrl_c));
    assert!(!is_ctrl_c(&plain_c));

    let mouse_click = FrameworkEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Right),
        column: 3,
        row: 4,
        modifiers: KeyModifiers::NONE,
    });
    assert!(is_mouse_click(&mouse_click, MouseButton::Right));
    assert!(!is_mouse_click(&mouse_click, MouseButton::Left));

    let scroll_up = FrameworkEvent::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column: 0,
        row: 0,
        modifiers: KeyModifiers::NONE,
    });
    let scroll_down = FrameworkEvent::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 0,
        row: 0,
        modifiers: KeyModifiers::NONE,
    });
    assert_eq!(mouse_scroll_delta(&scroll_up), 1);
    assert_eq!(mouse_scroll_delta(&scroll_down), -1);
    assert_eq!(mouse_scroll_delta(&plain_c), 0);

    assert_eq!(mouse_position(&mouse_click), Some((3, 4)));
    assert_eq!(mouse_position(&plain_c), None);
}

#[test]
fn event_bus_publish_delivers_to_subscribers() {
    let bus = EventBus::new(4);
    let mut rx = bus.subscribe();
    bus.publish(FrameworkEvent::Tick);
    match rx.try_recv().expect("event delivered") {
        FrameworkEvent::Tick => {}
        other => panic!("unexpected event: {other:?}"),
    }
}
