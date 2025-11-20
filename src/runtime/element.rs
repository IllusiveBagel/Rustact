use ratatui::style::Color;

use crate::text_input::TextInputHandle;

use super::component::ComponentElement;

#[derive(Clone, Debug)]
pub enum Element {
    Empty,
    Text(TextNode),
    Flex(FlexNode),
    Block(BlockNode),
    List(ListNode),
    Gauge(GaugeNode),
    Button(ButtonNode),
    Table(TableNode),
    Tree(TreeNode),
    Form(FormNode),
    Input(TextInputNode),
    Fragment(Vec<Element>),
    Component(ComponentElement),
}

#[derive(Clone, Debug)]
pub struct TextNode {
    pub content: String,
    pub color: Option<Color>,
}

#[derive(Clone, Debug)]
pub struct FlexNode {
    pub direction: FlexDirection,
    pub children: Vec<Element>,
}

#[derive(Clone, Debug)]
pub struct BlockNode {
    pub title: Option<String>,
    pub child: Box<Element>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
}

impl Element {
    pub fn text(content: impl Into<String>) -> Self {
        Element::Text(TextNode {
            content: content.into(),
            color: None,
        })
    }

    pub fn colored_text(content: impl Into<String>, color: Color) -> Self {
        Element::Text(TextNode {
            content: content.into(),
            color: Some(color),
        })
    }

    pub fn vstack(children: Vec<Element>) -> Self {
        Element::Flex(FlexNode {
            direction: FlexDirection::Column,
            children,
        })
    }

    pub fn hstack(children: Vec<Element>) -> Self {
        Element::Flex(FlexNode {
            direction: FlexDirection::Row,
            children,
        })
    }

    pub fn block(title: impl Into<String>, child: Element) -> Self {
        Element::Block(BlockNode {
            title: Some(title.into()),
            child: Box::new(child),
        })
    }

    pub fn fragment(children: Vec<Element>) -> Self {
        Element::Fragment(children)
    }

    pub fn list(node: ListNode) -> Self {
        Element::List(node)
    }

    pub fn gauge(node: GaugeNode) -> Self {
        Element::Gauge(node)
    }

    pub fn button(node: ButtonNode) -> Self {
        Element::Button(node)
    }

    pub fn table(node: TableNode) -> Self {
        Element::Table(node)
    }

    pub fn tree(node: TreeNode) -> Self {
        Element::Tree(node)
    }

    pub fn form(node: FormNode) -> Self {
        Element::Form(node)
    }

    pub fn text_input(node: TextInputNode) -> Self {
        Element::Input(node)
    }
}

#[derive(Clone, Debug)]
pub struct ListNode {
    pub title: Option<String>,
    pub items: Vec<ListItemNode>,
    pub highlight: Option<usize>,
    pub highlight_color: Option<Color>,
}

impl ListNode {
    pub fn new(items: Vec<ListItemNode>) -> Self {
        Self {
            title: None,
            items,
            highlight: None,
            highlight_color: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn highlight(mut self, index: usize) -> Self {
        self.highlight = Some(index);
        self
    }

    pub fn highlight_color(mut self, color: Color) -> Self {
        self.highlight_color = Some(color);
        self
    }
}

#[derive(Clone, Debug)]
pub struct ListItemNode {
    pub content: String,
    pub color: Option<Color>,
}

impl ListItemNode {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            color: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Clone, Debug)]
pub struct GaugeNode {
    pub label: Option<String>,
    pub ratio: f64,
    pub color: Option<Color>,
}

impl GaugeNode {
    pub fn new(ratio: f64) -> Self {
        Self {
            label: None,
            ratio,
            color: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Clone, Debug)]
pub struct ButtonNode {
    pub id: String,
    pub label: String,
    pub accent: Option<Color>,
    pub filled: bool,
}

impl ButtonNode {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            accent: None,
            filled: false,
        }
    }

    pub fn accent(mut self, color: Color) -> Self {
        self.accent = Some(color);
        self
    }

    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }
}

#[derive(Clone, Debug)]
pub struct TableNode {
    pub title: Option<String>,
    pub header: Option<TableRowNode>,
    pub rows: Vec<TableRowNode>,
    pub highlight: Option<usize>,
    pub column_widths: Option<Vec<u16>>,
}

