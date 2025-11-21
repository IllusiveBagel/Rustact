use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use parking_lot::Mutex;

use crate::runtime::{ComponentId, Dispatcher};
use crate::text_input::{TextInputHandle, TextInputs};

pub(crate) type AnySlot = dyn Any + Send + Sync;
pub type Cleanup = Box<dyn FnOnce() + Send + Sync>;

#[derive(Default)]
pub struct HookRegistry {
    stores: Mutex<HashMap<ComponentId, Arc<Mutex<HookStore>>>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            stores: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn store_for(&self, id: &ComponentId) -> Arc<Mutex<HookStore>> {
        let mut guard = self.stores.lock();
        guard
            .entry(id.clone())
            .or_insert_with(|| Arc::new(Mutex::new(HookStore::default())))
            .clone()
    }

    pub fn prune(&self, live: &HashSet<ComponentId>) {
        let mut guard = self.stores.lock();
        guard.retain(|id, store| {
            if live.contains(id) {
                true
            } else {
                store.lock().drain();
                false
            }
        });
    }

    pub fn with_effect_slot<F, R>(&self, id: &ComponentId, slot_index: usize, f: F) -> R
    where
        F: FnOnce(&mut EffectHook) -> R,
    {
        let store = self.store_for(id);
        let mut guard = store.lock();
        let slot = guard.slot(slot_index);
        if !matches!(slot, HookSlot::Effect(_)) {
            if matches!(slot, HookSlot::Vacant) {
                *slot = HookSlot::Effect(EffectHook::default());
            } else {
                panic!("effect slot type mismatch");
            }
        }
        match slot {
            HookSlot::Effect(effect) => f(effect),
            _ => unreachable!(),
        }
    }
}

#[derive(Default)]
pub(crate) struct HookStore {
    slots: Vec<HookSlot>,
}

impl HookStore {
    pub(crate) fn slot(&mut self, index: usize) -> &mut HookSlot {
        while self.slots.len() <= index {
            self.slots.push(HookSlot::Vacant);
        }
        &mut self.slots[index]
    }

    pub(crate) fn drain(&mut self) {
        for slot in &mut self.slots {
            match slot {
                HookSlot::Effect(effect) => {
                    if let Some(cleanup) = effect.cleanup.take() {
                        cleanup();
                    }
                }
                HookSlot::TextInput(entry) => {
                    if let Some(binding) = entry.downcast_mut::<TextInputEntry>() {
                        binding.release();
                    }
                }
                _ => {}
            }
        }
        self.slots.clear();
    }
}

#[derive(Default)]
pub(crate) enum HookSlot {
    #[default]
    Vacant,
    State(Box<AnySlot>),
    Effect(EffectHook),
    Memo(Box<AnySlot>),
    Reducer(Box<AnySlot>),
    RefCell(Box<AnySlot>),
    TextInput(Box<AnySlot>),
}

#[derive(Default)]
pub struct EffectHook {
    pub(crate) deps: Option<Box<AnySlot>>,
    cleanup: Option<Cleanup>,
}

impl EffectHook {
    pub(crate) fn take_cleanup(&mut self) -> Option<Cleanup> {
        self.cleanup.take()
    }

    pub(crate) fn set_cleanup(&mut self, cleanup: Option<Cleanup>) {
        self.cleanup = cleanup;
    }

    pub(crate) fn set_deps(&mut self, deps: Box<AnySlot>) {
        self.deps = Some(deps);
    }
}

pub struct EffectInvocation {
    pub component_id: ComponentId,
    pub slot_index: usize,
    pub deps: Box<AnySlot>,
    pub task: Box<dyn FnOnce(Dispatcher) -> Option<Cleanup> + Send + Sync>,
}

pub(crate) struct TextInputEntry {
    id: String,
    handle: TextInputHandle,
}

impl TextInputEntry {
    pub(crate) fn new(id: String, handle: TextInputHandle) -> Self {
        Self { id, handle }
    }

    pub(crate) fn release(&mut self) {
        if !self.id.is_empty() {
            TextInputs::unregister_binding(&self.id);
            self.id.clear();
        }
    }

    pub(crate) fn handle(&self) -> TextInputHandle {
        self.handle.clone()
    }

    pub(crate) fn ensure_id(&self, id: &str) {
        if self.id != id {
            panic!(
                "use_text_input hook ID mismatch: expected {}, received {}",
                self.id, id
            );
        }
    }
}
