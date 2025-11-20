use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::hooks::registry::{EffectHook, HookRegistry, HookSlot};
use crate::runtime::ComponentId;

#[test]
fn prune_runs_effect_cleanup_and_drops_store() {
    let registry = HookRegistry::new();
    let component = ComponentId::new(&[0], "Test", None);
    let flag = Arc::new(AtomicBool::new(false));
    {
        let store = registry.store_for(&component);
        let mut guard = store.lock();
        let slot = guard.slot(0);
        *slot = HookSlot::Effect(EffectHook::default());
        if let HookSlot::Effect(effect) = slot {
            let flag = flag.clone();
            effect.set_cleanup(Some(Box::new(move || flag.store(true, Ordering::SeqCst))));
        }
    }

    registry.prune(&HashSet::new());
    assert!(flag.load(Ordering::SeqCst));
}

#[test]
fn with_effect_slot_initializes_vacant_entries() {
    let registry = HookRegistry::new();
    let component = ComponentId::new(&[1], "Comp", None);

    registry.with_effect_slot(&component, 2, |effect| {
        assert!(effect.deps.is_none());
        effect.set_deps(Box::new(123usize));
    });

    let store = registry.store_for(&component);
    let mut guard = store.lock();
    match guard.slot(2) {
        HookSlot::Effect(effect) => {
            assert!(effect.deps.is_some());
        }
        _ => panic!("expected effect slot"),
    }
}
