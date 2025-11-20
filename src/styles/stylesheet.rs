use std::collections::HashMap;

use anyhow::{Result, anyhow};

use super::computed::ComputedStyle;
use super::parser::{parse_declarations, strip_comments};
use super::query::StyleQuery;

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
        ComputedStyle::from_props(self.root.clone())
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
        ComputedStyle::from_props(props)
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

fn merge_maps(into: &mut HashMap<String, String>, from: &HashMap<String, String>) {
    for (key, value) in from {
        into.insert(key.to_ascii_lowercase(), value.clone());
    }
}
