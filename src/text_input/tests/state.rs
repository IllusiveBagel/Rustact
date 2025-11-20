use std::sync::Arc;

use crate::runtime::FormFieldStatus;
use crate::text_input::{TextInputSnapshot, TextInputState};

#[test]
fn new_state_places_cursor_at_end() {
    let state = TextInputState::new("hello".into());
    assert_eq!(state.value, "hello");
    assert_eq!(state.cursor, 5);
    assert!(state.status.is_none());
}

#[test]
fn snapshot_copies_runtime_values() {
    let mut base = TextInputState::new("abc".into());
    base.cursor = 1;
    base.status = Some(FormFieldStatus::Error);
    let id = Arc::new("input#1".to_string());
    let snapshot = TextInputSnapshot {
        id: id.clone(),
        value: base.value.clone(),
        cursor: base.cursor,
        status: base.status,
    };
    assert!(Arc::ptr_eq(&snapshot.id, &id));
    assert_eq!(snapshot.value, "abc");
    assert_eq!(snapshot.cursor, 1);
    assert_eq!(snapshot.status, base.status);
}
