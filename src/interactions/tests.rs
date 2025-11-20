use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use crate::events::FrameworkEvent;

use super::{Hitbox, is_button_click, register_button_hitbox, reset_button_hitboxes};

#[test]
fn button_click_detects_coordinates_within_hitbox() {
    reset_button_hitboxes();
    register_button_hitbox(
        "submit",
        Hitbox {
            x: 10,
            y: 5,
            width: 4,
            height: 2,
        },
    );
    let event = FrameworkEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 11,
        row: 6,
        modifiers: KeyModifiers::NONE,
    });

    assert!(is_button_click(&event, "submit"));
}

#[test]
fn reset_clears_hitboxes_and_prevents_future_matches() {
    reset_button_hitboxes();
    register_button_hitbox(
        "danger",
        Hitbox {
            x: 0,
            y: 0,
            width: 2,
            height: 1,
        },
    );
    let click = FrameworkEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 1,
        row: 0,
        modifiers: KeyModifiers::NONE,
    });
    assert!(is_button_click(&click, "danger"));

    reset_button_hitboxes();
    assert!(!is_button_click(&click, "danger"));
}
