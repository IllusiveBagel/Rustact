mod app;
mod component;
mod dispatcher;
mod element;
mod tasks;
mod view;

#[cfg(test)]
mod tests;

pub use ratatui::style::Color;

pub use app::{App, AppConfig};
pub use component::{ComponentElement, ComponentFn, component};
pub use dispatcher::Dispatcher;
pub use element::{
    ButtonNode, Element, FlexDirection, FormFieldNode, FormFieldStatus, FormNode, GaugeNode,
    ListItemNode, ListNode, TableCellNode, TableNode, TableRowNode, TextInputNode, TreeItemNode,
    TreeNode,
};
pub use view::{
    BlockView, ButtonView, FlexView, FormFieldView, FormView, GaugeView, ListItemView, ListView,
    TableCellView, TableRowView, TableView, TextInputView, TextView, TreeRowView, TreeView, View,
};

pub(crate) use component::ComponentId;
