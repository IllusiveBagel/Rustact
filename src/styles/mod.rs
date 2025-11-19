use std::collections::HashMap;

use anyhow::{Result, anyhow};

use crate::runtime::Color;

#[derive(Clone, Debug, Default)]
pub struct Stylesheet {
    root: HashMap<String, String>,
    rules: Vec<StyleRule>,
}

impl Stylesheet {
    pub fn parse(input: &str) -> Result<Self> {
        let mut sheet = Stylesheet::default();
        let mut order = 0usize;
        let cleaned = strip_comments(input);
        for block in cleaned.split('}') {
            if block.trim().is_empty() {
                continue;
            }
            let (selector_raw, body_raw) = match block.split_once('{') {
                Some(pair) => pair,
                None => continue,
            };
            let selector_raw = selector_raw.trim();
            if selector_raw.is_empty() {
                continue;
            }
            let declarations = parse_declarations(body_raw);
            for selector in selector_raw.split(',') {
                let selector = selector.trim();
                if selector.is_empty() {
                    continue;
                }
                if selector == ":root" {
                    merge_maps(&mut sheet.root, &declarations);
                    continue;
                }
                let selector = Selector::parse(selector)?;
                sheet.rules.push(StyleRule {
                    selector,
                    declarations: declarations.clone(),
                    order,
                });
                order += 1;
            }
        }
        Ok(sheet)
    }

    pub fn root(&self) -> ComputedStyle {
        ComputedStyle {
            props: self.root.clone(),
        }
    }

    pub fn query<'a>(&'a self, query: StyleQuery<'a>) -> ComputedStyle {
        let mut props = self.root.clone();
        let mut matches: Vec<&StyleRule> = self
            .rules
            .iter()
            .filter(|rule| rule.selector.matches(&query))
            .collect();
        matches.sort_by(|a, b| {
            a.selector
                .specificity()
                .cmp(&b.selector.specificity())
                .then(a.order.cmp(&b.order))
        });
        for rule in matches {
            merge_maps(&mut props, &rule.declarations);
        }
        ComputedStyle { props }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_empty() && self.rules.is_empty()
    }
}

#[derive(Clone, Debug)]
struct StyleRule {
    selector: Selector,
    declarations: HashMap<String, String>,
    order: usize,
}

#[derive(Clone, Debug, Default)]
struct Selector {
    element: Option<String>,
    id: Option<String>,
    class: Option<String>,
}

#[derive(Clone, Copy)]
enum SegmentTarget {
    Element,
    Id,
    Class,
}

impl Selector {
    fn parse(raw: &str) -> Result<Self> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("empty selector"));
        }
        let mut selector = Selector::default();
        let mut current = String::new();
        let mut mode = SegmentTarget::Element;
        for ch in trimmed.chars() {
            match ch {
                '#' => {
                    selector.push_segment(&mut current, mode)?;
                    mode = SegmentTarget::Id;
                }
                '.' => {
                    selector.push_segment(&mut current, mode)?;
                    mode = SegmentTarget::Class;
                }
                _ => current.push(ch),
            }
        }
        selector.push_segment(&mut current, mode)?;
        if selector
            .element
            .as_ref()
            .map(|s| s.is_empty())
            .unwrap_or(false)
        {
            selector.element = None;
        }
        Ok(selector)
    }

    fn push_segment(&mut self, buffer: &mut String, mode: SegmentTarget) -> Result<()> {
        let value = buffer.trim();
        if value.is_empty() {
            buffer.clear();
            return Ok(());
        }
        let value = value.to_string();
        match mode {
            SegmentTarget::Element => {
                if self.element.is_some() {
                    return Err(anyhow!("selector already has element"));
                }
                self.element = Some(value.to_ascii_lowercase());
            }
            SegmentTarget::Id => {
                if self.id.is_some() {
                    return Err(anyhow!("selector already has id"));
                }
                self.id = Some(value);
            }
            SegmentTarget::Class => {
                if self.class.is_some() {
                    return Err(anyhow!("selector already has class"));
                }
                self.class = Some(value.to_ascii_lowercase());
            }
        }
        buffer.clear();
        Ok(())
    }

    fn matches(&self, query: &StyleQuery<'_>) -> bool {
        if let Some(element) = self.element.as_ref() {
            if query.element.is_empty() {
                return false;
            }
            if !element.eq_ignore_ascii_case(query.element) {
                return false;
            }
        }
        if let Some(id) = self.id.as_ref() {
            if query.id != Some(id.as_str()) {
                return false;
            }
        }
        if let Some(class) = self.class.as_ref() {
            if !query
                .classes
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(class))
            {
                return false;
            }
        }
        true
    }

    fn specificity(&self) -> (u8, u8, u8) {
        (
            if self.id.is_some() { 1 } else { 0 },
            if self.class.is_some() { 1 } else { 0 },
            if self.element.is_some() { 1 } else { 0 },
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StyleQuery<'a> {
    element: &'a str,
    id: Option<&'a str>,
    classes: &'a [&'a str],
}

impl<'a> StyleQuery<'a> {
    pub fn element(element: &'a str) -> Self {
        Self {
            element,
            id: None,
            classes: &[],
        }
    }

    pub fn with_id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_classes(mut self, classes: &'a [&'a str]) -> Self {
        self.classes = classes;
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct ComputedStyle {
    props: HashMap<String, String>,
}

impl ComputedStyle {
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

fn merge_maps(into: &mut HashMap<String, String>, from: &HashMap<String, String>) {
    for (key, value) in from {
        into.insert(key.to_ascii_lowercase(), value.clone());
    }
}

fn parse_declarations(body: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for declaration in body.split(';') {
        if let Some((name, value)) = declaration.split_once(':') {
            let key = name.trim().to_ascii_lowercase();
            if key.is_empty() {
                continue;
            }
            let value = clean_value(value.trim());
            map.insert(key, value);
        }
    }
    map
}

fn clean_value(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

fn strip_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i += 2;
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    result
}

fn parse_color(value: &str) -> Option<Color> {
    let trimmed = value.trim();
    if trimmed.starts_with('#') {
        let hex = &trimmed[1..];
        return parse_hex_color(hex);
    }
    if let Some(inner) = trimmed
        .strip_prefix("rgb(")
        .and_then(|v| v.strip_suffix(')'))
    {
        let parts: Vec<u8> = inner
            .split(',')
            .filter_map(|part| part.trim().parse::<u8>().ok())
            .collect();
        if parts.len() == 3 {
            return Some(Color::Rgb(parts[0], parts[1], parts[2]));
        }
    }
    named_color(trimmed)
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(Color::Rgb(r, g, b))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::Rgb(r, g, b))
        }
        _ => None,
    }
}

fn named_color(value: &str) -> Option<Color> {
    match value.to_ascii_lowercase().as_str() {
        "black" => Some(Color::Black),
        "white" => Some(Color::White),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "blue" => Some(Color::Blue),
        "yellow" => Some(Color::Yellow),
        "cyan" => Some(Color::Cyan),
        "magenta" => Some(Color::Magenta),
        "gray" | "grey" => Some(Color::Gray),
        "lightgray" | "lightgrey" => Some(Color::DarkGray),
        _ => None,
    }
}
