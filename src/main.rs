use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use crossterm::event::KeyCode;
use crossterm::event::MouseButton;
use tokio::sync::broadcast::error::RecvError;
use tracing::warn;

use rustact::runtime::{AppConfig, Color, TextInputNode};
use rustact::styles::{ComputedStyle, StyleQuery, Stylesheet};
use rustact::{
    App, ButtonNode, Element, FormFieldNode, FormFieldStatus, FormNode, FrameworkEvent, GaugeNode,
    ListItemNode, ListNode, Scope, TableCellNode, TableNode, TableRowNode, TreeItemNode, TreeNode,
    component,
};
use rustact::{is_button_click, is_mouse_click, mouse_position, mouse_scroll_delta};

const APP_NAME: &str = "Rustact Demo";
const DEMO_STYLES: &str = include_str!("../styles/demo.css");
const DEMO_STYLES_PATH: &str = "styles/demo.css";
const COUNTER_MINUS_BUTTON: &str = "counter:minus";
const COUNTER_PLUS_BUTTON: &str = "counter:plus";
const COUNTER_GAUGE_ID: &str = "counter-progress";
const COUNTER_PANEL_ID: &str = "counter";
const STATS_LIST_ID: &str = "stats";
const SERVICES_TABLE_ID: &str = "services";
const RELEASE_FORM_ID: &str = "release";
const FEEDBACK_NAME_INPUT: &str = "feedback-name";
const FEEDBACK_EMAIL_INPUT: &str = "feedback-email";
const FEEDBACK_TOKEN_INPUT: &str = "feedback-token";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stylesheet = load_demo_stylesheet();
    let mut app = App::new(APP_NAME, component("AppRoot", app_root))
        .with_config(AppConfig {
            tick_rate: Duration::from_millis(200),
        })
        .with_stylesheet(stylesheet);
    if should_watch_styles() {
        if Path::new(DEMO_STYLES_PATH).exists() {
            app = app.watch_stylesheet(DEMO_STYLES_PATH);
        } else {
            warn!(
                path = DEMO_STYLES_PATH,
                "RUSTACT_WATCH_STYLES was set but stylesheet file was not found",
            );
        }
    }
    app.run().await
}

