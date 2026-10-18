use super::super::{Evaluator, File, Value};
use crate::fx::FxHashMap;
use std::path::PathBuf;

pub(super) fn assert_same_state<F, G>(old: &Evaluator<'_, F>, new: &Evaluator<'_, G>) {
    // Value/Values equality ignores memo/cache identity. Scope deref equality
    // protects semantics without requiring equal Arc addresses or warm caches.
    assert_eq!(old.scopes.len(), new.scopes.len());
    for (old, new) in old.scopes.iter().zip(&new.scopes) {
        assert!(**old == **new);
    }
    macro_rules! same { ($($field:ident),+ $(,)?) => { $(assert!(old.$field == new.$field, stringify!($field));)+ }; }
    same!(
        modules,
        invalidated_builders,
        deleted_argument_slots,
        mapped_arguments,
        fresh_mapped_parameters,
        argument_objects,
        argument_extra_slots,
        definite_deleted_argument_slots,
        recreated_argument_slots,
        disconnected_argument_slots,
        builder_updates,
        untrusted_array_intrinsics,
        captured_bindings,
        captured_binding_readers,
        mapped_argument_owners,
        mapped_parameter_indices,
        fresh_mapped_argument_bindings,
        events,
        next_builder,
        active_callback_functions,
        active_callback_executions
    );
    // A completed root must leave no active alternative or transient context.
    assert!(old.active_module_initials.is_empty() && new.active_module_initials.is_empty());
    assert!(old.transient_roots.is_empty() && new.transient_roots.is_empty());
}

// Test constructor copied from the fresh-main project evaluator; no dependency
// on the unmerged initial-template constructor or observer field.
pub(super) fn new<'a, F>(
    files: &'a FxHashMap<PathBuf, File<'a>>,
    resolve: F,
    untrusted_array_intrinsics: bool,
) -> Evaluator<'a, F> {
    Evaluator {
        files,
        resolve,
        untrusted_array_intrinsics,
        events: Default::default(),
        scopes: Vec::new(),
        modules: crate::fx::fx_map(),
        active_module_initials: Default::default(),
        transient_roots: Default::default(),
        active_callback_functions: Default::default(),
        active_callback_executions: Default::default(),
        next_builder: 0,
        invalidated_builders: Default::default(),
        builder_updates: Default::default(),
        captured_bindings: Default::default(),
        captured_binding_readers: Default::default(),
        mapped_argument_owners: Default::default(),
        mapped_parameter_indices: Default::default(),
        deleted_argument_slots: Default::default(),
        mapped_arguments: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        argument_objects: Default::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        recreated_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
    }
}

// Require a complete runtime SQL value; references/containers and an unsafe
// possibility beside Unknown do not prove that a fixture executes exact SQL.
pub(super) fn known_prefixes(value: &Value) -> Option<&str> {
    match value {
        Value::Prefix(text, true, _) => Some(text.as_str()),
        Value::Evaluated(value, _) => known_prefixes(value),
        _ => None,
    }
}
