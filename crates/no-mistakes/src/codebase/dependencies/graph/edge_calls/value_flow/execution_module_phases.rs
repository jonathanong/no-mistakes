use super::*;
use crate::codebase::dependencies::extract::{ModuleAssignedMutations, ValueAssignedMutations};
use std::sync::{Arc, OnceLock};
#[path = "execution_module_providers.rs"]
mod providers;
type Owner = (std::path::PathBuf, CallableId);
type SummarySlot<T> = Arc<OnceLock<T>>;
type SummaryMap<K, V> = dashmap::DashMap<K, SummarySlot<V>, rustc_hash::FxBuildHasher>;
struct ModuleWriterPhase {
    phase: Option<u32>,
    dependencies: FxHashSet<Owner>,
    repeated: bool,
    never: bool,
}

#[derive(Default)]
pub(in super::super) struct ModuleWriterPhases {
    phases: OnceLock<SummaryMap<Owner, Arc<ModuleWriterPhase>>>,
    guarded: OnceLock<SummaryMap<(std::path::PathBuf, usize), providers::GuardedSummary>>,
}

impl ValueFlowResolution<'_, '_> {
    /// A closed synchronous caller chain can prove that every execution starts
    /// after an independently acquired module snapshot, even when it repeats.
    /// Floyd's walk rejects cycles without retaining an extra caller graph.
    pub(in super::super) fn writer_after_module_snapshot(
        &self,
        path: &std::path::Path,
        writer: CallableId,
        acquisition: u32,
    ) -> bool {
        let proof = self.module_writer_proof(path, writer);
        self.writer_dependencies
            .borrow_mut()
            .extend(proof.dependencies.iter().cloned());
        proof.phase.is_some_and(|offset| offset > acquisition)
    }

    fn module_writer_proof(
        &self,
        path: &std::path::Path,
        writer: CallableId,
    ) -> std::sync::Arc<ModuleWriterPhase> {
        let key = (path.to_path_buf(), writer);
        let phases = self.module_writer_phases.phases.get_or_init(|| {
            dashmap::DashMap::with_hasher_and_shard_amount(rustc_hash::FxBuildHasher, 8)
        });
        let slot = std::sync::Arc::clone(phases.entry(key.clone()).or_default().value());
        std::sync::Arc::clone(slot.get_or_init(|| {
            let mut dependencies = fx_set();
            let (mut repeated, mut never) = (false, false);
            let phase = self.module_writer_phase(key, &mut dependencies, &mut repeated, &mut never);
            std::sync::Arc::new(ModuleWriterPhase {
                phase,
                dependencies,
                repeated,
                never,
            })
        }))
    }

    pub(in super::super) fn merge_writer_dependencies(&mut self) {
        self.used_parameters
            .extend(self.writer_dependencies.get_mut().drain());
    }

    fn module_writer_phase(
        &self,
        mut key: Owner,
        dependencies: &mut FxHashSet<Owner>,
        repeated: &mut bool,
        never: &mut bool,
    ) -> Option<u32> {
        let (mut slow, mut fast) = (key.1, key.1);
        loop {
            key.1 = slow;
            slow = match self.module_writer_step(&key, dependencies, repeated, never) {
                Some(Step::Owner(owner)) => owner,
                Some(Step::Phase(offset, _)) => return Some(offset),
                None => return None,
            };
            for _ in 0..2 {
                key.1 = fast;
                fast = match self.module_writer_step(&key, dependencies, repeated, never) {
                    Some(Step::Owner(owner)) => owner,
                    Some(Step::Phase(offset, _)) => return Some(offset),
                    None => return None,
                };
            }
            if slow == fast {
                return None;
            }
        }
    }

    fn module_writer_step(
        &self,
        key: &Owner,
        dependencies: &mut FxHashSet<Owner>,
        repeated: &mut bool,
        never: &mut bool,
    ) -> Option<Step> {
        dependencies.insert(key.clone());
        #[cfg(feature = "test-instrumentation")]
        self.cache.record_execution_step();
        let index = self.indexes.file(self.facts, &key.0)?;
        let scope = index.values.scopes.get(&key.1)?;
        // Named module bindings without value uses have closed invocation facts.
        // Nested/object callables and stored/returned values can escape opaquely.
        if index
            .callable_bindings
            .get(&0)
            .and_then(|names| names.get(scope))
            != Some(&key.1)
            || index
                .values
                .callable_value_uses
                .as_ref()
                .and_then(|uses| uses.get(&0))
                .is_some_and(|uses| uses.contains(scope))
            || index.exported_scopes.contains(scope)
            || !self.synchronous_callable(&key.0, key.1)
        {
            return None;
        }
        let incoming = self.incoming.get(key).map_or(&[][..], Vec::as_slice);
        if incoming.is_empty() {
            *never = true;
            return None;
        }
        let [input] = incoming else {
            return None;
        };
        if input.callback || input.file != key.0 {
            return None;
        }
        let call = index
            .values
            .sites
            .get(&input.offset)
            .and_then(|sites| sites.get(&input.callee))
            .and_then(|key| index.values.calls.get(key))?;
        if call.bound || call.spread_at.is_some() {
            return None;
        }
        *repeated |= call.repeated;
        Some(match call.context {
            Some(owner) => Step::Owner(owner),
            None => Step::Phase(call.end.saturating_sub(1), None),
        })
    }
}
