use crate::runtime::Color;
use crate::styles::{StyleQuery, Stylesheet};

#[test]
fn parses_stylesheet_and_applies_root_properties() {
    let css = r"
        :root { padding: 4; color: #111; }
        button { color: #222; }
    ";
    let sheet = Stylesheet::parse(css).expect("parse css");
    assert!(!sheet.is_empty());
    let root = sheet.root();
    assert_eq!(root.u16("padding"), Some(4));
    assert_eq!(root.color("color"), Some(Color::Rgb(17, 17, 17)));

    let button = sheet.query(StyleQuery::element("button"));
    assert_eq!(button.color("color"), Some(Color::Rgb(34, 34, 34)));
    assert_eq!(button.u16("padding"), Some(4));
}

#[test]
fn specificity_and_order_control_overrides() {
    let css = r"
        button { color: blue; }
        button.primary { color: red; }
        #submit { color: green; }
        button.primary { border: 1; }
    ";
    let sheet = Stylesheet::parse(css).expect("parse css");
    let classes: [&str; 1] = ["primary"];
    let query = StyleQuery::element("button")
        .with_id("submit")
        .with_classes(&classes);
    let style = sheet.query(query);
    assert_eq!(style.color("color"), Some(Color::Green));
    assert_eq!(style.u16("border"), Some(1));
}