fn load_demo_stylesheet() -> Stylesheet {
    match Stylesheet::from_file(DEMO_STYLES_PATH) {
        Ok(sheet) => sheet,
        Err(err) => {
            warn!(
                path = DEMO_STYLES_PATH,
                error = ?err,
                "Unable to read stylesheet from disk, falling back to embedded CSS",
            );
            Stylesheet::parse(DEMO_STYLES).expect("embedded demo stylesheet should parse")
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

fn app_root(ctx: &mut Scope) -> Element {
    let root_style = ctx.styles().root();
    let _theme = ctx.provide_context(Theme {
        accent: root_style.color("--accent-color").unwrap_or(Color::Cyan),
        warning: root_style.color("--warning-color").unwrap_or(Color::Yellow),
        success: root_style.color("--success-color").unwrap_or(Color::Green),
        danger: root_style.color("--danger-color").unwrap_or(Color::Red),
        info: root_style.color("--info-color").unwrap_or(Color::Blue),
    });
    Element::block(
        "rustact demo",
        Element::vstack(vec![
            component("Hero", hero).into(),
            component("Meta", meta_banner).into(),
            Element::hstack(vec![
                component("Counter", counter_panel).into(),
                component("Stats", stats_panel).into(),
                component("Tips", tips_panel).into(),
            ]),
            Element::hstack(vec![
                component("Services", service_table).into(),
                component("ProjectTree", tree_panel).into(),
            ]),
            Element::hstack(vec![
                component("Events", event_log).into(),
                component("Config", config_form).into(),
                component("Feedback", feedback_panel).into(),
            ]),
        ]),
    )
}

fn hero(ctx: &mut Scope) -> Element {
    let theme = ctx
        .use_context::<Theme>()
        .unwrap_or_else(|| Arc::new(Theme::default()));
    let hero_style = ctx.styles().query(StyleQuery::element("hero"));
    let title_color = hero_style.color("color").unwrap_or(theme.accent);
    let subtitle_color = hero_style
        .color("--subtitle-color")
        .unwrap_or(Color::DarkGray);
    Element::vstack(vec![
        Element::colored_text("Welcome to rustact", title_color),
        Element::colored_text(
            "Press '+' / '-' or click the on-screen buttons to adjust the counter",
            subtitle_color,
        ),
        Element::colored_text(
            "Use mouse scroll to browse stats; click buttons for actions",
            subtitle_color,
        ),
        Element::colored_text("Press Ctrl+C to quit", subtitle_color),
        Element::colored_text("Edit styles/demo.css to reskin the UI", subtitle_color),
    ])
}

fn counter_panel(ctx: &mut Scope) -> Element {
    let (count, counter) = ctx.use_reducer(
        || 0i32,
        |state, action: CounterAction| match action {
            CounterAction::Increment => *state += 1,
            CounterAction::Decrement => *state -= 1,
            CounterAction::Reset => *state = 0,
        },
    );
    let summary = {
        let value = count;
        ctx.use_memo(value, move || CounterSummary::new(value))
    };
    let theme = ctx
        .use_context::<Theme>()
        .unwrap_or_else(|| Arc::new(Theme::default()));
    let panel_style = ctx
        .styles()
        .query(StyleQuery::element("panel").with_id(COUNTER_PANEL_ID));
    let instructions_color = panel_style.color("color").unwrap_or(theme.info);
    let plus_style = ctx
        .styles()
        .query(StyleQuery::element("button").with_id(COUNTER_PLUS_BUTTON));
    let minus_style = ctx
        .styles()
        .query(StyleQuery::element("button").with_id(COUNTER_MINUS_BUTTON));
    let gauge_style = ctx
        .styles()
        .query(StyleQuery::element("gauge").with_id(COUNTER_GAUGE_ID));
    let plus_accent = plus_style.color("accent-color").unwrap_or(theme.accent);
    let plus_filled = plus_style.bool("--filled").unwrap_or(true);
    let minus_accent = minus_style.color("accent-color").unwrap_or(theme.danger);
    let minus_filled = minus_style.bool("--filled").unwrap_or(true);
    let gauge_color = gauge_style.color("color").unwrap_or(theme.accent);
    let gauge_label = gauge_style
        .text("--label")
        .map(|label| label.to_string())
        .unwrap_or_else(|| "Progress to ±10".to_string());

    let key_handler = ctx.use_callback((), move || {
        let reducer = counter.clone();
        move |event: &FrameworkEvent| {
            match event {
                FrameworkEvent::Key(key) => match key.code {
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        reducer.dispatch(CounterAction::Increment)
                    }
                    KeyCode::Char('-') => reducer.dispatch(CounterAction::Decrement),
                    KeyCode::Char('r') => reducer.dispatch(CounterAction::Reset),
                    KeyCode::Char('q') => return false,
                    _ => {}
                },
                FrameworkEvent::Mouse(_) => {
                    if is_button_click(event, COUNTER_PLUS_BUTTON) {
                        reducer.dispatch(CounterAction::Increment);
                        return true;
                    }
                    if is_button_click(event, COUNTER_MINUS_BUTTON) {
                        reducer.dispatch(CounterAction::Decrement);
                        return true;
                    }
                }
                _ => {}
            }
            true
        }
    });

    ctx.use_effect((), move |dispatcher| {
        let handler = key_handler.clone();
        let mut events = dispatcher.events().subscribe();
        let handle = tokio::spawn(async move {
            loop {
                match events.recv().await {
                    Ok(event) => {
                        if !handler(&event) {
                            break;
                        }
                    }
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => break,
                }
            }
        });
        Some(Box::new(move || handle.abort()))
    });

    Element::block(
        "Counter",
        Element::vstack(vec![
            Element::text(summary.label.clone()),
            Element::text(format!("Parity: {}", summary.parity)),
            Element::gauge(
                GaugeNode::new(summary.normalized())
                    .label(gauge_label)
                    .color(gauge_color),
            ),
            Element::hstack(vec![
                Element::button(
                    ButtonNode::new(COUNTER_MINUS_BUTTON, "-")
                        .accent(minus_accent)
                        .filled(minus_filled),
                ),
                Element::button(
                    ButtonNode::new(COUNTER_PLUS_BUTTON, "+")
                        .accent(plus_accent)
                        .filled(plus_filled),
                ),
            ]),
            Element::colored_text(
                "Keys: +/-/r/q • Click buttons to adjust",
                instructions_color,
            ),
        ]),
    )
}

fn event_log(ctx: &mut Scope) -> Element {
    let (status, set_status) = ctx.use_state(EventStatus::default);
    let updater = set_status.clone();
    ctx.use_effect((), move |dispatcher| {
        let mut events = dispatcher.events().subscribe();
        let handle = tokio::spawn(async move {
            loop {
                match events.recv().await {
                    Ok(event) => updater.update(|state| state.record(&event)),
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => break,
                }
            }
        });
        Some(Box::new(move || handle.abort()))
    });

    Element::block(
        "Events",
        Element::vstack(vec![
            Element::text(format!("Last event: {}", status.description)),
            Element::text(format!("Tick count: {}", status.ticks)),
        ]),
    )
}

fn stats_panel(ctx: &mut Scope) -> Element {
    let (events, set_events) = ctx.use_state(Vec::<String>::new);
    let (selected, set_selected) = ctx.use_state(|| 0usize);
    let total_events = ctx.use_ref(|| 0usize);
    let theme = ctx
        .use_context::<Theme>()
        .unwrap_or_else(|| Arc::new(Theme::default()));
    let list_style = ctx
        .styles()
        .query(StyleQuery::element("list").with_id(STATS_LIST_ID));
    let max_items = list_style.u16("--max-items").unwrap_or(10) as usize;
    let highlight_color = list_style
        .color("--highlight-color")
        .unwrap_or(theme.warning);
    let instruction_color = list_style.color("color").unwrap_or(theme.info);

    let feed = set_events.clone();
    let selection = set_selected.clone();
    let total_ref = total_events.clone();
    let max_items_limit = max_items.max(1);
    ctx.use_effect((), move |dispatcher| {
        let mut stream = dispatcher.events().subscribe();
        let max_items = max_items_limit;
        let handle = tokio::spawn(async move {
            while let Ok(event) = stream.recv().await {
                let label = match &event {
                    FrameworkEvent::Key(key) => format!("Key: {:?}", key.code),
                    FrameworkEvent::Mouse(mouse) => format!("Mouse: {:?}", mouse.kind),
                    FrameworkEvent::Resize(w, h) => format!("Resize: {w}x{h}"),
                    FrameworkEvent::Tick => "Tick".to_string(),
                };

                let mut new_len = 0usize;
                feed.update(|list| {
                    if list.len() >= max_items {
                        list.remove(0);
                    }
                    list.push(label);
                    new_len = list.len();
                });
                total_ref.with_mut(|count| *count += 1);

                match &event {
                    FrameworkEvent::Mouse(_) => {
                        let delta = mouse_scroll_delta(&event);
                        if delta != 0 {
                            selection.update(|sel| {
                                if delta > 0 {
                                    *sel = sel.saturating_sub(delta as usize);
                                } else {
                                    let steps = delta.unsigned_abs() as usize;
                                    *sel = sel.saturating_add(steps);
                                }
                                if *sel >= new_len {
                                    *sel = new_len.saturating_sub(1);
                                }
                            });
                        } else if is_mouse_click(&event, MouseButton::Left) {
                            if new_len > 0 {
                                if let Some((col, row)) = mouse_position(&event) {
                                    let seed = col as usize + row as usize;
                                    selection.set(seed % new_len);
                                }
                            }
                        } else {
                            selection.update(|sel| {
                                if *sel >= new_len {
                                    *sel = new_len.saturating_sub(1);
                                }
                            });
                        }
                    }
                    _ => selection.set(new_len.saturating_sub(1)),
                }
            }
        });
        Some(Box::new(move || handle.abort()))
    });

    let total_seen = total_events.with(|count| *count);

    let list_items = events
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let color = if idx % 2 == 0 {
                Color::Yellow
            } else {
                Color::Blue
            };
            ListItemNode::new(format!("#{idx}: {entry}")).color(color)
        })
        .collect::<Vec<_>>();

    let mut list = ListNode::new(list_items)
        .title("Recent events (scroll to navigate)")
        .highlight_color(highlight_color);
    if !events.is_empty() {
        let max_index = events.len().saturating_sub(1);
        let highlight = selected.min(max_index);
        list = list.highlight(highlight);
    }

    Element::block(
        "Stats",
        Element::vstack(vec![
            Element::colored_text(
                "Mouse scroll cycles entries; left click jumps between them.",
                instruction_color,
            ),
            Element::text(format!("Events observed (use_ref): {total_seen}")),
            Element::list(list),
        ]),
    )
}

