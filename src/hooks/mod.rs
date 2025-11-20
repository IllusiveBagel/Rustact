use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use parking_lot::Mutex;

use crate::context::{ContextGuard, ContextStack};
use crate::runtime::{ComponentId, Dispatcher, FormFieldStatus};
use crate::styles::Stylesheet;
use crate::text_input::{TextInputHandle, TextInputSnapshot, TextInputs};

type AnySlot = dyn Any + Send + Sync;
type ReducerFn<S, A> = dyn Fn(&mut S, A) + Send + Sync + 'static;

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

    pub fn store_for(&self, id: &ComponentId) -> Arc<Mutex<HookStore>> {
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
pub struct HookStore {
    slots: Vec<HookSlot>,
}

impl HookStore {
    fn slot(&mut self, index: usize) -> &mut HookSlot {
        while self.slots.len() <= index {
            self.slots.push(HookSlot::Vacant);
        }
        &mut self.slots[index]
    }

    pub fn drain(&mut self) {
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

enum HookSlot {
    Vacant,
    State(Box<AnySlot>),
    Effect(EffectHook),
    Memo(Box<AnySlot>),
    Reducer(Box<AnySlot>),
    RefCell(Box<AnySlot>),
    TextInput(Box<AnySlot>),
}

impl Default for HookSlot {
    fn default() -> Self {
        HookSlot::Vacant
    }
}

#[derive(Default)]
pub struct EffectHook {
    deps: Option<Box<AnySlot>>,
    cleanup: Option<Cleanup>,
}

pub type Cleanup = Box<dyn FnOnce() + Send + Sync>;

pub struct EffectInvocation {
    pub component_id: ComponentId,
    pub slot_index: usize,
    pub deps: Box<AnySlot>,
    pub task: Box<dyn FnOnce(Dispatcher) -> Option<Cleanup> + Send + Sync>,
}

pub struct Scope<'a> {
    component_id: ComponentId,
    store: Arc<Mutex<HookStore>>,
    dispatcher: Dispatcher,
    hook_cursor: usize,
    context: &'a mut ContextStack,
    pending_effects: Vec<EffectInvocation>,
    styles: Arc<Stylesheet>,
}

impl<'a> Scope<'a> {
    pub(crate) fn new(
        component_id: ComponentId,
        store: Arc<Mutex<HookStore>>,
        dispatcher: Dispatcher,
        context: &'a mut ContextStack,
        styles: Arc<Stylesheet>,
    ) -> Self {
        Self {
            component_id,
            store,
            dispatcher,
            hook_cursor: 0,
            context,
            pending_effects: Vec::new(),
            styles,
        }
    }

    pub fn use_state<T, F>(&mut self, init: F) -> (T, StateHandle<T>)
    where
        T: Clone + Send + 'static,
        F: FnOnce() -> T,
    {
        let index = self.next_index();
        let shared = {
            let mut store = self.store.lock();
            let slot = store.slot(index);
            match slot {
                HookSlot::Vacant => {
                    let state = Arc::new(Mutex::new(init()));
                    *slot = HookSlot::State(Box::new(state.clone()));
                    state
                }
                HookSlot::State(existing) => existing
                    .downcast_ref::<Arc<Mutex<T>>>()
                    .expect("use_state hook order mismatch")
                    .clone(),
                _ => panic!("use_state hook order mismatch"),
            }
        };
        let value = shared.lock().clone();
        let handle = StateHandle::new(shared, self.dispatcher.clone());
        (value, handle)
    }

    pub fn use_effect<D, F>(&mut self, deps: D, effect: F)
    where
        D: PartialEq + Clone + Send + Sync + 'static,
        F: FnOnce(Dispatcher) -> Option<Cleanup> + Send + Sync + 'static,
    {
        let index = self.next_index();
        let should_run = {
            let mut store = self.store.lock();
            let slot = store.slot(index);
            match slot {
                HookSlot::Vacant => {
                    *slot = HookSlot::Effect(EffectHook::default());
                    true
                }
                HookSlot::Effect(effect_slot) => effect_slot
                    .deps
                    .as_ref()
                    .and_then(|value| value.downcast_ref::<D>())
                    .map(|existing| existing != &deps)
                    .unwrap_or(true),
                _ => panic!("use_effect hook order mismatch"),
            }
        };

        if should_run {
            self.pending_effects.push(EffectInvocation {
                component_id: self.component_id.clone(),
                slot_index: index,
                deps: Box::new(deps),
                task: Box::new(effect),
            });
        }
    }

    pub fn provide_context<T>(&mut self, value: T) -> ContextGuard<'_>
    where
        T: Send + Sync + 'static,
    {
        self.context.provide(value)
    }

    pub fn use_context<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.context.get::<T>()
    }

    pub fn use_memo<T, D, F>(&mut self, deps: D, compute: F) -> Arc<T>
    where
        T: Send + Sync + 'static,
        D: PartialEq + Clone + Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        let index = self.next_index();
        let result = {
            let mut store = self.store.lock();
            let slot = store.slot(index);
            match slot {
                HookSlot::Vacant => {
                    let value = Arc::new(compute());
                    *slot = HookSlot::Memo(Box::new(MemoEntry::new(deps.clone(), value.clone())));
                    value
                }
                HookSlot::Memo(entry) => entry
                    .downcast_mut::<MemoEntry>()
                    .expect("use_memo hook order mismatch")
                    .apply_or_update(deps, compute),
                _ => panic!("use_memo hook order mismatch"),
            }
        };
        result
    }

    pub fn use_callback<T, D, F>(&mut self, deps: D, factory: F) -> Arc<T>
    where
        T: Send + Sync + 'static,
        D: PartialEq + Clone + Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        self.use_memo(deps, factory)
    }

    pub fn use_reducer<S, A, Init, R>(
        &mut self,
        init: Init,
        reducer: R,
    ) -> (S, ReducerDispatch<S, A>)
    where
        S: Clone + Send + 'static,
        A: Send + 'static,
        Init: FnOnce() -> S,
        R: Fn(&mut S, A) + Send + Sync + 'static,
    {
        let index = self.next_index();
        let (shared, driver) = {
            let mut store = self.store.lock();
            let slot = store.slot(index);
            match slot {
                HookSlot::Vacant => {
                    let state = Arc::new(Mutex::new(init()));
                    let reducer = into_reducer_arc(reducer);
                    *slot = HookSlot::Reducer(Box::new(ReducerEntry::new(
                        state.clone(),
                        reducer.clone(),
                    )));
                    (state, reducer)
                }
                HookSlot::Reducer(entry) => {
                    let entry = entry
                        .downcast_mut::<ReducerEntry<S, A>>()
                        .expect("use_reducer hook order mismatch");
                    let reducer = into_reducer_arc(reducer);
                    entry.update_reducer(reducer.clone());
                    (entry.state.clone(), entry.reducer.clone())
                }
                _ => panic!("use_reducer hook order mismatch"),
            }
        };
        let value = shared.lock().clone();
        let handle = ReducerDispatch::new(shared, driver, self.dispatcher.clone());
        (value, handle)
    }

    pub fn use_ref<T, Init>(&mut self, init: Init) -> RefHandle<T>
    where
        T: Send + 'static,
        Init: FnOnce() -> T,
    {
        let index = self.next_index();
        let shared = {
            let mut store = self.store.lock();
            let slot = store.slot(index);
            match slot {
                HookSlot::Vacant => {
                    let handle = Arc::new(Mutex::new(init()));
                    *slot = HookSlot::RefCell(Box::new(RefEntry::new(handle.clone())));
                    handle
                }
                HookSlot::RefCell(entry) => entry
                    .downcast_mut::<RefEntry<T>>()
                    .expect("use_ref hook order mismatch")
                    .handle
                    .clone(),
                _ => panic!("use_ref hook order mismatch"),
            }
        };
        RefHandle::new(shared)
    }

    pub fn use_text_input<F>(&mut self, id: impl Into<String>, init: F) -> TextInputHandle
    where
        F: FnOnce() -> String,
    {
        let index = self.next_index();
        let id = id.into();
        let dispatcher = self.dispatcher.clone();
        let handle = {
            let mut store = self.store.lock();
            let slot = store.slot(index);
            match slot {
                HookSlot::Vacant => {
                    let handle = TextInputHandle::new(id.clone(), init(), dispatcher);
                    *slot = HookSlot::TextInput(Box::new(TextInputEntry::new(id, handle.clone())));
                    handle
                }
                HookSlot::TextInput(entry) => {
                    let entry = entry
                        .downcast_mut::<TextInputEntry>()
                        .expect("use_text_input hook order mismatch");
                    entry.ensure_id(&id);
                    entry.handle()
                }
                _ => panic!("use_text_input hook order mismatch"),
            }
        };
        handle
    }

    pub fn use_text_input_validation<F>(
        &mut self,
        handle: &TextInputHandle,
        validator: F,
    ) -> FormFieldStatus
    where
        F: Fn(&TextInputSnapshot) -> FormFieldStatus,
    {
        let snapshot = handle.snapshot();
        let status = validator(&snapshot);
        handle.set_status(status);
        status
    }

    pub fn dispatcher(&self) -> &Dispatcher {
        &self.dispatcher
    }

    pub fn styles(&self) -> &Stylesheet {
        &self.styles
    }

    pub(crate) fn take_effects(&mut self) -> Vec<EffectInvocation> {
        std::mem::take(&mut self.pending_effects)
    }

    fn next_index(&mut self) -> usize {
        let current = self.hook_cursor;
        self.hook_cursor += 1;
        current
    }
}

