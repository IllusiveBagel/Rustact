use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders};

use crate::runtime::BlockView;

use super::RenderFn;

pub fn render_block(frame: &mut Frame<'_>, area: Rect, view: &BlockView, render_child: RenderFn) {
    let mut widget = Block::default().borders(Borders::ALL);
    if let Some(title) = &view.title {
        widget = widget.title(title.as_str());
    }
    frame.render_widget(widget.clone(), area);

    if let Some(child) = view.child.as_ref() {
        let inner = widget.inner(area);
        render_child(frame, inner, child);
    }
}