impl TableNode {
    pub fn new(rows: Vec<TableRowNode>) -> Self {
        Self {
            title: None,
            header: None,
            rows,
            highlight: None,
            column_widths: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn header(mut self, header: TableRowNode) -> Self {
        self.header = Some(header);
        self
    }

    pub fn highlight(mut self, index: usize) -> Self {
        self.highlight = Some(index);
        self
    }

    pub fn widths(mut self, widths: Vec<u16>) -> Self {
        self.column_widths = Some(widths);
        self
    }
}

#[derive(Clone, Debug)]
pub struct TableRowNode {
    pub cells: Vec<TableCellNode>,
}

impl TableRowNode {
    pub fn new(cells: Vec<TableCellNode>) -> Self {
        Self { cells }
    }

    pub fn cell(mut self, cell: TableCellNode) -> Self {
        self.cells.push(cell);
        self
    }
}

#[derive(Clone, Debug)]
pub struct TableCellNode {
    pub content: String,
    pub color: Option<Color>,
    pub bold: bool,
}

impl TableCellNode {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            color: None,
            bold: false,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
}

#[derive(Clone, Debug)]
pub struct TreeNode {
    pub title: Option<String>,
    pub items: Vec<TreeItemNode>,
    pub highlight: Option<usize>,
}

impl TreeNode {
    pub fn new(items: Vec<TreeItemNode>) -> Self {
        Self {
            title: None,
            items,
            highlight: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn highlight(mut self, index: usize) -> Self {
        self.highlight = Some(index);
        self
    }
}

#[derive(Clone, Debug)]
pub struct TreeItemNode {
    pub label: String,
    pub children: Vec<TreeItemNode>,
    pub expanded: bool,
}

impl TreeItemNode {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
            expanded: true,
        }
    }

    pub fn child(mut self, child: TreeItemNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: Vec<TreeItemNode>) -> Self {
        self.children = children;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
}

#[derive(Clone, Debug)]
pub struct FormNode {
    pub title: Option<String>,
    pub fields: Vec<FormFieldNode>,
    pub label_width: u16,
}

impl FormNode {
    pub fn new(fields: Vec<FormFieldNode>) -> Self {
        Self {
            title: None,
            fields,
            label_width: 30,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn label_width(mut self, percent: u16) -> Self {
        self.label_width = percent.clamp(10, 90);
        self
    }
}

#[derive(Clone, Debug)]
pub struct FormFieldNode {
    pub label: String,
    pub value: String,
    pub status: FormFieldStatus,
}

impl FormFieldNode {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            status: FormFieldStatus::Normal,
        }
    }

    pub fn status(mut self, status: FormFieldStatus) -> Self {
        self.status = status;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormFieldStatus {
    Normal,
    Warning,
    Error,
    Success,
}

#[derive(Clone, Debug)]
pub struct TextInputNode {
    pub binding: TextInputHandle,
    pub label: Option<String>,
    pub placeholder: Option<String>,
    pub width: Option<u16>,
    pub secure: bool,
    pub accent: Option<Color>,
    pub border_color: Option<Color>,
    pub text_color: Option<Color>,
    pub placeholder_color: Option<Color>,
    pub background_color: Option<Color>,
    pub focus_background: Option<Color>,
    pub status: FormFieldStatus,
}

impl TextInputNode {
    pub fn new(binding: TextInputHandle) -> Self {
        Self {
            binding,
            label: None,
            placeholder: None,
            width: None,
            secure: false,
            accent: None,
            border_color: None,
            text_color: None,
            placeholder_color: None,
            background_color: None,
            focus_background: None,
            status: FormFieldStatus::Normal,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    pub fn accent(mut self, color: Color) -> Self {
        self.accent = Some(color);
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
        self
    }

    pub fn placeholder_color(mut self, color: Color) -> Self {
        self.placeholder_color = Some(color);
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn focus_background(mut self, color: Color) -> Self {
        self.focus_background = Some(color);
        self
    }

    pub fn status(mut self, status: FormFieldStatus) -> Self {
        self.status = status;
        self
    }
}
