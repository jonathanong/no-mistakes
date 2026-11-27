use super::super::super::{Environment, Evaluator, Scope};
use super::super::InitPuritySnapshot;
use super::{
    remap::{remap_scope, remap_scope_strict},
    CachedModule, SharedEntry, SharedInit,
};
use crate::codebase::postgres::query_annotation::module_ids::ModuleId;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(in crate::codebase::postgres::query_annotation) struct SharedCaptureGuard {
    shared: Arc<SharedInit>,
    before: InitPuritySnapshot,
}

impl SharedCaptureGuard {
    pub(in crate::codebase::postgres::query_annotation) fn before(&self) -> InitPuritySnapshot {
        self.before
    }
}

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation) fn begin_shared_capture(
        &self,
        module_id: ModuleId,
        env: Environment,
    ) -> Option<SharedCaptureGuard> {
        let shared = self.modules.shared.as_ref()?;
        if !shared.eligible.contains(&module_id) || shared.entries.contains_key(&module_id) {
            return None;
        }
        Some(SharedCaptureGuard {
            shared: Arc::clone(shared),
            before: self.init_purity_snapshot(env),
        })
    }

    pub(in crate::codebase::postgres::query_annotation) fn finish_shared_capture(
        &self,
        module_id: ModuleId,
        env: Environment,
        guard: SharedCaptureGuard,
    ) {
        let entries = &guard.shared.entries;
        let read_other_mode = guard.before().untrusted != guard.shared.untrusted_array_intrinsics
            && self.modules.untrusted_intrinsic_reads != guard.before().untrusted_reads;
        if read_other_mode || !self.is_clean_init(env, guard.before()) {
            entries.entry(module_id).or_insert(SharedEntry::Impure);
            return;
        }
        match self.build_cached_module(&guard.shared, module_id, env) {
            BuildOutcome::Ready(cached) => {
                entries
                    .entry(module_id)
                    .or_insert_with(|| SharedEntry::Pure(Arc::new(cached)));
            }
            BuildOutcome::Defer => {}
            BuildOutcome::Impure => {
                entries.entry(module_id).or_insert(SharedEntry::Impure);
            }
        }
    }

    fn build_cached_module(
        &self,
        shared: &SharedInit,
        module_id: ModuleId,
        env: Environment,
    ) -> BuildOutcome {
        let all_deps = shared
            .direct_deps
            .get(&module_id)
            .cloned()
            .unwrap_or_default();
        for &dep in &all_deps {
            match shared.entries.get(&dep) {
                Some(entry) if matches!(entry.value(), SharedEntry::Pure(_)) => {}
                Some(_) => return BuildOutcome::Impure,
                None => return BuildOutcome::Defer,
            }
        }
        let mut dep_refs = Vec::new();
        let mut seen = FxHashSet::default();
        let mut raw_dep_scopes = Vec::new();
        for &dep in all_deps.iter().filter(|d| self.modules.contains_key(d)) {
            if !collect_dep_refs(shared, dep, &mut seen, &mut dep_refs, &mut raw_dep_scopes) {
                return BuildOutcome::Defer;
            }
        }
        let mut eval_to_canon = FxHashMap::default();
        let mut canon_to_eval = Vec::with_capacity(dep_refs.len());
        for (index, &(dep_id, slot)) in dep_refs.iter().enumerate() {
            let Some(eval_env) = self
                .modules
                .get(&dep_id)
                .map(|&base| base + slot)
                .filter(|&env_idx| env_idx < self.scopes.len())
            else {
                return BuildOutcome::Defer;
            };
            eval_to_canon.insert(eval_env, index);
            canon_to_eval.push(eval_env);
        }
        let mut dep_scopes = Vec::with_capacity(dep_refs.len());
        for (index, raw_scope) in raw_dep_scopes.iter().enumerate() {
            let current = &self.scopes[canon_to_eval[index]];
            let expected = remap_scope(raw_scope, &canon_to_eval);
            if !current.same_bindings(&expected) && *current != expected {
                return BuildOutcome::Impure;
            }
            dep_scopes.push(raw_scope.clone());
        }
        eval_to_canon.insert(env, dep_refs.len());
        let Some(own_scope) = remap_scope_strict(&self.scopes[env], &eval_to_canon) else {
            return BuildOutcome::Impure;
        };
        BuildOutcome::Ready(CachedModule {
            deps: all_deps,
            dep_refs,
            dep_scopes,
            own_scopes: vec![own_scope],
        })
    }
}

enum BuildOutcome {
    Ready(CachedModule),
    Defer,
    Impure,
}

fn collect_dep_refs(
    shared: &SharedInit,
    dep: ModuleId,
    seen: &mut FxHashSet<ModuleId>,
    dep_refs: &mut Vec<(ModuleId, usize)>,
    raw_dep_scopes: &mut Vec<Scope>,
) -> bool {
    if !seen.insert(dep) {
        return true;
    }
    let cached = {
        let Some(entry) = shared.entries.get(&dep) else {
            return false;
        };
        let SharedEntry::Pure(cached) = entry.value() else {
            return false;
        };
        Arc::clone(cached)
    };
    for &(sub, _) in &cached.dep_refs {
        if !collect_dep_refs(shared, sub, seen, dep_refs, raw_dep_scopes) {
            return false;
        }
    }
    let mut sub_remap = Vec::with_capacity(cached.dep_refs.len() + cached.own_scopes.len());
    for &(sub_id, sub_slot) in &cached.dep_refs {
        let Some(pos) = dep_refs.iter().position(|&item| item == (sub_id, sub_slot)) else {
            return false;
        };
        sub_remap.push(pos);
    }
    let base = dep_refs.len();
    sub_remap.extend(base..base + cached.own_scopes.len());
    for (slot, scope) in cached.own_scopes.iter().enumerate() {
        dep_refs.push((dep, slot));
        raw_dep_scopes.push(remap_scope(scope, &sub_remap));
    }
    true
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "capture_tests.rs"]
mod tests;
