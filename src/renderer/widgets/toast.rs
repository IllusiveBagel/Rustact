use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::runtime::{ToastLevel, ToastStackView};

pub fn render_toast_stack(frame: &mut Frame<'_>, area: Rect, view: &ToastStackView) {
    if view.toasts.is_empty() {
        return;
    }

    let width = area.width.min(40).max(20);
    let mut cursor_y = area.y + area.height;

    for toast in view.toasts.iter().rev() {
        let height = if toast.body.is_some() { 5 } else { 4 };
        if cursor_y < height {
            break;
        }
        cursor_y -= height;
        let rect = Rect::new(
            area.x + area.width.saturating_sub(width),
            cursor_y,
            width,
            height,
        );
        frame.render_widget(Clear, rect);
        let style = style_for_level(toast.level);
        let block = Block::default().borders(Borders::ALL).style(style);
        frame.render_widget(block.clone(), rect);
        let inner = block.inner(rect);
        let mut lines = vec![Line::from(Span::styled(
            toast.title.clone(),
            style.add_modifier(Modifier::BOLD),
        ))];
        if let Some(body) = &toast.body {
            lines.push(Line::from(body.clone()));
        }
        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }
}

fn style_for_level(level: ToastLevel) -> Style {
    match level {
        ToastLevel::Info => Style::default().fg(Color::Black).bg(Color::Cyan),
        ToastLevel::Success => Style::default().fg(Color::Black).bg(Color::Green),
        ToastLevel::Warning => Style::default().fg(Color::Black).bg(Color::Yellow),
        ToastLevel::Error => Style::default().fg(Color::White).bg(Color::Red),
    }
}