#[derive(Clone)]
pub struct StateHandle<T: Send + 'static> {
    shared: Arc<Mutex<T>>,
    dispatcher: Dispatcher,
}

impl<T: Send + 'static> StateHandle<T> {
    fn new(shared: Arc<Mutex<T>>, dispatcher: Dispatcher) -> Self {
        Self { shared, dispatcher }
    }

    pub fn set(&self, next: T) {
        *self.shared.lock() = next;
        self.dispatcher.request_render();
    }

    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        f(&mut *self.shared.lock());
        self.dispatcher.request_render();
    }
}

#[derive(Clone)]
pub struct ReducerDispatch<S: Send + 'static, A: Send + 'static> {
    shared: Arc<Mutex<S>>,
    reducer: Arc<ReducerFn<S, A>>,
    dispatcher: Dispatcher,
}

impl<S: Send + 'static, A: Send + 'static> ReducerDispatch<S, A> {
    fn new(shared: Arc<Mutex<S>>, reducer: Arc<ReducerFn<S, A>>, dispatcher: Dispatcher) -> Self {
        Self {
            shared,
            reducer,
            dispatcher,
        }
    }

    pub fn dispatch(&self, action: A) {
        {
            let mut state = self.shared.lock();
            (self.reducer)(&mut state, action);
        }
        self.dispatcher.request_render();
    }