fn meta_banner(ctx: &mut Scope) -> Element {
    let accent = ctx
        .use_context::<Theme>()
        .map(|theme| theme.accent)
        .unwrap_or(Color::Magenta);
    let version = env!("CARGO_PKG_VERSION");
    Element::block(
        "Framework overview",
        Element::fragment(vec![
            Element::colored_text(format!("Running {APP_NAME} v{version}"), accent),
            Element::text("Hooks: state, reducer, ref, memo, callback, effect, context"),
            Element::text("Widgets: text, flex, buttons, gauges, lists, tables, trees, forms"),
        ]),
    )
}

fn service_table(ctx: &mut Scope) -> Element {
    let theme = ctx
        .use_context::<Theme>()
        .unwrap_or_else(|| Arc::new(Theme::default()));
    let table_style = ctx
        .styles()
        .query(StyleQuery::element("table").with_id(SERVICES_TABLE_ID));
    let rows = vec![
        TableRowNode::new(vec![
            TableCellNode::new("api").bold(),
            TableCellNode::new("Healthy").color(theme.success),
            TableCellNode::new("320 req/s"),
        ]),
        TableRowNode::new(vec![
            TableCellNode::new("jobs").bold(),
            TableCellNode::new("Degraded").color(theme.warning),
            TableCellNode::new("Backlog growing"),
        ]),
        TableRowNode::new(vec![
            TableCellNode::new("billing").bold(),
            TableCellNode::new("Offline").color(theme.danger),
            TableCellNode::new("Investigating outage"),
        ]),
    ];

    let mut table = TableNode::new(rows)
        .header(TableRowNode::new(vec![
            TableCellNode::new("Service").bold(),
            TableCellNode::new("Status").bold(),
            TableCellNode::new("Notes").bold(),
        ]))
        .highlight(1);

    if let Some(widths) = table_style.list_u16("--column-widths") {
        table = table.widths(widths);
    }

    Element::block("Services", Element::table(table))
}

