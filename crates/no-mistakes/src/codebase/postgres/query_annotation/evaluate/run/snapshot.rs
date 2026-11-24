use super::super::{
    module_cache::ModuleCache, ArgumentObjects, CapturedBindings, Environment, Evaluator, Scope,
    Value,
};
use crate::codebase::postgres::query_annotation::module_ids::ModuleId;
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;
use std::path::Path;

pub(super) struct RunSnapshot {
    pub(super) scopes: Vec<Scope>,
    pub(super) modules: ModuleCache,
    invalidated: FxHashSet<u64>,
    deleted: FxHashSet<(u64, Option<usize>)>,
    mapped: FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    fresh: FxHashMap<Environment, FxHashSet<String>>,
    objects: ArgumentObjects,
    extra_slots: FxHashMap<u64, BTreeMap<usize, Value>>,
    definite: FxHashSet<(u64, usize)>,
    recreated: FxHashSet<(u64, usize)>,
    disconnected: FxHashSet<(u64, usize)>,
    builder_updates: FxHashMap<u64, Value>,
    untrusted_array_intrinsics: bool,
    captured_bindings: CapturedBindings,
    captured_binding_readers: FxHashMap<Environment, FxHashMap<String, FxHashSet<Environment>>>,
    mapped_argument_owners: FxHashMap<u64, Environment>,
    mapped_parameter_indices: FxHashMap<u64, FxHashMap<String, usize>>,
}

impl RunSnapshot {
    pub(super) fn capture<F>(evaluator: &Evaluator<'_, F>) -> Self {
        let timing = super::super::super::progress::timing("postgres.annotation.snapshot_clone");
        let snapshot = Self {
            scopes: evaluator.scopes.clone(),
            modules: evaluator.modules.clone(),
            invalidated: evaluator.invalidated_builders.clone(),
            deleted: evaluator.deleted_argument_slots.clone(),
            mapped: evaluator.mapped_arguments.clone(),
            fresh: evaluator.fresh_mapped_parameters.clone(),
            objects: evaluator.argument_objects.clone(),
            extra_slots: evaluator.argument_extra_slots.clone(),
            definite: evaluator.definite_deleted_argument_slots.clone(),
            recreated: evaluator.recreated_argument_slots.clone(),
            disconnected: evaluator.disconnected_argument_slots.clone(),
            builder_updates: evaluator.builder_updates.clone(),
            untrusted_array_intrinsics: evaluator.untrusted_array_intrinsics,
            captured_bindings: evaluator.captured_bindings.clone(),
            captured_binding_readers: evaluator.captured_binding_readers.clone(),
            mapped_argument_owners: evaluator.mapped_argument_owners.clone(),
            mapped_parameter_indices: evaluator.mapped_parameter_indices.clone(),
        };
        drop(timing);
        snapshot
    }

    pub(super) fn restore<F: Fn(&str, &Path) -> Option<std::path::PathBuf>>(
        &self,
        evaluator: &mut Evaluator<'_, F>,
    ) {
        let timing = super::super::super::progress::timing("postgres.annotation.snapshot_restore");
        evaluator.scopes.clone_from(&self.scopes);
        evaluator.modules.clone_from(&self.modules);
        evaluator.invalidated_builders.clone_from(&self.invalidated);
        evaluator.deleted_argument_slots.clone_from(&self.deleted);
        evaluator.mapped_arguments.clone_from(&self.mapped);
        evaluator.fresh_mapped_parameters.clone_from(&self.fresh);
        evaluator.rebuild_fresh_mapped_argument_bindings();
        evaluator.argument_objects.clone_from(&self.objects);
        evaluator.argument_extra_slots.clone_from(&self.extra_slots);
        evaluator
            .definite_deleted_argument_slots
            .clone_from(&self.definite);
        evaluator
            .recreated_argument_slots
            .clone_from(&self.recreated);
        evaluator
            .disconnected_argument_slots
            .clone_from(&self.disconnected);
        evaluator.builder_updates.clone_from(&self.builder_updates);
        evaluator.untrusted_array_intrinsics = self.untrusted_array_intrinsics;
        evaluator
            .captured_bindings
            .clone_from(&self.captured_bindings);
        evaluator
            .captured_binding_readers
            .clone_from(&self.captured_binding_readers);
        evaluator
            .mapped_argument_owners
            .clone_from(&self.mapped_argument_owners);
        evaluator
            .mapped_parameter_indices
            .clone_from(&self.mapped_parameter_indices);
        drop(timing);
    }

