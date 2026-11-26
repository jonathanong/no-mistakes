use super::super::{
    module_cache::{shared::remap::remap_captured_scope, ModuleCache},
    ArgumentObjects, CapturedBindings, Environment, Evaluator, File, Scope, Value,
};
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::postgres::query_annotation::{
    module_ids::ModuleId, project::shared_init::executor_scan::is_statically_pure_top_level,
};
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

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

    pub(super) fn restore<F: Fn(&str, &Path) -> Option<PathBuf>>(
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

    pub(super) fn promote_reused<F: Fn(&str, &Path) -> Option<PathBuf>>(
        &mut self,
        evaluator: &mut Evaluator<'_, F>,
        unpromotable: &mut FxHashSet<ModuleId>,
        remaining_tasks: usize,
    ) {
        if evaluator.modules.sources.is_tracking() || evaluator.modules.len() <= self.modules.len()
        {
            return;
        }
        let mut captured: Vec<_> = evaluator.modules.captured_inits.drain().collect();
        captured.sort_unstable_by_key(|(id, _)| *id);
        for (id, entry) in captured {
            self.modules.capture_on_init.remove(&id);
            if !self.modules.contains_key(&id) && !unpromotable.contains(&id) {
                let installed = entry
                    .is_some_and(|(old_env, scope)| self.try_install_captured(id, old_env, &scope));
                if !installed {
                    unpromotable.insert(id);
                }
            }
        }
        if remaining_tasks < 2 {
            return;
        }
        let mut newly_loaded: Vec<_> = evaluator
            .modules
            .keys()
            .copied()
            .filter(|id| {
                !self.modules.contains_key(id)
                    && !self.modules.capture_on_init.contains_key(id)
                    && !unpromotable.contains(id)
            })
            .collect();
        newly_loaded.sort_unstable();
        for id in newly_loaded {
            let target = evaluator.module_ids().path(id);
            let Some(file) = evaluator
                .files
                .get(target)
                .filter(|f| is_statically_pure_top_level(f, target, &evaluator.resolve))
            else {
                unpromotable.insert(id);
                continue;
            };
            if let Some(deps) = self.static_dep_snapshots(evaluator, file, target) {
                self.modules.capture_on_init.insert(id, deps);
            }
        }
    }

    fn try_install_captured(&mut self, id: ModuleId, old_env: Environment, scope: &Scope) -> bool {
        if scope.contains_reference() {
            return false;
        }
        let prefix = self.scopes.len();
        let Some(remapped) = remap_captured_scope(scope, old_env, prefix, prefix) else {
            return false;
        };
        self.scopes.push(remapped);
        self.modules.insert(id, prefix);
        true
    }

    fn static_dep_snapshots<F: Fn(&str, &Path) -> Option<PathBuf>>(
        &self,
        evaluator: &Evaluator<'_, F>,
        file: &File<'_>,
        path: &Path,
    ) -> Option<Vec<(Environment, Scope)>> {
        let ids = evaluator.module_ids();
        let mut deps = Vec::new();
        for import in &file.module.ts.imports {
            if import.kind != ImportKind::Static || import.function_scope.is_some() {
                continue;
            }
            let Some(target) = file
                .module
                .target(&import.specifier, path, &evaluator.resolve)
            else {
                continue;
            };
            if !evaluator.files.contains_key(target) {
                continue;
            }
            let &dep_env = self.modules.get(&ids.id(target))?;
            deps.push((dep_env, self.scopes[dep_env].clone()));
        }
        Some(deps)
    }
}
