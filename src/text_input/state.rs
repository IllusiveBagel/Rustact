use std::sync::Arc;

use crate::runtime::FormFieldStatus;

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

#[derive(Clone, Debug)]
pub struct TextInputSnapshot {
    pub id: Arc<String>,
    pub value: String,
    pub cursor: usize,
    pub status: Option<FormFieldStatus>,
}
