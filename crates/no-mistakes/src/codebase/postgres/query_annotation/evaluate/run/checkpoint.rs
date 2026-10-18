use super::super::{Environment, Evaluator, Scope, Value};
use crate::fx::{FxHashMap, FxHashSet};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

// Exactly the fields historically captured by run; events/next_builder and
// active callback/alternative contexts intentionally remain outside rollback.
pub(super) struct Checkpoint {
    scopes: Vec<Scope>,
    modules: FxHashMap<PathBuf, Environment>,
    invalidated_builders: FxHashSet<u64>,
    deleted_argument_slots: FxHashSet<(u64, Option<usize>)>,
    mapped_arguments: FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    fresh_mapped_parameters: FxHashMap<Environment, FxHashSet<String>>,
    argument_objects: FxHashMap<u64, Vec<Value>>,
    argument_extra_slots: FxHashMap<u64, BTreeMap<usize, Value>>,
    definite_deleted_argument_slots: FxHashSet<(u64, usize)>,
    recreated_argument_slots: FxHashSet<(u64, usize)>,
    disconnected_argument_slots: FxHashSet<(u64, usize)>,
    builder_updates: FxHashMap<u64, Value>,
    untrusted_array_intrinsics: bool,
    captured_bindings: FxHashMap<Environment, FxHashMap<String, Environment>>,
    captured_binding_readers: FxHashMap<Environment, FxHashMap<String, FxHashSet<Environment>>>,
    mapped_argument_owners: FxHashMap<u64, Environment>,
    mapped_parameter_indices: FxHashMap<u64, FxHashMap<String, usize>>,
}

impl Checkpoint {
    pub(super) fn capture<F>(e: &Evaluator<'_, F>) -> Self {
        super::operations::capture();
        Self {
            scopes: e.scopes.clone(),
            modules: e.modules.clone(),
            invalidated_builders: e.invalidated_builders.clone(),
            deleted_argument_slots: e.deleted_argument_slots.clone(),
            mapped_arguments: e.mapped_arguments.clone(),
            fresh_mapped_parameters: e.fresh_mapped_parameters.clone(),
            argument_objects: e.argument_objects.clone(),
            argument_extra_slots: e.argument_extra_slots.clone(),
            definite_deleted_argument_slots: e.definite_deleted_argument_slots.clone(),
            recreated_argument_slots: e.recreated_argument_slots.clone(),
            disconnected_argument_slots: e.disconnected_argument_slots.clone(),
            builder_updates: e.builder_updates.clone(),
            untrusted_array_intrinsics: e.untrusted_array_intrinsics,
            captured_bindings: e.captured_bindings.clone(),
            captured_binding_readers: e.captured_binding_readers.clone(),
            mapped_argument_owners: e.mapped_argument_owners.clone(),
            mapped_parameter_indices: e.mapped_parameter_indices.clone(),
        }
    }
    pub(super) fn restore<F: Fn(&str, &Path) -> Option<PathBuf>>(&self, e: &mut Evaluator<'_, F>) {
        super::operations::restore();
        e.scopes.clone_from(&self.scopes);
        e.modules.clone_from(&self.modules);
        e.invalidated_builders
            .clone_from(&self.invalidated_builders);
        e.deleted_argument_slots
            .clone_from(&self.deleted_argument_slots);
        e.mapped_arguments.clone_from(&self.mapped_arguments);
        e.fresh_mapped_parameters
            .clone_from(&self.fresh_mapped_parameters);
        e.rebuild_fresh_mapped_argument_bindings();
        super::operations::fresh();
        e.argument_objects.clone_from(&self.argument_objects);
        e.argument_extra_slots
            .clone_from(&self.argument_extra_slots);
        e.definite_deleted_argument_slots
            .clone_from(&self.definite_deleted_argument_slots);
        e.recreated_argument_slots
            .clone_from(&self.recreated_argument_slots);
        e.disconnected_argument_slots
            .clone_from(&self.disconnected_argument_slots);
        e.builder_updates.clone_from(&self.builder_updates);
        e.untrusted_array_intrinsics = self.untrusted_array_intrinsics;
        e.captured_bindings.clone_from(&self.captured_bindings);
        e.captured_binding_readers
            .clone_from(&self.captured_binding_readers);
        e.mapped_argument_owners
            .clone_from(&self.mapped_argument_owners);
        e.mapped_parameter_indices
            .clone_from(&self.mapped_parameter_indices);
    }
}
