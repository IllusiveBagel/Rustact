use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use unicode_width::UnicodeWidthStr;

use crate::interactions::Hitbox;
use crate::runtime::{FormFieldStatus, TextInputView};
use crate::text_input::TextInputs;

pub fn render_text_input(frame: &mut Frame<'_>, area: Rect, input: &TextInputView) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let mut input_area = area;
    if let Some(label) = &input.label {
        if input_area.height > 1 {
            let label_area = Rect {
                x: input_area.x,
                y: input_area.y,
                width: input_area.width,
                height: 1,
            };
            let mut label_style = Style::default().add_modifier(Modifier::BOLD);
            if let Some(color) = input.text_color.or(input.accent) {
                label_style = label_style.fg(color);
            } else {
                label_style = label_style.fg(Color::DarkGray);
            }
            frame.render_widget(
                Paragraph::new(Line::from(label.clone())).style(label_style),
                label_area,
            );
            input_area.y = input_area.y.saturating_add(1);
            input_area.height = input_area.height.saturating_sub(1);
        }
    }

    if input_area.height == 0 {
        return;
    }

    let desired_width = input.width.unwrap_or(input_area.width);
    let mut render_area = input_area;
    render_area.width = desired_width.min(input_area.width);

    let mut block = Block::default().borders(Borders::ALL);
    let status_color = status_to_color(input.status);
    let accent = input.accent.unwrap_or(Color::Cyan);
    let default_border = input.border_color.unwrap_or(Color::DarkGray);
    let focus_border = input.border_color.unwrap_or(accent);
    let border_color = status_color
        .or_else(|| {
            if input.focused {
                Some(focus_border)
            } else {
                None
            }
        })
        .unwrap_or(default_border);
    let mut border_style = Style::default().fg(border_color);
    if input.focused {
        border_style = border_style.add_modifier(Modifier::BOLD);
    }
    block = block.border_style(border_style);

    TextInputs::register_hitbox(
        &input.id,
        Hitbox {
            x: render_area.x,
            y: render_area.y,
            width: render_area.width,
            height: render_area.height.max(1),
        },
    );

    let background_color = if input.focused {
        input.focus_background.or(input.background_color)
    } else {
        input.background_color
    };
    let display_value = if input.secure {
        let count = input.value.chars().count();
        "*".repeat(count)
    } else {
        input.value.clone()
    };

    let placeholder_text = input.placeholder.clone().unwrap_or_default();
    let showing_placeholder = display_value.is_empty() && !placeholder_text.is_empty();
    let content = if showing_placeholder {
        placeholder_text.clone()
    } else {
        display_value.clone()
    };
    let mut text_style = Style::default();
    if let Some(bg) = background_color {
        text_style = text_style.bg(bg);
    }
    if let Some(color) = input.text_color {
        text_style = text_style.fg(color);
    }

    let mut paragraph = Paragraph::new(Line::from(content)).block(block.clone());
    if showing_placeholder {
        let placeholder_color = input.placeholder_color.unwrap_or(Color::DarkGray);
        paragraph = paragraph.style(text_style.fg(placeholder_color));
    } else {
        paragraph = paragraph.style(text_style);
    }
    frame.render_widget(paragraph, render_area);

    if input.focused && input.cursor_visible {
        let inner = block.inner(render_area);
        if inner.height > 0 {
            let cursor_index = input.cursor.min(input.value.len());
            let prefix = &input.value[..cursor_index];
            let cursor_width = if input.secure {
                prefix.chars().count() as u16
            } else {
                UnicodeWidthStr::width(prefix) as u16
            };
            let mut cursor_x = inner.x.saturating_add(cursor_width);
            let max_x = render_area
                .x
                .saturating_add(render_area.width.saturating_sub(1));
            if cursor_x > max_x {
                cursor_x = max_x;
            }
            frame.set_cursor(cursor_x, inner.y);
        }
    }
}

fn status_to_color(status: FormFieldStatus) -> Option<Color> {
    match status {
        FormFieldStatus::Normal => None,
        FormFieldStatus::Warning => Some(Color::Yellow),
        FormFieldStatus::Error => Some(Color::Red),
        FormFieldStatus::Success => Some(Color::Green),
    }
}