fn tree_panel(_ctx: &mut Scope) -> Element {
    let tree = TreeNode::new(vec![
        TreeItemNode::new("src").children(vec![
            TreeItemNode::new("main.rs"),
            TreeItemNode::new("runtime").children(vec![TreeItemNode::new("mod.rs")]),
            TreeItemNode::new("renderer").children(vec![TreeItemNode::new("mod.rs")]),
        ]),
        TreeItemNode::new("docs").children(vec![
            TreeItemNode::new("README.md"),
            TreeItemNode::new("architecture.md"),
        ]),
        TreeItemNode::new("Cargo.toml").expanded(false),
    ])
    .title("Workspace tree")
    .highlight(2);

    Element::block("Project", Element::tree(tree))
}

fn config_form(ctx: &mut Scope) -> Element {
    let form_style = ctx
        .styles()
        .query(StyleQuery::element("form").with_id(RELEASE_FORM_ID));
    let label_width = form_style.u16("--label-width").unwrap_or(35);
    let fields = vec![
        FormFieldNode::new("Environment", "production"),
        FormFieldNode::new("Version", "v0.4.7"),
        FormFieldNode::new("Migrations", "pending").status(FormFieldStatus::Warning),
        FormFieldNode::new("Smoke tests", "failing").status(FormFieldStatus::Error),
        FormFieldNode::new("Approver", "ops-team").status(FormFieldStatus::Success),
    ];

    let form = FormNode::new(fields)
        .title("Release checklist")
        .label_width(label_width);
    Element::block("Config", Element::form(form))
}

