use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};

use crate::runtime::{TableRowView, TableView};

pub fn render_table(frame: &mut Frame<'_>, area: Rect, view: &TableView) {
    let mut block = Block::default().borders(Borders::ALL);
    if let Some(title) = &view.title {
        block = block.title(title.as_str());
    }

    let rows: Vec<Row> = if view.rows.is_empty() {
        vec![Row::new(vec![Cell::from("(no rows)")])]
    } else {
        view.rows.iter().map(build_table_row).collect()
    };

    let widths = resolve_table_widths(view);
    let mut widget = Table::new(rows, widths).block(block).column_spacing(1);
    if let Some(header) = view.header.as_ref() {
        widget = widget.header(build_table_row(header));
    }

    if let Some(index) = view.highlight.filter(|_| !view.rows.is_empty()) {
        let mut state = TableState::default();
        state.select(Some(index.min(view.rows.len() - 1)));
        widget = widget.highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::REVERSED),
        );
        frame.render_stateful_widget(widget, area, &mut state);
    } else {
        frame.render_widget(widget, area);
    }
}

fn build_table_row(row: &TableRowView) -> Row<'static> {
    let cells: Vec<Cell> = row
        .cells
        .iter()
        .map(|cell| {
            let mut style = Style::default();
            if let Some(color) = cell.color {
                style = style.fg(color);
            }
            if cell.bold {
                style = style.add_modifier(Modifier::BOLD);
            }
            Cell::from(Span::raw(cell.content.clone())).style(style)
        })
        .collect();
    Row::new(cells)
}

fn resolve_table_widths(table: &TableView) -> Vec<Constraint> {
    let column_count = table
        .header
        .as_ref()
        .map(|row| row.cells.len())
        .or_else(|| table.rows.first().map(|row| row.cells.len()))
        .unwrap_or(1)
        .max(1);

    if let Some(widths) = &table.column_widths {
        let mut constraints: Vec<Constraint> = widths
            .iter()
            .copied()
            .map(|percent| Constraint::Percentage(percent.min(100)))
            .collect();
        if constraints.len() > column_count {
            constraints.truncate(column_count);
        } else if constraints.len() < column_count {
            let fallback = Constraint::Ratio(1, column_count as u32);
            constraints.resize(column_count, fallback);
        }
        constraints
    } else {
        vec![Constraint::Ratio(1, column_count as u32); column_count]
    }
}
