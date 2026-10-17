use super::*;

pub(super) struct GuardedSummary {
    assigned: Arc<ValueAssignedMutations>,
    dependencies: FxHashSet<Owner>,
}

impl ValueFlowResolution<'_, '_> {
    pub(in super::super::super) fn module_provider_summary(
        &self,
        file: &std::path::Path,
        guards: &ModuleAssignedMutations,
    ) -> Arc<ValueAssignedMutations> {
        let key = (
            file.to_path_buf(),
            guards as *const ModuleAssignedMutations as usize,
        );
        let cache = self.module_writer_phases.guarded.get_or_init(|| {
            dashmap::DashMap::with_hasher_and_shard_amount(rustc_hash::FxBuildHasher, 8)
        });
        let slot = Arc::clone(cache.entry(key).or_default().value());
        let prepared = slot.get_or_init(|| {
            let dependencies = std::cell::RefCell::new(fx_set());
            let assigned = guards.active_summary(|writer, target| {
                #[cfg(feature = "test-instrumentation")]
                self.cache.record_module_provider_check();
                let Some(writer) = writer else { return true };
                let phase = self.module_writer_proof(file, writer);
                dependencies
                    .borrow_mut()
                    .extend(phase.dependencies.iter().cloned());
                !phase.never
                    && (phase.repeated || phase.phase.is_none_or(|offset| target.contains(offset)))
            });
            GuardedSummary {
                assigned: Arc::new(assigned),
                dependencies: dependencies.into_inner(),
            }
        });
        self.writer_dependencies
            .borrow_mut()
            .extend(prepared.dependencies.iter().cloned());
        Arc::clone(&prepared.assigned)
    }
}