fn feedback_panel(ctx: &mut Scope) -> Element {
    let theme = ctx
        .use_context::<Theme>()
        .unwrap_or_else(|| Arc::new(Theme::default()));
    let name_input = ctx.use_text_input(FEEDBACK_NAME_INPUT, || "Rusty User".to_string());
    let email_input = ctx.use_text_input(FEEDBACK_EMAIL_INPUT, String::new);
    let token_input = ctx.use_text_input(FEEDBACK_TOKEN_INPUT, String::new);

    let name_status_kind = ctx.use_text_input_validation(&name_input, |snapshot| {
        if snapshot.value.trim().is_empty() {
            FormFieldStatus::Warning
        } else {
            FormFieldStatus::Success
        }
    });
    let email_status_kind = ctx.use_text_input_validation(&email_input, |snapshot| {
        let trimmed = snapshot.value.trim();
        if trimmed.is_empty() {
            FormFieldStatus::Normal
        } else if trimmed.contains('@') {
            FormFieldStatus::Success
        } else {
            FormFieldStatus::Error
        }
    });
    let token_status_kind = ctx.use_text_input_validation(&token_input, |snapshot| {
        if snapshot.value.is_empty() {
            FormFieldStatus::Warning
        } else {
            FormFieldStatus::Success
        }
    });

    let name_snapshot = name_input.snapshot();
    let email_snapshot = email_input.snapshot();
    let token_snapshot = token_input.snapshot();

    let input_style = |id: &str| ctx.styles().query(StyleQuery::element("input").with_id(id));
    let style_input = |mut node: TextInputNode, styles: &ComputedStyle| {
        if let Some(color) = styles.color("accent-color") {
            node = node.accent(color);
        }
        if let Some(color) = styles.color("--border-color") {
            node = node.border_color(color);
        }
        if let Some(color) = styles.color("color") {
            node = node.text_color(color);
        }
        if let Some(color) = styles.color("--placeholder-color") {
            node = node.placeholder_color(color);
        }
        if let Some(color) = styles.color("--background-color") {
            node = node.background_color(color);
        }
        if let Some(color) = styles.color("--focus-background") {
            node = node.focus_background(color);
        }
        node
    };

    let name_status = match name_status_kind {
        FormFieldStatus::Warning => {
            "Type your display name above to personalize the message.".to_string()
        }
        _ => format!(
            "Hello, {}! ({} chars)",
            name_snapshot.value.trim(),
            name_snapshot.value.chars().count()
        ),
    };
    let email_trimmed = email_snapshot.value.trim();
    let email_status = match email_status_kind {
        FormFieldStatus::Normal => "Add an email to receive follow-ups.".to_string(),
        FormFieldStatus::Success => format!("We will follow up at {}.", email_trimmed),
        FormFieldStatus::Error => format!("\"{}\" doesn't look like an email.", email_trimmed),
        _ => String::new(),
    };
    let token_status = match token_status_kind {
        FormFieldStatus::Warning => "API token not provided.".to_string(),
        _ => format!(
            "Captured token length: {} chars.",
            token_snapshot.value.chars().count()
        ),
    };

    let name_styles = input_style(FEEDBACK_NAME_INPUT);
    let email_styles = input_style(FEEDBACK_EMAIL_INPUT);
    let token_styles = input_style(FEEDBACK_TOKEN_INPUT);

    let name_field = style_input(
        TextInputNode::new(name_input.clone())
            .label("Display name")
            .placeholder("Rustacean in Residence")
            .width(32)
            .accent(theme.accent),
        &name_styles,
    );
    let email_field = style_input(
        TextInputNode::new(email_input.clone())
            .label("Email (optional)")
            .placeholder("dev@example.com")
            .width(36),
        &email_styles,
    );
    let token_field = style_input(
        TextInputNode::new(token_input.clone())
            .label("API token")
            .placeholder("Optional secret")
            .secure(true)
            .width(36)
            .accent(theme.warning),
        &token_styles,
    );

    Element::block(
        "Feedback",
        Element::vstack(vec![
            Element::text("Try the new text inputs:"),
            Element::text_input(name_field),
            Element::text_input(email_field),
            Element::text_input(token_field),
            Element::text(format!("{name_status} {email_status} {token_status}")),
        ]),
    )
}

