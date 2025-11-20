use rustact::interactions::is_button_click;
use rustact::runtime::{ButtonNode, Element, FlexDirection, FormFieldStatus, GaugeNode, TextInputNode};
use rustact::{FrameworkEvent, Scope};
use rustact::hooks::StateHandle;

pub fn root(ctx: &mut Scope) -> Element {
    let (count, set_count) = ctx.use_state(|| 0i32);
    let name = ctx.use_text_input("profile:name", || String::new());
    let name_status = ctx.use_text_input_validation(&name, |snapshot| {
        if snapshot.value.trim().is_empty() {
            FormFieldStatus::Warning
        } else {
            FormFieldStatus::Success
        }
    });

    ctx.use_effect((), move |dispatcher| {
        let mut events = dispatcher.events().subscribe();
        let decrement = set_count.clone();
        let increment = set_count.clone();
        tokio::spawn(async move {
            while let Ok(event) = events.recv().await {
                handle_event(&event, &decrement, &increment);
            }
        });
        None
    });

    Element::Flex(rustact::runtime::FlexNode {
        direction: FlexDirection::Column,
        children: vec![
            Element::text(format!("Hello, {}!", name.snapshot().value.trim())),
            Element::gauge(
                GaugeNode::new((count.abs() as f64) / 10.0)
                    .label(format!("Progress to ±10 ({count})")),
            ),
            Element::text_input(
                TextInputNode::new(name)
                    .label("Display name")
                    .placeholder("Rustacean")
                    .status(name_status),
            ),
            Element::Flex(rustact::runtime::FlexNode {
                direction: FlexDirection::Row,
                children: vec![
                    Element::button(ButtonNode::new("counter-minus", "-")),
                    Element::button(ButtonNode::new("counter-plus", "+")),
                ],
            }),
            Element::text(format!("Counter: {count}")),
        ],
    })
}

fn handle_event(event: &FrameworkEvent, decrement: &StateHandle<i32>, increment: &StateHandle<i32>) {
    match event {
        FrameworkEvent::Key(key) => match key.code {
            crossterm::event::KeyCode::Char('-') => decrement.update(|value| *value -= 1),
            crossterm::event::KeyCode::Char('+') => increment.update(|value| *value += 1),
            crossterm::event::KeyCode::Char('r') => {
                decrement.set(0);
            }
            _ => {}
        },
        FrameworkEvent::Mouse(_) => {
            if is_button_click(event, "counter-minus") {
                decrement.update(|value| *value -= 1);
            }
            if is_button_click(event, "counter-plus") {
                increment.update(|value| *value += 1);
            }
        }
        _ => {}
    }
}
