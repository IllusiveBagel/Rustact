use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use crossterm::event::EventStream;
use futures::StreamExt;
pub use ratatui::style::Color;
use tokio::signal;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::context::ContextStack;
use crate::events::{DEFAULT_TICK_RATE, EventBus, FrameworkEvent, is_ctrl_c, map_terminal_event};
use crate::hooks::{EffectInvocation, HookRegistry, Scope};
use crate::renderer::Renderer;
use crate::styles::Stylesheet;
use crate::text_input::{TextInputHandle, TextInputs};

#[derive(Clone)]
pub struct App {
    name: &'static str,
    root: ComponentElement,
    hooks: Arc<HookRegistry>,
    event_bus: EventBus,
    config: AppConfig,
    styles: Arc<Stylesheet>,
}

#[derive(Clone, Copy)]
pub struct AppConfig {
    pub tick_rate: Duration,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            tick_rate: DEFAULT_TICK_RATE,
        }
    }
}

impl App {
    pub fn new(name: &'static str, root: ComponentElement) -> Self {
        Self {
            name,
            root,
            hooks: Arc::new(HookRegistry::new()),
            event_bus: EventBus::new(64),
            config: AppConfig::default(),
            styles: Arc::new(Stylesheet::default()),
        }
    }

    pub fn with_config(mut self, config: AppConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_stylesheet(mut self, stylesheet: Stylesheet) -> Self {
        self.styles = Arc::new(stylesheet);
        self
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let (tx, mut rx) = mpsc::channel(128);
        let dispatcher = Dispatcher::new(tx.clone(), self.event_bus.clone());
        let mut renderer = Renderer::new(self.name).context("initialize renderer")?;
        let mut last_view: Option<View> = None;

        let event_task = spawn_terminal_events(tx.clone());
        let tick_task = spawn_tick_loop(tx.clone(), self.config.tick_rate);
        let shutdown_task = spawn_shutdown_watcher(tx.clone());

        tx.send(AppMessage::RequestRender).await.ok();
        let mut live_components = HashSet::new();

        while let Some(message) = rx.recv().await {
            match message {
                AppMessage::RequestRender => {
                    live_components.clear();
                    let mut effects = Vec::new();
                    let mut context = ContextStack::new();
                    let mut path = vec![0usize];
                    let view = self
                        .render_element(
                            Element::from(self.root.clone()),
                            &dispatcher,
                            &mut path,
                            &mut context,
                            &mut live_components,
                            &mut effects,
                        )?
                        .unwrap_or(View::Empty);

                    let should_render =
                        last_view.as_ref().map(|prev| prev != &view).unwrap_or(true);
                    if should_render {
                        renderer.draw(&view)?;
                    }
                    last_view = Some(view);
                    self.run_effects(effects, &dispatcher);
                    self.hooks.prune(&live_components);
                }
                AppMessage::ExternalEvent(event) => {
                    TextInputs::handle_event(&event, &dispatcher);
                    self.event_bus.publish(event);
                }
                AppMessage::Shutdown => break,
            }
        }

        drop(renderer);
        event_task.abort();
        tick_task.abort();
        shutdown_task.abort();
        Ok(())
    }

    fn run_effects(&self, effects: Vec<EffectInvocation>, dispatcher: &Dispatcher) {
        for effect in effects {
            let EffectInvocation {
                component_id,
                slot_index,
                deps,
                task,
            } = effect;
            self.hooks
                .with_effect_slot(&component_id, slot_index, |slot| {
                    if let Some(cleanup) = slot.take_cleanup() {
                        cleanup();
                    }
                });
            let cleanup = task(dispatcher.clone());
            self.hooks
                .with_effect_slot(&component_id, slot_index, |slot| {
                    slot.set_deps(deps);
                    slot.set_cleanup(cleanup);
                });
        }
    }

    fn render_element(
        &self,
        element: Element,
        dispatcher: &Dispatcher,
        path: &mut Vec<usize>,
        context: &mut ContextStack,
        live: &mut HashSet<ComponentId>,
        effects: &mut Vec<EffectInvocation>,
    ) -> anyhow::Result<Option<View>> {
        match element {
            Element::Empty => Ok(Some(View::Empty)),
            Element::Text(node) => Ok(Some(View::Text(TextView {
                content: node.content,
                color: node.color,
            }))),
            Element::Flex(node) => {
                let mut children = Vec::new();
                for (index, child) in node.children.into_iter().enumerate() {
                    path.push(index);
                    if let Some(view) =
                        self.render_element(child, dispatcher, path, context, live, effects)?
                    {
                        children.push(view);
                    }
                    path.pop();
                }
                if children.is_empty() {
                    Ok(Some(View::Empty))
                } else {
                    Ok(Some(View::Flex(FlexView {
                        direction: node.direction,
                        children,
                    })))
                }
            }
            Element::Block(node) => {
                path.push(0);
                let child =
                    self.render_element(*node.child, dispatcher, path, context, live, effects)?;
                path.pop();
                Ok(Some(View::Block(BlockView {
                    title: node.title,
                    child: child.map(Box::new),
                })))
            }
            Element::List(node) => {
                let items = node
                    .items
                    .into_iter()
                    .map(|item| ListItemView {
                        content: item.content,
                        color: item.color,
                    })
                    .collect();
                Ok(Some(View::List(ListView {
                    title: node.title,
                    items,
                    highlight: node.highlight,
                    highlight_color: node.highlight_color,
                })))
            }
            Element::Gauge(node) => Ok(Some(View::Gauge(GaugeView {
                label: node.label,
                ratio: node.ratio,
                color: node.color,
            }))),
            Element::Button(node) => Ok(Some(View::Button(ButtonView {
                id: node.id,
                label: node.label,
                accent: node.accent,
                filled: node.filled,
            }))),
            Element::Table(node) => {
                let header = node.header.map(|row| TableRowView {
                    cells: row
                        .cells
                        .into_iter()
                        .map(|cell| TableCellView {
                            content: cell.content,
                            color: cell.color,
                            bold: cell.bold,
                        })
                        .collect(),
                });
                let rows = node
                    .rows
                    .into_iter()
                    .map(|row| TableRowView {
                        cells: row
                            .cells
                            .into_iter()
                            .map(|cell| TableCellView {
                                content: cell.content,
                                color: cell.color,
                                bold: cell.bold,
                            })
                            .collect(),
                    })
                    .collect();
                Ok(Some(View::Table(TableView {
                    title: node.title,
                    header,
                    rows,
                    highlight: node.highlight,
                    column_widths: node.column_widths,
                })))
            }
            Element::Tree(node) => {
                let rows = flatten_tree_items(node.items);
                Ok(Some(View::Tree(TreeView {
                    title: node.title,
                    rows,
                    highlight: node.highlight,
                })))
            }
            Element::Form(node) => {
                let fields = node
                    .fields
                    .into_iter()
                    .map(|field| FormFieldView {
                        label: field.label,
                        value: field.value,
                        status: field.status,
                    })
                    .collect();
                Ok(Some(View::Form(FormView {
                    title: node.title,
                    fields,
                    label_width: node.label_width,
                })))
            }
            Element::Input(node) => {
                let snapshot = node.binding.snapshot();
                let id = (*snapshot.id).clone();
                let focused = TextInputs::is_focused(&id);
                let cursor_visible = TextInputs::cursor_visible(&id);
                let status = snapshot.status.unwrap_or(node.status);
                Ok(Some(View::Input(TextInputView {
                    id,
                    label: node.label,
                    value: snapshot.value,
                    placeholder: node.placeholder,
                    width: node.width,
                    focused,
                    cursor: snapshot.cursor,
                    secure: node.secure,
                    accent: node.accent,
                    border_color: node.border_color,
                    text_color: node.text_color,
                    placeholder_color: node.placeholder_color,
                    background_color: node.background_color,
                    focus_background: node.focus_background,
                    status,
                    cursor_visible,
                })))
            }
            Element::Fragment(children) => {
                let mut views = Vec::new();
                for (index, child) in children.into_iter().enumerate() {
                    path.push(index);
                    if let Some(view) =
                        self.render_element(child, dispatcher, path, context, live, effects)?
                    {
                        views.push(view);
                    }
                    path.pop();
                }
                if views.is_empty() {
                    Ok(Some(View::Empty))
                } else if views.len() == 1 {
                    Ok(views.pop())
                } else {
                    Ok(Some(View::Flex(FlexView {
                        direction: FlexDirection::Column,
                        children: views,
                    })))
                }
            }
            Element::Component(component) => {
                self.render_component(component, dispatcher, path, context, live, effects)
            }
        }
    }

    fn render_component(
        &self,
        component: ComponentElement,
        dispatcher: &Dispatcher,
        path: &mut Vec<usize>,
        context: &mut ContextStack,
        live: &mut HashSet<ComponentId>,
        effects: &mut Vec<EffectInvocation>,
    ) -> anyhow::Result<Option<View>> {
        let id = ComponentId::new(path, component.name, component.key.as_deref());
        live.insert(id.clone());
        let store = self.hooks.store_for(&id);
        let mut scope = Scope::new(
            id.clone(),
            store,
            dispatcher.clone(),
            context,
            self.styles.clone(),
        );
        let child = (component.render)(&mut scope);
        effects.extend(scope.take_effects());
        self.render_element(child, dispatcher, path, context, live, effects)
    }
}

fn spawn_terminal_events(tx: mpsc::Sender<AppMessage>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut events = EventStream::new();
        while let Some(event) = events.next().await {
            match event {
                Ok(evt) => {
                    if let Some(mapped) = map_terminal_event(evt) {
                        let shutdown = is_ctrl_c(&mapped);
                        if tx.send(AppMessage::ExternalEvent(mapped)).await.is_err() {
                            break;
                        }
                        if shutdown {
                            let _ = tx.send(AppMessage::Shutdown).await;
                            break;
                        }
                    }
                }
                Err(_) => break,
            }
        }
    })
}

