use ratatui::Frame;
use ratatui::layout::Rect;

use crate::runtime::View;

pub type RenderFn = fn(&mut Frame<'_>, Rect, &View);

pub mod block;
pub mod button;
pub mod flex;
pub mod form;
pub mod gauge;
pub mod input;
pub mod layers;
pub mod list;
pub mod modal;
pub mod table;
pub mod tabs;
pub mod text;
pub mod toast;
pub mod tree;

pub use block::render_block;
pub use button::render_button;
pub use flex::render_flex;
pub use form::render_form;
pub use gauge::render_gauge;
pub use input::render_text_input;
pub use layers::render_layers;
pub use list::render_list;
pub use modal::render_modal;
pub use table::render_table;
pub use tabs::render_tabs;
pub use text::render_text;
pub use toast::render_toast_stack;
pub use tree::render_tree;