    pub fn with_state<R>(&self, f: impl FnOnce(&S) -> R) -> R {
        let state = self.shared.lock();
        f(&state)
    }
}

#[derive(Clone)]
pub struct RefHandle<T: Send + 'static> {
    shared: Arc<Mutex<T>>,
}

impl<T: Send + 'static> RefHandle<T> {
    fn new(shared: Arc<Mutex<T>>) -> Self {
        Self { shared }
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let value = self.shared.lock();
        f(&value)
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut value = self.shared.lock();
        f(&mut value)
    }

    pub fn set(&self, next: T) {
        *self.shared.lock() = next;
    }
}

struct MemoEntry {
    deps: Box<AnySlot>,
    value: Box<AnySlot>,
}

struct ReducerEntry<S: Send + 'static, A: Send + 'static> {
    state: Arc<Mutex<S>>,
    reducer: Arc<ReducerFn<S, A>>,
}

impl<S: Send + 'static, A: Send + 'static> ReducerEntry<S, A> {
    fn new(state: Arc<Mutex<S>>, reducer: Arc<ReducerFn<S, A>>) -> Self {
        Self { state, reducer }
    }

    fn update_reducer(&mut self, reducer: Arc<ReducerFn<S, A>>) {
        self.reducer = reducer;
    }
}

struct RefEntry<T: Send + 'static> {
    handle: Arc<Mutex<T>>,
}

impl<T: Send + 'static> RefEntry<T> {
    fn new(handle: Arc<Mutex<T>>) -> Self {
        Self { handle }
    }
}

struct TextInputEntry {
    id: String,
    handle: TextInputHandle,
}

impl TextInputEntry {
    fn new(id: String, handle: TextInputHandle) -> Self {
        Self { id, handle }
    }

    fn release(&mut self) {
        if !self.id.is_empty() {
            TextInputs::unregister_binding(&self.id);
            self.id.clear();
        }
    }

    fn handle(&self) -> TextInputHandle {
        self.handle.clone()
    }

    fn ensure_id(&self, id: &str) {
        if self.id != id {
            panic!(
                "use_text_input hook ID mismatch: expected {}, received {}",
                self.id, id
            );
        }
    }
}

fn into_reducer_arc<S, A, R>(reducer: R) -> Arc<ReducerFn<S, A>>
where
    S: Send + 'static,
    A: Send + 'static,
    R: Fn(&mut S, A) + Send + Sync + 'static,
{
    Arc::new(move |state: &mut S, action: A| reducer(state, action))
}

impl MemoEntry {
    fn new<D, T>(deps: D, value: Arc<T>) -> Self
    where
        D: Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        Self {
            deps: Box::new(deps),
            value: Box::new(value),
        }
    }

    fn apply_or_update<T, D, F>(&mut self, deps: D, compute: F) -> Arc<T>
    where
        T: Send + Sync + 'static,
        D: PartialEq + Clone + Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        let should_recompute = self
            .deps
            .as_ref()
            .downcast_ref::<D>()
            .map(|existing| existing != &deps)
            .unwrap_or(true);

        if should_recompute {
            let value = Arc::new(compute());
            self.deps = Box::new(deps);
            self.value = Box::new(value.clone());
            value
        } else {
            self.value
                .as_ref()
                .downcast_ref::<Arc<T>>()
                .expect("use_memo stored value mismatch")
                .clone()
        }
    }
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
