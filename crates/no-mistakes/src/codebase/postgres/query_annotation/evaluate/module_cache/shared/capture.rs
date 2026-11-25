use super::super::super::{Environment, Evaluator, Scope};
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
    before_untrusted: bool,
    before_events: usize,
    before_invalidated: usize,
    before_builder_updates: usize,
    before_captured_prefix: usize,
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
            before_untrusted: self.untrusted_array_intrinsics,
            before_events: self.event_log.len(),
            before_invalidated: self.invalidated_builders.len(),
            before_builder_updates: self.builder_updates.len(),
            before_captured_prefix: self
                .captured_bindings
                .keys()
                .filter(|&&frame| frame <= env)
                .count(),
        })
    }

    pub(in crate::codebase::postgres::query_annotation) fn finish_shared_capture(
        &self,
        module_id: ModuleId,
        env: Environment,
        guard: SharedCaptureGuard,
    ) {
        let shared = &guard.shared;
        if !self.is_clean_shared_init(env, &guard) {
            shared
                .entries
                .entry(module_id)
                .or_insert(SharedEntry::Impure);
            return;
        }
        match self.build_cached_module(shared, module_id, env) {
            BuildOutcome::Ready(cached) => {
                shared
                    .entries
                    .entry(module_id)
                    .or_insert_with(|| SharedEntry::Pure(Arc::new(cached)));
            }
            BuildOutcome::Defer => {}
            BuildOutcome::Impure => {
                shared
                    .entries
                    .entry(module_id)
                    .or_insert(SharedEntry::Impure);
            }
        }
    }

    fn is_clean_shared_init(&self, env: Environment, guard: &SharedCaptureGuard) -> bool {
        let captured_prefix = self
            .captured_bindings
            .keys()
            .filter(|&&frame| frame <= env)
            .count();
        self.untrusted_array_intrinsics == guard.before_untrusted
            && self.event_log.len() == guard.before_events
            && self.invalidated_builders.len() == guard.before_invalidated
            && self.builder_updates.len() == guard.before_builder_updates
            && captured_prefix == guard.before_captured_prefix
            && !self.scopes[env].contains_reference()
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
            collect_dep_refs(shared, dep, &mut seen, &mut dep_refs, &mut raw_dep_scopes);
        }
        let mut eval_to_canon = FxHashMap::default();
        let mut canon_to_eval = Vec::with_capacity(dep_refs.len());
        for (index, &(dep_id, slot)) in dep_refs.iter().enumerate() {
            let eval_env = self.modules[&dep_id] + slot;
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
) {
    if !seen.insert(dep) {
        return;
    }
    let cached = {
        let entry = shared
            .entries
            .get(&dep)
            .expect("pure dependency must exist in SharedInit");
        let SharedEntry::Pure(cached) = entry.value() else {
            unreachable!("pure dependency must be Pure")
        };
        Arc::clone(cached)
    };
    for &sub in &cached.deps {
        collect_dep_refs(shared, sub, seen, dep_refs, raw_dep_scopes);
    }
    let mut sub_remap = Vec::with_capacity(cached.dep_refs.len() + cached.own_scopes.len());
    for &(sub_id, sub_slot) in &cached.dep_refs {
        let pos = dep_refs
            .iter()
            .position(|&(id, slot)| id == sub_id && slot == sub_slot)
            .expect("transitive dep must be collected first");
        sub_remap.push(pos);
    }
    let base = dep_refs.len();
    for offset in 0..cached.own_scopes.len() {
        sub_remap.push(base + offset);
    }
    for (slot, scope) in cached.own_scopes.iter().enumerate() {
        dep_refs.push((dep, slot));
        raw_dep_scopes.push(remap_scope(scope, &sub_remap));
    }
}