    /// Promote a module into the root snapshot only when its initialization
    /// leaves every pre-existing scope, builder, argument, capture, and event
    /// untouched so unrelated entrypoints cannot observe its side effects.
    pub(super) fn try_promote_module<F: Fn(&str, &Path) -> Option<std::path::PathBuf>>(
        &mut self,
        evaluator: &mut Evaluator<'_, F>,
        target: &Path,
        unpromotable: &mut FxHashSet<ModuleId>,
    ) {
        let id = evaluator.module_ids().id(target);
        if self.modules.contains_key(&id) || unpromotable.contains(&id) {
            return;
        }
        let before_events = evaluator.event_log.len();
        let before_next_builder = evaluator.next_builder;
        evaluator.module_environment(target);
        if self.is_pure_extension(evaluator, before_events, before_next_builder) {
            self.scopes.clone_from(&evaluator.scopes);
            self.modules.clone_from(&evaluator.modules);
            return;
        }
        rollback_events(evaluator, before_events);
        unpromotable.insert(id);
        self.restore(evaluator);
    }

    pub(super) fn promote_loaded<F: Fn(&str, &Path) -> Option<std::path::PathBuf>>(
        &mut self,
        evaluator: &mut Evaluator<'_, F>,
        unpromotable: &mut FxHashSet<ModuleId>,
    ) -> bool {
        if evaluator.modules.len() <= self.modules.len() {
            return false;
        }
        let mut candidates = evaluator
            .modules
            .keys()
            .copied()
            .filter(|id| !self.modules.contains_key(id) && !unpromotable.contains(id))
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            return false;
        }
        candidates.sort_unstable();
        self.restore(evaluator);
        for id in candidates {
            if self.modules.contains_key(&id) || unpromotable.contains(&id) {
                continue;
            }
            let target = evaluator.module_ids().path(id).to_path_buf();
            self.try_promote_module(evaluator, &target, unpromotable);
        }
        true
    }

    fn is_pure_extension<F>(
        &self,
        evaluator: &Evaluator<'_, F>,
        before_events: usize,
        before_next_builder: u64,
    ) -> bool {
        let prefix = self.scopes.len();
        evaluator.event_log.len() == before_events
            && evaluator.next_builder == before_next_builder
            && evaluator.scopes.len() >= prefix
            && evaluator.scopes[..prefix]
                .iter()
                .zip(&self.scopes)
                .all(|(current, base)| current.same_bindings(base))
            && evaluator.invalidated_builders == self.invalidated
            && evaluator.builder_updates == self.builder_updates
            && evaluator.deleted_argument_slots == self.deleted
            && evaluator.definite_deleted_argument_slots == self.definite
            && evaluator.disconnected_argument_slots == self.disconnected
            && evaluator.recreated_argument_slots == self.recreated
            && evaluator.mapped_arguments == self.mapped
            && evaluator.fresh_mapped_parameters == self.fresh
            && evaluator.argument_objects == self.objects
            && evaluator.argument_extra_slots == self.extra_slots
            && evaluator.captured_bindings == self.captured_bindings
            && evaluator.captured_binding_readers == self.captured_binding_readers
            && evaluator.mapped_argument_owners == self.mapped_argument_owners
            && evaluator.mapped_parameter_indices == self.mapped_parameter_indices
            && evaluator.untrusted_array_intrinsics == self.untrusted_array_intrinsics
    }
}

fn rollback_events<F>(evaluator: &mut Evaluator<'_, F>, before_events: usize) {
    for key in evaluator.event_log.drain(before_events..).rev() {
        if let Some(entries) = evaluator.events.get_mut(&key) {
            entries.pop();
            if entries.is_empty() {
                evaluator.events.remove(&key);
            }
        }
    }
}
