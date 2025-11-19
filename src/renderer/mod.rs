use std::io::{Stdout, stdout};

use anyhow::Context;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, SetTitle, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, Gauge, List, ListItem, ListState, Paragraph, Row, Table, TableState,
};
use ratatui::{Frame, Terminal};

use crate::interactions::{Hitbox, register_button_hitbox, reset_button_hitboxes};
use crate::runtime::{FlexDirection, FormFieldStatus, TableRowView, TableView, View};

pub struct Renderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Renderer {
    pub fn new(title: &str) -> anyhow::Result<Self> {
        enable_raw_mode().context("enable raw mode")?;
        let mut stdout = stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            Hide,
            SetTitle(title)
        )
        .context("prepare terminal")?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).context("build terminal")?;
        Ok(Self { terminal })
    }

    pub fn draw(&mut self, view: &View) -> anyhow::Result<()> {
        reset_button_hitboxes();
        self.terminal.draw(|frame| {
            let area = frame.size();
            render_view(frame, area, view);
        })?;
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = stdout();
        let _ = execute!(
            stdout,
            Show,
            DisableMouseCapture,
            LeaveAlternateScreen,
            SetTitle("Terminal")
        );
    }
}

fn render_view(frame: &mut Frame<'_>, area: Rect, view: &View) {
    match view {
        View::Empty => {}
        View::Text(text) => {
            let style = Style::default().fg(text.color.unwrap_or(Color::White));
            let widget = Paragraph::new(Line::from(text.content.clone())).style(style);
            frame.render_widget(widget, area);
        }
        View::Flex(flex) => {
            if flex.children.is_empty() {
                return;
            }
            let chunk_count = flex.children.len();
            let constraints = vec![Constraint::Ratio(1, chunk_count as u32); chunk_count];
            let layout = Layout::default()
                .direction(Direction::from(flex.direction))
                .constraints(constraints);
            let chunks = layout.split(area);
            for (child, rect) in flex.children.iter().zip(chunks.iter()) {
                render_view(frame, *rect, child);
            }
        }
        View::Block(block) => {
            let mut widget = Block::default().borders(Borders::ALL);
            if let Some(title) = &block.title {
                widget = widget.title(title.as_str());
            }
            frame.render_widget(widget.clone(), area);
            if let Some(child) = block.child.as_ref() {
                let inner = widget.inner(area);
                render_view(frame, inner, child);
            }
        }
        View::List(list) => {
            let items: Vec<ListItem> = if list.items.is_empty() {
                vec![ListItem::new(Line::from("(no entries)"))]
            } else {
                list.items
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
            if let Some(title) = &list.title {
                widget = widget.block(Block::default().borders(Borders::ALL).title(title.as_str()));
            }
            if let Some(index) = list.highlight.filter(|_| !list.items.is_empty()) {
                let mut state = ListState::default();
                state.select(Some(index.min(list.items.len() - 1)));
                let highlight_color = list.highlight_color.unwrap_or(Color::Yellow);
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
        View::Gauge(gauge) => {
            let mut widget = Gauge::default()
                .use_unicode(true)
                .ratio(gauge.ratio.clamp(0.0, 1.0));
            if let Some(label) = &gauge.label {
                widget = widget.label(Span::raw(label.clone()));
            } else {
                let percent = (gauge.ratio * 100.0).round();
                widget = widget.label(Span::raw(format!("{percent:.0}%")));
            }
            if let Some(color) = gauge.color {
                widget = widget.style(Style::default().fg(color));
            }
            frame.render_widget(widget, area);
        }
        View::Button(button) => {
            register_button_hitbox(
                &button.id,
                Hitbox {
                    x: area.x,
                    y: area.y,
                    width: area.width,
                    height: area.height,
                },
            );

            let mut style = Style::default();
            let mut highlight = Modifier::empty();
            let fg = button.accent.unwrap_or(Color::White);
            if button.filled {
                highlight = Modifier::BOLD;
            }
            if button.filled {
                style = style.bg(fg).fg(Color::Black);
            } else {
                style = style.fg(fg);
            }

            let content = Paragraph::new(Line::from(button.label.clone()))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL))
                .style(style.add_modifier(highlight));
            frame.render_widget(content, area);
        }
        View::Table(table) => {
            let mut block = Block::default().borders(Borders::ALL);
            if let Some(title) = &table.title {
                block = block.title(title.as_str());
            }

            let rows: Vec<Row> = if table.rows.is_empty() {
                vec![Row::new(vec![Cell::from("(no rows)")])]
            } else {
                table.rows.iter().map(build_table_row).collect()
            };

            let widths = resolve_table_widths(table);
            let mut widget = Table::new(rows, widths).block(block).column_spacing(1);
            if let Some(header) = table.header.as_ref() {
                widget = widget.header(build_table_row(header));
            }

            if let Some(index) = table.highlight.filter(|_| !table.rows.is_empty()) {
                let mut state = TableState::default();
                state.select(Some(index.min(table.rows.len() - 1)));
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
        View::Tree(tree) => {
            let mut block = Block::default().borders(Borders::ALL);
            if let Some(title) = &tree.title {
                block = block.title(title.as_str());
            }

            let items: Vec<ListItem> = if tree.rows.is_empty() {
                vec![ListItem::new(Line::from("(empty tree)"))]
            } else {
                tree.rows
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
            if let Some(index) = tree.highlight.filter(|_| !tree.rows.is_empty()) {
                let mut state = ListState::default();
                state.select(Some(index.min(tree.rows.len() - 1)));
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
        View::Form(form) => {
            let mut block = Block::default().borders(Borders::ALL);
            if let Some(title) = &form.title {
                block = block.title(title.as_str());
            }

            let rows: Vec<Row> = if form.fields.is_empty() {
                vec![Row::new(vec![Cell::from("(no fields)"), Cell::from("")])]
            } else {
                form.fields
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

            let label_pct = form.label_width.min(90).max(10);
            let widths = vec![
                Constraint::Percentage(label_pct),
                Constraint::Percentage(100 - label_pct),
            ];
            let widget = Table::new(rows, widths).block(block).column_spacing(1);
            frame.render_widget(widget, area);
        }
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

impl From<FlexDirection> for Direction {
    fn from(value: FlexDirection) -> Self {
        match value {
            FlexDirection::Row => Direction::Horizontal,
            FlexDirection::Column => Direction::Vertical,
        }
    }
}
