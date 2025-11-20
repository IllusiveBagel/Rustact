use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

use crate::runtime::ListView;

pub fn render_list(frame: &mut Frame<'_>, area: Rect, view: &ListView) {
    let items: Vec<ListItem> = if view.items.is_empty() {
        vec![ListItem::new(Line::from("(no entries)"))]
    } else {
        view.items
            .iter()
            .map(|item| {
                let mut line = Line::from(item.content.clone());
                if let Some(color) = item.color {
                    line = line.style(Style::default().fg(color));
                }
                ListItem::new(line)
            })
            .collect()
    };

    let mut widget = List::new(items);
    if let Some(title) = &view.title {
        widget = widget.block(Block::default().borders(Borders::ALL).title(title.as_str()));
    }

    if let Some(index) = view.highlight.filter(|_| !view.items.is_empty()) {
        let mut state = ListState::default();
        state.select(Some(index.min(view.items.len() - 1)));
        let highlight_color = view.highlight_color.unwrap_or(Color::Yellow);
        widget = widget.highlight_symbol("▶ ").highlight_style(
            Style::default()
                .fg(highlight_color)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_stateful_widget(widget, area, &mut state);
    } else {
        frame.render_widget(widget, area);
    }
}
