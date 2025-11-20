use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use tokio::sync::mpsc;

use crate::context::ContextStack;
use crate::events::{DEFAULT_TICK_RATE, EventBus};
use crate::hooks::{EffectInvocation, HookRegistry, Scope};
use crate::renderer::Renderer;
use crate::styles::Stylesheet;
use crate::text_input::TextInputs;

use super::component::{ComponentElement, ComponentId};
use super::dispatcher::{AppMessage, Dispatcher};
use super::element::{Element, FlexDirection, TreeItemNode};
use super::tasks::{spawn_shutdown_watcher, spawn_terminal_events, spawn_tick_loop};
use super::view::{
    BlockView, ButtonView, FlexView, FormFieldView, FormView, GaugeView, ListItemView, ListView,
    TableCellView, TableRowView, TableView, TextInputView, TextView, TreeRowView, TreeView, View,
};

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

pub(crate) fn flatten_tree_items(items: Vec<TreeItemNode>) -> Vec<TreeRowView> {
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
