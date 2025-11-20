use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::runtime::TextView;

pub fn render_text(frame: &mut Frame<'_>, area: Rect, view: &TextView) {
    let style = Style::default().fg(view.color.unwrap_or(Color::White));
    let widget = Paragraph::new(Line::from(view.content.clone())).style(style);
    frame.render_widget(widget, area);
}
