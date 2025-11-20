use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::runtime::{FlexDirection, FlexView};

use super::RenderFn;

pub fn render_flex(frame: &mut Frame<'_>, area: Rect, view: &FlexView, render_child: RenderFn) {
    if view.children.is_empty() {
        return;
    }

    let chunk_count = view.children.len();
    let constraints = vec![Constraint::Ratio(1, chunk_count as u32); chunk_count];
    let layout = Layout::default()
        .direction(Direction::from(view.direction))
        .constraints(constraints);
    let chunks = layout.split(area);
    for (child, rect) in view.children.iter().zip(chunks.iter()) {
        render_child(frame, *rect, child);
    }
}

impl From<FlexDirection> for Direction {
    fn from(value: FlexDirection) -> Self {
        match value {
            FlexDirection::Row => Direction::Horizontal,
            FlexDirection::Column => Direction::Vertical,
        }
    }
}
