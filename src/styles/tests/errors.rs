use crate::styles::Stylesheet;

#[test]
fn parse_fails_when_selector_repeats_id_segment() {
    let css = "button#submit#again { color: red; }";
    let err = Stylesheet::parse(css).expect_err("expected duplicate id failure");
    assert!(err.to_string().contains("selector already has id"));
}

#[test]
fn parse_fails_when_selector_repeats_class_segment() {
    let css = ".primary.secondary.secondary { color: blue; }";
    let err = Stylesheet::parse(css).expect_err("expected duplicate class failure");
    assert!(err.to_string().contains("selector already has class"));
}
