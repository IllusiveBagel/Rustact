use ratatui::Frame;
use ratatui::layout::Rect;

use crate::runtime::LayersView;

use super::RenderFn;

pub fn render_layers(frame: &mut Frame<'_>, area: Rect, view: &LayersView, render_child: RenderFn) {
    for layer in &view.layers {
        render_child(frame, area, layer);
    }
}
