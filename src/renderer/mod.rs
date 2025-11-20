use std::io::{Stdout, stdout};

use anyhow::Context;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, SetTitle, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::{Frame, Terminal};

use crate::interactions::reset_button_hitboxes;
use crate::runtime::View;
use crate::text_input::TextInputs;

mod widgets;

use widgets::{
    render_block, render_button, render_flex, render_form, render_gauge, render_list, render_table,
    render_text, render_text_input, render_tree,
};

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
        TextInputs::reset_hitboxes();
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
        View::Text(text) => render_text(frame, area, text),
        View::Flex(flex) => render_flex(frame, area, flex, render_view),
        View::Block(block) => render_block(frame, area, block, render_view),
        View::List(list) => render_list(frame, area, list),
        View::Gauge(gauge) => render_gauge(frame, area, gauge),
        View::Button(button) => render_button(frame, area, button),
        View::Table(table) => render_table(frame, area, table),
        View::Tree(tree) => render_tree(frame, area, tree),
        View::Form(form) => render_form(frame, area, form),
        View::Input(input) => render_text_input(frame, area, input),
    }
}
