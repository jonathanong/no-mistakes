use super::super::{Environment, Evaluator, Scope};
use crate::codebase::postgres::query_annotation::module_ids::ModuleId;
use crate::fx::{FxHashMap, FxHashSet};
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod capture;
pub(in crate::codebase::postgres::query_annotation) mod remap;
use remap::remap_scope;

#[derive(Clone)]
pub(in crate::codebase::postgres::query_annotation) struct CachedModule {
    pub(in crate::codebase::postgres::query_annotation) deps: Vec<ModuleId>,
    pub(in crate::codebase::postgres::query_annotation) dep_refs: Vec<(ModuleId, usize)>,
    pub(in crate::codebase::postgres::query_annotation) dep_scopes: Vec<Scope>,
    pub(in crate::codebase::postgres::query_annotation) own_scopes: Vec<Scope>,
}

pub(in crate::codebase::postgres::query_annotation) enum SharedEntry {
    Pure(Arc<CachedModule>),
    Impure,
}

#[derive(Default)]
pub(in crate::codebase::postgres::query_annotation) struct SharedInit {
    pub(in crate::codebase::postgres::query_annotation) untrusted_array_intrinsics: bool,
    pub(in crate::codebase::postgres::query_annotation) eligible: FxHashSet<ModuleId>,
    pub(in crate::codebase::postgres::query_annotation) direct_deps:
        FxHashMap<ModuleId, Vec<ModuleId>>,
    pub(in crate::codebase::postgres::query_annotation) entries: DashMap<ModuleId, SharedEntry>,
}

impl SharedInit {
    pub(in crate::codebase::postgres::query_annotation) fn cached_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| matches!(entry.value(), SharedEntry::Pure(_)))
            .count()
    }
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
        let cached = {
            let entry = shared.entries.get(&module_id)?;
            let SharedEntry::Pure(cached) = entry.value() else {
                return None;
            };
            Arc::clone(cached)
        };
        if !self.deps_match_cached(&cached) {
            return None;
        }
        Some(self.install_cached_module(path, module_id, &cached))
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
            let current = &self.scopes[dep_base + slot];
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

    fn install_cached_module(
        &mut self,
        path: &Path,
        module_id: ModuleId,
        cached: &CachedModule,
    ) -> Environment {
        for &dep_id in &cached.deps {
            if !self.modules.contains_key(&dep_id) {
                let dep_path = self.module_ids().path(dep_id).to_path_buf();
                let dep_cached = {
                    let shared = self.modules.shared.as_ref().expect("SharedInit active");
                    let entry = shared
                        .entries
                        .get(&dep_id)
                        .expect("cached module dependency must exist in SharedInit");
                    let SharedEntry::Pure(cached) = entry.value() else {
                        unreachable!("cached module dependency must be Pure")
                    };
                    Arc::clone(cached)
                };
                self.install_cached_module(&dep_path, dep_id, &dep_cached);
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
        for scope in &cached.own_scopes {
            self.scopes.push(remap_scope(scope, &remap));
        }
        self.modules.insert(module_id, base);
        self.record_alternative_module_initial(path, base);
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("postgres.annotation.shared_init.cache_hits", 1);
        }
        base
    }
}
