use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear};

use crate::runtime::ModalView;

use super::RenderFn;

pub fn render_modal(frame: &mut Frame<'_>, area: Rect, view: &ModalView, render_child: RenderFn) {
    let width = desired_dimension(area.width, view.width, 8, 20);
    let height = desired_dimension(area.height, view.height, 6, 6);
    let origin_x = area.x + (area.width.saturating_sub(width)) / 2;
    let origin_y = area.y + (area.height.saturating_sub(height)) / 2;
    let modal_area = Rect::new(origin_x, origin_y, width, height);

    frame.render_widget(Clear, modal_area);
    let block = Block::default()
        .title(view.title.clone().unwrap_or_else(|| "Modal".to_string()))
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    frame.render_widget(block.clone(), modal_area);
    let inner = block.inner(modal_area);
    render_child(frame, inner, view.content.as_ref());
}

fn desired_dimension(total: u16, desired: Option<u16>, padding: u16, minimum: u16) -> u16 {
    let fallback = total.saturating_sub(padding).max(minimum);
    desired.unwrap_or(fallback).min(total).max(minimum)
}