fn tips_panel(ctx: &mut Scope) -> Element {
    let tips = ctx.use_memo((), || DEMO_TIPS.to_vec());
    let cards: Vec<Element> = tips
        .iter()
        .enumerate()
        .map(|(index, tip)| {
            let props = *tip;
            let key = format!("tip:{index}:{}", props.id);
            component("TipCard", move |ctx| tip_card(ctx, props))
                .key(key)
                .into()
        })
        .collect();

    Element::block("Tips", Element::fragment(cards))
}

fn tip_card(ctx: &mut Scope, tip: Tip) -> Element {
    let theme = ctx
        .use_context::<Theme>()
        .unwrap_or_else(|| Arc::new(Theme::default()));
    let classes = [tip.class];
    let tip_style = ctx
        .styles()
        .query(StyleQuery::element("tip").with_classes(&classes));
    let accent = tip_style.color("color").unwrap_or(theme.info);
    Element::block(
        tip.title,
        Element::fragment(vec![
            Element::colored_text(tip.title, accent),
            Element::text(tip.body),
        ]),
    )
}

#[derive(Clone)]
struct Theme {
    accent: Color,
    warning: Color,
    success: Color,
    danger: Color,
    info: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent: Color::Cyan,
            warning: Color::Yellow,
            success: Color::Green,
            danger: Color::Red,
            info: Color::Blue,
        }
    }
}

#[derive(Clone, Default)]
struct EventStatus {
    description: String,
    ticks: usize,
}

impl EventStatus {
    fn record(&mut self, event: &FrameworkEvent) {
        match event {
            FrameworkEvent::Key(key) => {
                self.description = format!("Key: {:?}", key.code);
            }
            FrameworkEvent::Mouse(mouse) => {
                self.description = format!("Mouse: {:?}", mouse.kind);
            }
            FrameworkEvent::Resize(w, h) => {
                self.description = format!("Resize: {w}x{h}");
            }
            FrameworkEvent::Tick => {
                self.description = "Tick".into();
                self.ticks += 1;
                return;
            }
        }
        self.ticks = 0;
    }
}

#[derive(Clone)]
struct CounterSummary {
    value: i32,
    label: String,
    parity: &'static str,
}

impl CounterSummary {
    fn new(value: i32) -> Self {
        let parity = if value % 2 == 0 { "even" } else { "odd" };
        Self {
            value,
            label: format!("Current count: {value}"),
            parity,
        }
    }

    fn normalized(&self) -> f64 {
        (self.value as f64 / 10.0).abs().min(1.0)
    }
}

#[derive(Clone, Copy)]
enum CounterAction {
    Increment,
    Decrement,
    Reset,
}

#[derive(Clone, Copy)]
struct Tip {
    id: &'static str,
    title: &'static str,
    body: &'static str,
    class: &'static str,
}

const DEMO_TIPS: [Tip; 3] = [
    Tip {
        id: "keys",
        title: "Keyboard",
        body: "Use +/-/r/q to control the counter and exit.",
        class: "keyboard",
    },
    Tip {
        id: "mouse",
        title: "Mouse",
        body: "Scroll to move list focus or click the counter buttons.",
        class: "mouse",
    },
    Tip {
        id: "context",
        title: "Context",
        body: "The banner and tips share the accent color via provide_context.",
        class: "context",
    },
];
