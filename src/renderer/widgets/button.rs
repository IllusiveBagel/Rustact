use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::interactions::{Hitbox, register_button_hitbox};
use crate::runtime::ButtonView;

pub fn render_button(frame: &mut Frame<'_>, area: Rect, view: &ButtonView) {
    register_button_hitbox(
        &view.id,
        Hitbox {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height,
        },
    );

    let mut style = Style::default();
    let mut highlight = Modifier::empty();
    let fg = view.accent.unwrap_or(Color::White);
    if view.filled {
        highlight = Modifier::BOLD;
    }
    if view.filled {
        style = style.bg(fg).fg(Color::Black);
    } else {
        style = style.fg(fg);
    }

    let content = Paragraph::new(Line::from(view.label.clone()))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .style(style.add_modifier(highlight));
    frame.render_widget(content, area);
}
