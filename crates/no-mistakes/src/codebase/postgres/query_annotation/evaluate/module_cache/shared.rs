use super::super::{Environment, Evaluator, Scope};
use crate::codebase::postgres::query_annotation::module_ids::ModuleId;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(in crate::codebase::postgres::query_annotation) mod remap;
use remap::remap_scope;

#[derive(Clone)]
pub(in crate::codebase::postgres::query_annotation) struct CachedModule {
    pub(in crate::codebase::postgres::query_annotation) deps: Vec<ModuleId>,
    pub(in crate::codebase::postgres::query_annotation) dep_refs: Vec<(ModuleId, usize)>,
    pub(in crate::codebase::postgres::query_annotation) dep_scopes: Vec<Scope>,
    pub(in crate::codebase::postgres::query_annotation) own_scopes: Vec<Scope>,
}

#[derive(Clone, Default)]
pub(in crate::codebase::postgres::query_annotation) struct SharedInit {
    pub(in crate::codebase::postgres::query_annotation) untrusted_array_intrinsics: bool,
    pub(in crate::codebase::postgres::query_annotation) modules:
        FxHashMap<ModuleId, Arc<CachedModule>>,
}

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation) fn try_install_shared_module(
        &mut self,
        path: &Path,
        module_id: ModuleId,
    ) -> Option<Environment> {
        let shared = self.modules.shared.as_ref()?;
        if shared.untrusted_array_intrinsics != self.untrusted_array_intrinsics {
            return None;
        }
        let cached = shared.modules.get(&module_id)?.clone();
        if !self.deps_match_cached(&cached) {
            return None;
        }
        Some(self.install_cached_module(path, module_id, &cached, None))
    }

    fn deps_match_cached(&self, cached: &CachedModule) -> bool {
        let mut remap = Vec::with_capacity(cached.dep_refs.len());
        for &(dep_mod, slot) in &cached.dep_refs {
            let Some(&dep_base) = self.modules.get(&dep_mod) else {
                remap.push(usize::MAX);
                continue;
            };
            remap.push(dep_base + slot);
        }
        for (index, &(dep_mod, slot)) in cached.dep_refs.iter().enumerate() {
            let Some(&dep_base) = self.modules.get(&dep_mod) else {
                continue;
            };
            let Some(current) = self.scopes.get(dep_base + slot) else {
                return false;
            };
            let expected = &cached.dep_scopes[index];
            if current.same_bindings(expected) {
                continue;
            }
            if remap.contains(&usize::MAX) || *current != remap_scope(expected, &remap) {
                return false;
            }
        }
        true
    }

    pub(in crate::codebase::postgres::query_annotation) fn install_cached_module(
        &mut self,
        path: &Path,
        module_id: ModuleId,
        cached: &CachedModule,
        mut dep_log: Option<&mut Vec<(ModuleId, usize)>>,
    ) -> Environment {
        if let Some(env) = self.modules.get(&module_id) {
            return *env;
        }
        for &dep_id in &cached.deps {
            if !self.modules.contains_key(&dep_id) {
                let dep_path = self.module_ids().path(dep_id).to_path_buf();
                let dep_cached = self
                    .modules
                    .shared
                    .as_ref()
                    .and_then(|shared| shared.modules.get(&dep_id))
                    .cloned()
                    .expect("cached module dependency must exist in SharedInit");
                self.install_cached_module(&dep_path, dep_id, &dep_cached, dep_log.as_deref_mut());
            }
        }
        let mut remap = Vec::with_capacity(cached.dep_refs.len() + cached.own_scopes.len());
        for &(dep_mod, slot) in &cached.dep_refs {
            let dep_base = self.modules[&dep_mod];
            remap.push(dep_base + slot);
        }
        let base = self.scopes.len();
        for offset in 0..cached.own_scopes.len() {
            remap.push(base + offset);
        }
        for (slot, scope) in cached.own_scopes.iter().enumerate() {
            self.scopes.push(remap_scope(scope, &remap));
            if let Some(log) = dep_log.as_deref_mut() {
                log.push((module_id, slot));
            }
        }
        self.modules.insert(module_id, base);
        self.record_alternative_module_initial(path, base);
        if dep_log.is_none() {
            if let Some(observer) = crate::diagnostics::current() {
                observer.increment("postgres.annotation.shared_init.cache_hits", 1);
            }
        }
        base
    }

    pub(in crate::codebase::postgres::query_annotation) fn is_pure_shared_module(
        &self,
        prefix_scopes: &[Scope],
        dep_module_count: usize,
        initial_untrusted_array_intrinsics: bool,
    ) -> bool {
        let prefix = prefix_scopes.len();
        self.event_log.is_empty()
            && self.untrusted_array_intrinsics == initial_untrusted_array_intrinsics
            && self.modules.len() == dep_module_count + 1
            && self.scopes.len() > prefix
            && self.scopes[prefix..]
                .iter()
                .all(|scope| !scope.contains_reference())
            && self.scopes[..prefix]
                .iter()
                .zip(prefix_scopes)
                .all(|(current, base)| current.same_bindings(base))
            && self.invalidated_builders.is_empty()
            && self.builder_updates.is_empty()
            && self.deleted_argument_slots.is_empty()
            && self.definite_deleted_argument_slots.is_empty()
            && self.disconnected_argument_slots.is_empty()
            && self.recreated_argument_slots.is_empty()
            && self.mapped_arguments.is_empty()
            && self.fresh_mapped_parameters.is_empty()
            && self.argument_objects.is_empty()
            && self.argument_extra_slots.is_empty()
            && self.captured_bindings.is_empty()
            && self.captured_binding_readers.is_empty()
            && self.mapped_argument_owners.is_empty()
            && self.mapped_parameter_indices.is_empty()
    }
}
