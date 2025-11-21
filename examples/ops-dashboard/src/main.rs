use std::path::Path;
use std::time::Duration;

use crossterm::event::KeyCode;
use tokio::sync::broadcast::error::RecvError;
use tracing::warn;

use rustact::runtime::{AppConfig, Color, TabPaneNode};
use rustact::styles::Stylesheet;
use rustact::{
    App, Element, FormFieldNode, FormFieldStatus, FormNode, FrameworkEvent, GaugeNode, LayeredNode,
    ListItemNode, ListNode, ModalNode, Scope, StateHandle, TableCellNode, TableNode, TableRowNode,
    TabsNode, ToastLevel, ToastNode, ToastStackNode, component,
};

const APP_NAME: &str = "Rustact Ops Dashboard";
const OPS_STYLES: &str = include_str!("../styles/demo.css");
const OPS_STYLES_PATH: &str = "styles/demo.css";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stylesheet = load_ops_stylesheet();
    let mut app = App::new(APP_NAME, component("OpsRoot", ops_root))
        .with_config(AppConfig {
            tick_rate: Duration::from_millis(250),
        })
        .with_stylesheet(stylesheet);
    if should_watch_styles() {
        if Path::new(OPS_STYLES_PATH).exists() {
            app = app.watch_stylesheet(OPS_STYLES_PATH);
        } else {
            warn!(
                path = OPS_STYLES_PATH,
                "RUSTACT_WATCH_STYLES was set but stylesheet file was not found",
            );
        }
    }
    app.run().await
}

fn load_ops_stylesheet() -> Stylesheet {
    match Stylesheet::from_file(OPS_STYLES_PATH) {
        Ok(sheet) => sheet,
        Err(err) => {
            warn!(
                path = OPS_STYLES_PATH,
                error = ?err,
                "Unable to read stylesheet from disk, falling back to embedded CSS",
            );
            Stylesheet::parse(OPS_STYLES).expect("embedded ops stylesheet should parse")
        }
    }
}

fn should_watch_styles() -> bool {
    match std::env::var("RUSTACT_WATCH_STYLES") {
        Ok(value) => {
            let normalized = value.to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "on")
        }
        Err(_) => false,
    }
}

