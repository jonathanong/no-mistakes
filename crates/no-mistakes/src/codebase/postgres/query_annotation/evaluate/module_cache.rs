use super::{Environment, Evaluator, Scope};
use crate::codebase::postgres::query_annotation::module_ids::{ModuleId, ModuleIds};
use crate::fx::FxHashMap;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

/// Snapshots copy compact keys and share the request's immutable identity table.
#[derive(Clone, Default)]
pub(in crate::codebase::postgres::query_annotation) struct ModuleCache {
    ids: OnceLock<Arc<ModuleIds>>,
    values: FxHashMap<ModuleId, Environment>,
    pub(in crate::codebase::postgres::query_annotation) shared: Option<Arc<shared::SharedInit>>,
    pub(in crate::codebase::postgres::query_annotation) sources:
        source_dependencies::SourceDependencies,
    pub(in crate::codebase::postgres::query_annotation) capture_on_init:
        FxHashMap<ModuleId, Vec<(Environment, Scope)>>,
    pub(in crate::codebase::postgres::query_annotation) captured_inits:
        FxHashMap<ModuleId, Option<(Environment, Scope)>>,
    pub(in crate::codebase::postgres::query_annotation) untrusted_intrinsic_writes: u64,
    pub(in crate::codebase::postgres::query_annotation) untrusted_intrinsic_reads: u64,
}

pub(in crate::codebase::postgres::query_annotation) mod shared;
pub(in crate::codebase::postgres::query_annotation) mod source_dependencies;

impl PartialEq for ModuleCache {
    fn eq(&self, other: &Self) -> bool {
        self.ids == other.ids && self.values == other.values
    }
}
impl Eq for ModuleCache {}

impl ModuleCache {
    pub fn with_ids(ids: Arc<ModuleIds>) -> Self {
        Self {
            ids: OnceLock::from(ids),
            values: FxHashMap::default(),
            shared: None,
            sources: Default::default(),
            capture_on_init: FxHashMap::default(),
            captured_inits: FxHashMap::default(),
            untrusted_intrinsic_writes: 0,
            untrusted_intrinsic_reads: 0,
        }
    }
}

impl Deref for ModuleCache {
    type Target = FxHashMap<ModuleId, Environment>;
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}
impl DerefMut for ModuleCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}

#[derive(Clone, Copy)]
pub(in crate::codebase::postgres::query_annotation) struct InitPuritySnapshot {
    pub(in crate::codebase::postgres::query_annotation) untrusted: bool,
    pub(in crate::codebase::postgres::query_annotation) untrusted_reads: u64,
    untrusted_writes: u64,
    events: usize,
    invalidated: usize,
    builder_updates: usize,
    captured_prefix: usize,
}

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn module_ids(&self) -> &ModuleIds {
        self.shared_module_ids()
    }

    pub(in crate::codebase::postgres::query_annotation) fn init_purity_snapshot(
        &self,
        env: Environment,
    ) -> InitPuritySnapshot {
        InitPuritySnapshot {
            untrusted: self.untrusted_array_intrinsics,
            untrusted_reads: self.modules.untrusted_intrinsic_reads,
            untrusted_writes: self.modules.untrusted_intrinsic_writes,
            events: self.event_log.len(),
            invalidated: self.invalidated_builders.len(),
            builder_updates: self.builder_updates.len(),
            captured_prefix: self.captured_prefix_count(env),
        }
    }

    pub(in crate::codebase::postgres::query_annotation) fn is_clean_init(
        &self,
        env: Environment,
        before: InitPuritySnapshot,
    ) -> bool {
        self.untrusted_array_intrinsics == before.untrusted
            && self.modules.untrusted_intrinsic_writes == before.untrusted_writes
            && self.event_log.len() == before.events
            && self.invalidated_builders.len() == before.invalidated
            && self.builder_updates.len() == before.builder_updates
            && self.captured_prefix_count(env) == before.captured_prefix
            && !self.scopes[env].contains_reference()
    }

    fn captured_prefix_count(&self, env: Environment) -> usize {
        self.captured_bindings
            .keys()
            .filter(|&&frame| frame <= env)
            .count()
    }

    pub(in crate::codebase::postgres::query_annotation) fn shared_module_ids(
        &self,
    ) -> &Arc<ModuleIds> {
        self.modules.ids.get_or_init(|| {
            // Production evaluators already have the prepared shared table.
            // Direct unit evaluators freeze their supplied catalog lazily.
            // Every prepared module carries the same Arc, so the chosen table
            // is independent of hash-map iteration order.
            self.files
                .values()
                .find_map(|file| file.module.module_ids.as_ref())
                .map(Arc::clone)
                .unwrap_or_else(|| Arc::new(ModuleIds::new(self.files.keys().cloned())))
        })
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
