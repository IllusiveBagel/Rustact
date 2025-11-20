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
    LayeredNode, ListItemNode, ListNode, ModalNode, TabPaneNode, TableCellNode, TableNode,
    TableRowNode, TabsNode, TextInputNode, ToastLevel, ToastNode, ToastStackNode, TreeItemNode,
    TreeNode,
};
pub use tasks::{DefaultRuntimeDriver, RuntimeDriver};
pub use view::{
    BlockView, ButtonView, FlexView, FormFieldView, FormView, GaugeView, LayersView, ListItemView,
    ListView, ModalView, TabView, TableCellView, TableRowView, TableView, TabsView, TextInputView,
    TextView, ToastStackView, ToastView, TreeRowView, TreeView, View,
};

pub(crate) use component::ComponentId;