fn spawn_tick_loop(tx: mpsc::Sender<AppMessage>, rate: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(rate);
        loop {
            ticker.tick().await;
            if tx
                .send(AppMessage::ExternalEvent(FrameworkEvent::Tick))
                .await
                .is_err()
            {
                break;
            }
        }
    })
}

fn spawn_shutdown_watcher(tx: mpsc::Sender<AppMessage>) -> JoinHandle<()> {
    tokio::spawn(async move {
        if signal::ctrl_c().await.is_ok() {
            let _ = tx.send(AppMessage::Shutdown).await;
        }
    })
}

#[derive(Clone)]
pub struct Dispatcher {
    tx: mpsc::Sender<AppMessage>,
    event_bus: EventBus,
}

impl Dispatcher {
    fn new(tx: mpsc::Sender<AppMessage>, event_bus: EventBus) -> Self {
        Self { tx, event_bus }
    }

    pub fn request_render(&self) {
        let _ = self.tx.try_send(AppMessage::RequestRender);
    }

    pub fn events(&self) -> EventBus {
        self.event_bus.clone()
    }
}

enum AppMessage {
    RequestRender,
    ExternalEvent(FrameworkEvent),
    Shutdown,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComponentId(String);

impl ComponentId {
    fn new(path: &[usize], name: &str, key: Option<&str>) -> Self {
        let mut id = path
            .iter()
            .map(|segment| segment.to_string())
            .collect::<Vec<_>>()
            .join(".");
        if let Some(key) = key {
            id.push('#');
            id.push_str(key);
        }
        id.push(':');
        id.push_str(name);
        Self(id)
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone)]
pub struct ComponentElement {
    pub(crate) name: &'static str,
    pub(crate) key: Option<String>,
    pub(crate) render: ComponentFn,
}

pub type ComponentFn = Arc<dyn Fn(&mut Scope) -> Element + Send + Sync>;

impl ComponentElement {
    pub fn new<F>(name: &'static str, render: F) -> Self
    where
        F: Fn(&mut Scope) -> Element + Send + Sync + 'static,
    {
        Self {
            name,
            key: None,
            render: Arc::new(render),
        }
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl From<ComponentElement> for Element {
    fn from(value: ComponentElement) -> Self {
        Element::Component(value)
    }
}

impl fmt::Debug for ComponentElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComponentElement")
            .field("name", &self.name)
            .field("key", &self.key)
            .finish()
    }
}

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

pub fn component<F>(name: &'static str, render: F) -> ComponentElement
where
    F: Fn(&mut Scope) -> Element + Send + Sync + 'static,
{
    ComponentElement::new(name, render)
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

fn flatten_tree_items(items: Vec<TreeItemNode>) -> Vec<TreeRowView> {
    let mut rows = Vec::new();
    push_tree_items(items, 0, &mut rows);
    rows
}

fn push_tree_items(nodes: Vec<TreeItemNode>, depth: usize, rows: &mut Vec<TreeRowView>) {
    for node in nodes {
        let has_children = !node.children.is_empty();
        let expanded = node.expanded && has_children;
        rows.push(TreeRowView {
            label: node.label,
            depth,
            has_children,
            expanded,
        });
        if expanded {
            push_tree_items(node.children, depth + 1, rows);
        }
    }
}
