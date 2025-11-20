use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

use crate::runtime::TreeView;

pub fn render_tree(frame: &mut Frame<'_>, area: Rect, view: &TreeView) {
    let mut block = Block::default().borders(Borders::ALL);
    if let Some(title) = &view.title {
        block = block.title(title.as_str());
    }

    let items: Vec<ListItem> = if view.rows.is_empty() {
        vec![ListItem::new(Line::from("(empty tree)"))]
    } else {
        view.rows
            .iter()
            .map(|row| {
                let indent = "  ".repeat(row.depth);
                let marker = if row.has_children {
                    if row.expanded { "v " } else { "> " }
                } else {
                    "  "
                };
                let mut line = Line::from(format!("{indent}{marker}{}", row.label));
                if row.has_children {
                    line = line.style(Style::default().fg(Color::Cyan));
                }
                ListItem::new(line)
            })
            .collect()
    };

    let mut widget = List::new(items).block(block);
    if let Some(index) = view.highlight.filter(|_| !view.rows.is_empty()) {
        let mut state = ListState::default();
        state.select(Some(index.min(view.rows.len() - 1)));
        widget = widget.highlight_symbol("› ").highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_stateful_widget(widget, area, &mut state);
    } else {
        frame.render_widget(widget, area);
    }
}