fn ops_root(ctx: &mut Scope) -> Element {
    let (active_tab, set_active_tab) = ctx.use_state(|| 0usize);
    let (logs, set_logs) = ctx.use_state(Vec::<String>::new);
    let (incident, set_incident) = ctx.use_state(|| None as Option<IncidentDetails>);
    let (toasts, set_toasts) = ctx.use_state(Vec::<ToastMessage>::new);

    let tab_handle = set_active_tab.clone();
    let log_handle = set_logs.clone();
    let incident_handle = set_incident.clone();
    let toast_handle = set_toasts.clone();
    ctx.use_effect((), move |dispatcher| {
        let mut events = dispatcher.events().subscribe();
        let handle = tokio::spawn(async move {
            let mut tick = 0usize;
            loop {
                match events.recv().await {
                    Ok(event) => match event {
                        FrameworkEvent::Tick => {
                            tick += 1;
                            log_handle.update(|entries| {
                                entries.push(format!(
                                    "tick #{tick}: updated {} workers",
                                    2 + (tick % 4)
                                ));
                                if entries.len() > 40 {
                                    entries.remove(0);
                                }
                            });
                            if tick % 18 == 0 {
                                let toast = ToastMessage::new("Deployment succeeded")
                                    .level(ToastLevel::Success)
                                    .body(format!("cluster-west finished wave {tick}"));
                                toast_handle.update(|stack| {
                                    stack.push(toast.clone());
                                    if stack.len() > 4 {
                                        stack.remove(0);
                                    }
                                });
                            }
                        }
                        FrameworkEvent::Key(key) => match key.code {
                            KeyCode::Char('1') => tab_handle.set(0),
                            KeyCode::Char('2') => tab_handle.set(1),
                            KeyCode::Char('i') => open_incident_modal(&incident_handle),
                            KeyCode::Esc => incident_handle.set(None),
                            KeyCode::Char('c') => {
                                toast_handle.update(|stack| {
                                    if !stack.is_empty() {
                                        stack.remove(0);
                                    }
                                });
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => break,
                }
            }
        });
        Some(Box::new(move || handle.abort()))
    });

    let base = Element::block(
        "Operations surface",
        Element::tabs(
            TabsNode::new(vec![
                TabPaneNode::new("Overview", overview_tab()),
                TabPaneNode::new("Logs", logs_tab(&logs)),
            ])
            .active(active_tab)
            .title("Panels"),
        ),
    );

    let mut layers = vec![base];
    if let Some(details) = incident.as_ref() {
        layers.push(build_incident_modal(details));
    }
    if !toasts.is_empty() {
        layers.push(build_toast_stack(&toasts));
    }

    Element::layers(LayeredNode::new(layers))
}

fn overview_tab() -> Element {
    let health = Element::table(
        TableNode::new(vec![
            TableRowNode::new(vec![
                TableCellNode::new("api").bold(),
                TableCellNode::new("Healthy").color(Color::Green),
                TableCellNode::new("351 req/s"),
            ]),
            TableRowNode::new(vec![
                TableCellNode::new("queue").bold(),
                TableCellNode::new("Degraded").color(Color::Yellow),
                TableCellNode::new("Workers catching up"),
            ]),
            TableRowNode::new(vec![
                TableCellNode::new("billing").bold(),
                TableCellNode::new("Failing").color(Color::Red),
                TableCellNode::new("Partner outage"),
            ]),
        ])
        .title("Cluster health"),
    );

    let release_form = Element::form(
        FormNode::new(vec![
            FormFieldNode::new("Region", "us-west-2"),
            FormFieldNode::new("Wave", "7 of 9").status(FormFieldStatus::Success),
            FormFieldNode::new("Error budget", "84%").status(FormFieldStatus::Warning),
        ])
        .title("Current deploy")
        .label_width(40),
    );

    let capacity = Element::gauge(GaugeNode::new(0.72).label("Capacity").color(Color::Cyan));

    Element::vstack(vec![
        Element::hstack(vec![health, release_form]),
        Element::block(
            "Capacity",
            Element::vstack(vec![Element::text("Compute saturation"), capacity]),
        ),
        Element::text("Keys: [1] Overview  [2] Logs  [i] Incident modal  [c] Dismiss toast"),
    ])
}

fn logs_tab(logs: &[String]) -> Element {
    let items = logs
        .iter()
        .rev()
        .take(20)
        .enumerate()
        .map(|(idx, line)| {
            ListItemNode::new(format!("#{idx} {line}")).color(if idx % 2 == 0 {
                Color::Gray
            } else {
                Color::White
            })
        })
        .collect();
    Element::list(
        ListNode::new(items)
            .title("Recent activity")
            .highlight_color(Color::LightCyan),
    )
}

fn build_incident_modal(details: &IncidentDetails) -> Element {
    let content = Element::vstack(vec![
        Element::text(format!("Incident #{}", details.id)),
        Element::text(format!("Status: {}", details.status)),
        Element::text(format!("Impact: {}", details.impact)),
        Element::text(format!("Started: {}", details.started)),
        Element::text(details.summary),
        Element::text("Press Esc to close"),
    ]);
    Element::modal(
        ModalNode::new(content)
            .title("Major incident")
            .width(60)
            .height(12),
    )
}

fn build_toast_stack(toasts: &[ToastMessage]) -> Element {
    let nodes = toasts
        .iter()
        .cloned()
        .map(|toast| {
            let node = ToastNode::new(toast.title).level(toast.level);
            if let Some(body) = toast.body {
                node.body(body)
            } else {
                node
            }
        })
        .collect();
    Element::toast_stack(ToastStackNode::new(nodes))
}

fn open_incident_modal(handle: &StateHandle<Option<IncidentDetails>>) {
    handle.set(Some(IncidentDetails {
        id: "4827",
        status: "Investigation",
        impact: "Elevated API latency",
        started: "08:41 UTC",
        summary: "Traffic shift to backup AZ introduced 120ms latency spike.",
    }));
}

#[derive(Clone)]
struct IncidentDetails {
    id: &'static str,
    status: &'static str,
    impact: &'static str,
    started: &'static str,
    summary: &'static str,
}

#[derive(Clone)]
struct ToastMessage {
    title: String,
    body: Option<String>,
    level: ToastLevel,
}

impl ToastMessage {
    fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: None,
            level: ToastLevel::Info,
        }
    }

    fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    fn level(mut self, level: ToastLevel) -> Self {
        self.level = level;
        self
    }
}
