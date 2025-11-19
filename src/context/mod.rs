use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

type AnyArc = Arc<dyn Any + Send + Sync>;

#[derive(Default, Debug)]
pub struct ContextStack {
    layers: HashMap<TypeId, Vec<AnyArc>>,
}

impl ContextStack {
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
        }
    }

    pub fn provide<T>(&mut self, value: T) -> ContextGuard<'_>
    where
        T: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let entry = self.layers.entry(type_id).or_default();
        entry.push(Arc::new(value));
        ContextGuard {
            stack: self,
            type_id,
        }
    }

    pub fn get<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.layers
            .get(&TypeId::of::<T>())
            .and_then(|entries| entries.last())
            .and_then(|arc| arc.clone().downcast::<T>().ok())
    }

    fn pop(&mut self, type_id: TypeId) {
        if let Some(stack) = self.layers.get_mut(&type_id) {
            stack.pop();
            if stack.is_empty() {
                self.layers.remove(&type_id);
            }
        }
    }
}

pub struct ContextGuard<'a> {
    stack: &'a mut ContextStack,
    type_id: TypeId,
}

impl Drop for ContextGuard<'_> {
    fn drop(&mut self) {
        self.stack.pop(self.type_id);
    }
}
