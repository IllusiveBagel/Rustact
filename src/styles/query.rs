#[derive(Clone, Copy, Debug)]
pub struct StyleQuery<'a> {
    pub(crate) element: &'a str,
    pub(crate) id: Option<&'a str>,
    pub(crate) classes: &'a [&'a str],
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
