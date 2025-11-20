use std::fmt;
use std::sync::Arc;

use crate::hooks::Scope;

use super::element::Element;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComponentId(pub(crate) String);

impl ComponentId {
    pub fn new(path: &[usize], name: &str, key: Option<&str>) -> Self {
        let mut id = path
            .iter()
            .map(|segment| segment.to_string())
            .collect::<Vec<_>>()
            .join(".");
        if let Some(key) = key {
            id.push('#');
            id.push_str(key);
        }
        id.push(':');
        id.push_str(name);
        Self(id)
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type ComponentFn = Arc<dyn Fn(&mut Scope) -> Element + Send + Sync>;

#[derive(Clone)]
pub struct ComponentElement {
    pub(crate) name: &'static str,
    pub(crate) key: Option<String>,
    pub(crate) render: ComponentFn,
}

impl ComponentElement {
    pub fn new<F>(name: &'static str, render: F) -> Self
    where
        F: Fn(&mut Scope) -> Element + Send + Sync + 'static,
    {
        Self {
            name,
            key: None,
            render: Arc::new(render),
        }
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl From<ComponentElement> for Element {
    fn from(value: ComponentElement) -> Self {
        Element::Component(value)
    }
}

impl fmt::Debug for ComponentElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComponentElement")
            .field("name", &self.name)
            .field("key", &self.key)
            .finish()
    }
}

pub fn component<F>(name: &'static str, render: F) -> ComponentElement
where
    F: Fn(&mut Scope) -> Element + Send + Sync + 'static,
{
    ComponentElement::new(name, render)
}
