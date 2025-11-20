use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Tabs};

use crate::runtime::TabsView;

use super::RenderFn;

pub fn render_tabs(frame: &mut Frame<'_>, area: Rect, view: &TabsView, render_child: RenderFn) {
    if view.tabs.is_empty() {
        return;
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let active = view.active.min(view.tabs.len().saturating_sub(1));

    let titles = view.tabs.iter().map(|tab| Line::from(tab.label.clone()));
    let highlight_style = Style::default()
        .fg(view.accent.unwrap_or(Color::Cyan))
        .add_modifier(Modifier::BOLD);
    let mut tabs_widget = Tabs::new(titles)
        .select(active)
        .highlight_style(highlight_style);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(view.title.clone().unwrap_or_else(|| "Tabs".to_string()));
    tabs_widget = tabs_widget.block(block);
    frame.render_widget(tabs_widget, layout[0]);

    if let Some(active_view) = view.tabs.get(active) {
        render_child(frame, layout[1], &active_view.content);
    }
}
