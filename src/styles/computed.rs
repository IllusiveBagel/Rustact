use std::collections::HashMap;

use crate::runtime::Color;

use super::parser::parse_color;

#[derive(Clone, Debug, Default)]
pub struct ComputedStyle {
    props: HashMap<String, String>,
}

impl ComputedStyle {
    pub(crate) fn from_props(props: HashMap<String, String>) -> Self {
        Self { props }
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.props
            .get(&name.to_ascii_lowercase())
            .map(|s| s.as_str())
    }

    pub fn color(&self, name: &str) -> Option<Color> {
        self.get(name).and_then(parse_color)
    }

    pub fn bool(&self, name: &str) -> Option<bool> {
        self.get(name)
            .and_then(|value| match value.to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => Some(true),
                "false" | "0" | "no" | "off" => Some(false),
                _ => None,
            })
    }

    pub fn u16(&self, name: &str) -> Option<u16> {
        self.get(name)?.trim().parse().ok()
    }

    pub fn f64(&self, name: &str) -> Option<f64> {
        self.get(name)?.trim().parse().ok()
    }

    pub fn text(&self, name: &str) -> Option<&str> {
        self.get(name)
    }

    pub fn list_u16(&self, name: &str) -> Option<Vec<u16>> {
        let value = self.get(name)?;
        let mut out = Vec::new();
        for chunk in value.split(|c: char| c == ',' || c.is_ascii_whitespace()) {
            if chunk.trim().is_empty() {
                continue;
            }
            if let Ok(num) = chunk.trim().parse::<u16>() {
                out.push(num);
            }
        }
        if out.is_empty() { None } else { Some(out) }
    }

    pub fn is_empty(&self) -> bool {
        self.props.is_empty()
    }
}
