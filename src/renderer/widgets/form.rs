use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

use crate::runtime::{FormFieldStatus, FormView};

pub fn render_form(frame: &mut Frame<'_>, area: Rect, view: &FormView) {
    let mut block = Block::default().borders(Borders::ALL);
    if let Some(title) = &view.title {
        block = block.title(title.as_str());
    }

    let rows: Vec<Row> = if view.fields.is_empty() {
        vec![Row::new(vec![Cell::from("(no fields)"), Cell::from("")])]
    } else {
        view.fields
            .iter()
            .map(|field| {
                let mut value_style = Style::default();
                value_style = match field.status {
                    FormFieldStatus::Normal => value_style,
                    FormFieldStatus::Warning => value_style.fg(Color::Yellow),
                    FormFieldStatus::Error => {
                        value_style.fg(Color::Red).add_modifier(Modifier::BOLD)
                    }
                    FormFieldStatus::Success => value_style.fg(Color::Green),
                };
                Row::new(vec![
                    Cell::from(Span::raw(field.label.clone()))
                        .style(Style::default().add_modifier(Modifier::BOLD)),
                    Cell::from(Span::raw(field.value.clone())).style(value_style),
                ])
            })
            .collect()
    };

    let label_pct = view.label_width.min(90).max(10);
    let widths = vec![
        Constraint::Percentage(label_pct),
        Constraint::Percentage(100 - label_pct),
    ];
    let widget = Table::new(rows, widths).block(block).column_spacing(1);
    frame.render_widget(widget, area);
}
