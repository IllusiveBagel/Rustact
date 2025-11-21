use std::collections::{HashSet, hash_map::DefaultHasher};
use std::env;
use std::hash::{Hash, Hasher};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use tokio::fs;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tracing::{info, trace, warn};

use crate::context::ContextStack;
use crate::events::{DEFAULT_TICK_RATE, EventBus};
use crate::hooks::{EffectInvocation, HookRegistry, Scope};
use crate::renderer::Renderer;
use crate::styles::Stylesheet;
use crate::text_input::TextInputs;

use super::component::{ComponentElement, ComponentId};
use super::dispatcher::{AppMessage, Dispatcher};
use super::element::{Element, FlexDirection, TreeItemNode};
use super::tasks::{DefaultRuntimeDriver, RuntimeDriver};
use super::view::{
    BlockView, ButtonView, FlexView, FormFieldView, FormView, GaugeView, LayersView, ListItemView,
    ListView, ModalView, TabView, TableCellView, TableRowView, TableView, TabsView, TextInputView,
    TextView, ToastStackView, ToastView, TreeRowView, TreeView, View,
};

#[derive(Clone, Copy)]
enum RendererMode {
    Interactive,
    Headless,
}

#[derive(Clone)]
pub struct App {
    name: &'static str,
    root: ComponentElement,
    hooks: Arc<HookRegistry>,
    event_bus: EventBus,
    config: AppConfig,
    styles: Arc<Stylesheet>,
    driver: Arc<dyn RuntimeDriver>,
    stylesheet_watch: Option<PathBuf>,
    renderer_mode: RendererMode,
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
            driver: Arc::new(DefaultRuntimeDriver),
            stylesheet_watch: None,
            renderer_mode: RendererMode::Interactive,
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

    pub fn watch_stylesheet<P>(mut self, path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        let candidate = path.into();
        let resolved = if candidate.is_absolute() {
            candidate
        } else {
            match env::current_dir() {
                Ok(cwd) => cwd.join(&candidate),
                Err(_) => candidate,
            }
        };
        self.stylesheet_watch = Some(resolved);
        self
    }

    pub fn with_driver<D>(mut self, driver: D) -> Self
    where
        D: RuntimeDriver + 'static,
    {
        self.driver = Arc::new(driver);
        self
    }

