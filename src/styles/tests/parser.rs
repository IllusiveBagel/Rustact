use crate::runtime::Color;
use crate::styles::parser::{parse_color, parse_declarations, strip_comments};

#[test]
fn strips_block_comments() {
    let input = "color: red; /* remove me */ width: 10;";
    assert_eq!(strip_comments(input), "color: red;  width: 10;");
}

#[test]
fn parses_declarations_with_quotes() {
    let props = parse_declarations("label: \"Submit\"; width: 10;");
    assert_eq!(props.get("label").unwrap(), "Submit");
    assert_eq!(props.get("width").unwrap(), "10");
}

#[test]
fn parses_hex_and_rgb_colors() {
    assert_eq!(parse_color("#ff0000"), Some(Color::Rgb(255, 0, 0)));
    assert_eq!(parse_color("#0f0"), Some(Color::Rgb(0, 255, 0)));
    assert_eq!(parse_color("rgb(10,20,30)"), Some(Color::Rgb(10, 20, 30)));
}
