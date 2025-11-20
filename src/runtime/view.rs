use ratatui::style::Color;

use super::element::{FlexDirection, FormFieldStatus, ToastLevel};

#[derive(Clone, Debug, PartialEq)]
pub enum View {
    Empty,
    Text(TextView),
    Flex(FlexView),
    Block(BlockView),
    List(ListView),
    Gauge(GaugeView),
    Button(ButtonView),
    Table(TableView),
    Tree(TreeView),
    Form(FormView),
    Input(TextInputView),
    Tabs(TabsView),
    Layered(LayersView),
    Modal(ModalView),
    ToastStack(ToastStackView),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextView {
    pub content: String,
    pub color: Option<Color>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlexView {
    pub direction: FlexDirection,
    pub children: Vec<View>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockView {
    pub title: Option<String>,
    pub child: Option<Box<View>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ListView {
    pub title: Option<String>,
    pub items: Vec<ListItemView>,
    pub highlight: Option<usize>,
    pub highlight_color: Option<Color>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ListItemView {
    pub content: String,
    pub color: Option<Color>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GaugeView {
    pub label: Option<String>,
    pub ratio: f64,
    pub color: Option<Color>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonView {
    pub id: String,
    pub label: String,
    pub accent: Option<Color>,
    pub filled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TableView {
    pub title: Option<String>,
    pub header: Option<TableRowView>,
    pub rows: Vec<TableRowView>,
    pub highlight: Option<usize>,
    pub column_widths: Option<Vec<u16>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TableRowView {
    pub cells: Vec<TableCellView>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TableCellView {
    pub content: String,
    pub color: Option<Color>,
    pub bold: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TreeView {
    pub title: Option<String>,
    pub rows: Vec<TreeRowView>,
    pub highlight: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TreeRowView {
    pub label: String,
    pub depth: usize,
    pub has_children: bool,
    pub expanded: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FormView {
    pub title: Option<String>,
    pub fields: Vec<FormFieldView>,
    pub label_width: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FormFieldView {
    pub label: String,
    pub value: String,
    pub status: FormFieldStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextInputView {
    pub id: String,
    pub label: Option<String>,
    pub value: String,
    pub placeholder: Option<String>,
    pub width: Option<u16>,
    pub focused: bool,
    pub cursor: usize,
    pub secure: bool,
    pub accent: Option<Color>,
    pub border_color: Option<Color>,
    pub text_color: Option<Color>,
    pub placeholder_color: Option<Color>,
    pub background_color: Option<Color>,
    pub focus_background: Option<Color>,
    pub status: FormFieldStatus,
    pub cursor_visible: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TabsView {
    pub tabs: Vec<TabView>,
    pub active: usize,
    pub accent: Option<Color>,
    pub title: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TabView {
    pub label: String,
    pub content: View,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayersView {
    pub layers: Vec<View>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModalView {
    pub title: Option<String>,
    pub content: Box<View>,
    pub width: Option<u16>,
    pub height: Option<u16>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToastStackView {
    pub toasts: Vec<ToastView>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToastView {
    pub title: String,
    pub body: Option<String>,
    pub level: ToastLevel,
}
