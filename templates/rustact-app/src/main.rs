mod components;

use anyhow::Result;
use rustact::{component, App};
use rustact::runtime::Element;
use rustact::styles::Stylesheet;

use components::root::root;

#[tokio::main]
async fn main() -> Result<()> {
    let stylesheet = Stylesheet::parse(include_str!("../styles/app.css"))?;
    App::new("{{ project-name }}", component("Root", root))
        .with_stylesheet(stylesheet)
        .run()
        .await
}