    pub fn headless(mut self) -> Self {
        self.renderer_mode = RendererMode::Headless;
        self
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        info!(app = self.name, "starting runtime");
        let (tx, mut rx) = mpsc::channel(128);
        let dispatcher = Dispatcher::new(tx.clone(), self.event_bus.clone());
        let mut renderer = match self.renderer_mode {
            RendererMode::Interactive => Renderer::new(self.name).context("initialize renderer")?,
            RendererMode::Headless => Renderer::headless().context("initialize renderer")?,
        };
        let mut last_view: Option<View> = None;

        let event_task = self.driver.spawn_terminal_events(tx.clone());
        let tick_task = self
            .driver
            .spawn_tick_loop(tx.clone(), self.config.tick_rate);
        let shutdown_task = self.driver.spawn_shutdown_watcher(tx.clone());
        let stylesheet_task = self
            .stylesheet_watch
            .clone()
            .map(|path| spawn_stylesheet_watcher(path, tx.clone()));

        if tx.send(AppMessage::RequestRender).await.is_err() {
            warn!(app = self.name, "failed to enqueue initial render request");
        }
        let mut live_components = HashSet::new();

        while let Some(message) = rx.recv().await {
            trace!(app = self.name, message = ?message, "processing app message");
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
                        renderer.draw(&view).map_err(|err| {
                            warn!(app = self.name, error = ?err, "renderer draw failed");
                            err
                        })?;
                        trace!(app = self.name, "frame drawn");
                    }
                    last_view = Some(view);
                    trace!(
                        app = self.name,
                        effect_count = effects.len(),
                        "render completed"
                    );
                    self.run_effects(effects, &dispatcher);
                    self.hooks.prune(&live_components);
                }
                AppMessage::ExternalEvent(event) => {
                    trace!(app = self.name, event = ?event, "dispatching external event");
                    TextInputs::handle_event(&event, &dispatcher);
                    self.event_bus.publish(event);
                }
                AppMessage::Shutdown => {
                    info!(app = self.name, "shutdown requested");
                    break;
                }
                AppMessage::StylesheetUpdated(stylesheet) => {
                    self.styles = stylesheet;
                    info!(app = self.name, "stylesheet reloaded");
                    dispatcher.request_render();
                }
            }
        }

        drop(renderer);
        trace!(app = self.name, "tearing down runtime tasks");
        abort_and_log("terminal_events", event_task).await;
        abort_and_log("tick_loop", tick_task).await;
        abort_and_log("shutdown_watcher", shutdown_task).await;
        if let Some(task) = stylesheet_task {
            task.abort();
        }
        info!(app = self.name, "runtime stopped");
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
            trace!(
                component = %component_id,
                slot_index,
                "running effect cleanup"
            );
            self.hooks
                .with_effect_slot(&component_id, slot_index, |slot| {
                    if let Some(cleanup) = slot.take_cleanup() {
                        cleanup();
                    }
                });
            trace!(component = %component_id, slot_index, "invoking effect task");
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
            Element::Tabs(node) => {
                let mut tabs = Vec::new();
                for (index, tab) in node.tabs.into_iter().enumerate() {
                    path.push(index);
                    let view =
                        self.render_element(tab.content, dispatcher, path, context, live, effects)?;
                    path.pop();
                    if let Some(view) = view {
                        tabs.push(TabView {
                            label: tab.label,
                            content: view,
                        });
                    }
                }
                if tabs.is_empty() {
                    Ok(Some(View::Empty))
                } else {
                    let clamped = node.active.min(tabs.len().saturating_sub(1));
                    Ok(Some(View::Tabs(TabsView {
                        tabs,
                        active: clamped,
                        accent: node.accent,
                        title: node.title,
                    })))
                }
            }
            Element::Layered(node) => {
                let mut layers = Vec::new();
                for (index, layer) in node.layers.into_iter().enumerate() {
                    path.push(index);
                    if let Some(view) =
                        self.render_element(layer, dispatcher, path, context, live, effects)?
                    {
                        layers.push(view);
                    }
                    path.pop();
                }
                if layers.is_empty() {
                    Ok(Some(View::Empty))
                } else {
                    Ok(Some(View::Layered(LayersView { layers })))
                }
            }
            Element::Modal(node) => {
                path.push(0);
                let content =
                    self.render_element(*node.content, dispatcher, path, context, live, effects)?;
                path.pop();
                if let Some(content) = content {
                    Ok(Some(View::Modal(ModalView {
                        title: node.title,
                        content: Box::new(content),
                        width: node.width,
                        height: node.height,
                    })))
                } else {
                    Ok(Some(View::Empty))
                }
            }
            Element::ToastStack(node) => {
                if node.toasts.is_empty() {
                    return Ok(Some(View::Empty));
                }
                let toasts = node
                    .toasts
                    .into_iter()
                    .map(|toast| ToastView {
                        title: toast.title,
                        body: toast.body,
                        level: toast.level,
                    })
                    .collect();
                Ok(Some(View::ToastStack(ToastStackView { toasts })))
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

pub(crate) fn flatten_tree_items(items: Vec<TreeItemNode>) -> Vec<TreeRowView> {
    let mut rows = Vec::new();
    push_tree_items(items, 0, &mut rows);
    rows
}

fn spawn_stylesheet_watcher(path: PathBuf, tx: mpsc::Sender<AppMessage>) -> JoinHandle<()> {
    tokio::spawn(async move {
        info!(path = %path.display(), "stylesheet watcher started");
        let mut snapshot = fingerprint_if_exists(&path).await;
        loop {
            match maybe_reload_stylesheet(&path, &mut snapshot).await {
                Ok(Some(stylesheet)) => {
                    info!(path = %path.display(), "stylesheet change detected");
                    if tx
                        .send(AppMessage::StylesheetUpdated(stylesheet))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Ok(None) => {}
                Err(err) => warn!(path = %path.display(), error = ?err, "stylesheet reload failed"),
            }
            sleep(Duration::from_millis(400)).await;
        }
    })
}

async fn fingerprint_if_exists(path: &Path) -> Option<StylesheetSnapshot> {
    match fs::read_to_string(path).await {
        Ok(contents) => Some(StylesheetSnapshot {
            fingerprint: fingerprint(&contents),
        }),
        Err(_) => None,
    }
}

async fn maybe_reload_stylesheet(
    path: &Path,
    snapshot: &mut Option<StylesheetSnapshot>,
) -> anyhow::Result<Option<Arc<Stylesheet>>> {
    let contents = match fs::read_to_string(path).await {
        Ok(contents) => contents,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err.into()),
    };
    let fingerprint = fingerprint(&contents);
    if snapshot
        .as_ref()
        .map(|snap| snap.fingerprint == fingerprint)
        .unwrap_or(false)
    {
        return Ok(None);
    }
    let stylesheet = Stylesheet::parse(&contents)
        .with_context(|| format!("parse stylesheet {}", path.display()))?;
    *snapshot = Some(StylesheetSnapshot { fingerprint });
    Ok(Some(Arc::new(stylesheet)))
}

#[derive(Clone, Copy, Debug, Default)]
struct StylesheetSnapshot {
    fingerprint: u64,
}

fn fingerprint(input: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
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

async fn abort_and_log(label: &str, handle: JoinHandle<()>) {
    handle.abort();
    match handle.await {
        Ok(_) => trace!(task = label, "task aborted cleanly"),
        Err(err) if err.is_cancelled() => trace!(task = label, "task cancellation confirmed"),
        Err(err) => warn!(task = label, error = ?err, "task join failed"),
    }
}
