use super::super::Evaluator;
use super::{arena, frames, reachable};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn prune_snapshot_state(
        &mut self,
    ) {
        frames::compact_modules(
            &mut self.scopes,
            &mut self.modules,
            &mut self.mapped_arguments,
            arena::ArenaMut {
                objects: &mut self.argument_objects,
                extras: &mut self.argument_extra_slots,
            },
            &mut self.fresh_mapped_parameters,
            &mut self.captured_bindings,
        );
        self.rebuild_captured_readers();
        self.rebuild_mapped_argument_owners();
        frames::prune_state(
            &self.scopes,
            &[],
            frames::MutationState {
                deleted: &mut self.deleted_argument_slots,
                definite: &mut self.definite_deleted_argument_slots,
                disconnected: &mut self.disconnected_argument_slots,
                invalidated: &mut self.invalidated_builders,
            },
            &self.mapped_arguments,
            arena::ArenaMut {
                objects: &mut self.argument_objects,
                extras: &mut self.argument_extra_slots,
            },
            &self.captured_bindings,
        );
        let live = reachable::collect_from_roots(
            &self.scopes,
            &self.modules.values().copied().collect::<Vec<_>>(),
            &[],
            arena::Arena {
                objects: &self.argument_objects,
                extras: &self.argument_extra_slots,
            },
            &self.mapped_arguments,
            &self.captured_bindings,
        );
        self.builder_updates
            .retain(|id, _| live.identities.contains(id));
    }
    pub(super) fn prune_alternative_state(&mut self, returned: &[super::super::Value]) {
        frames::prune_state(
            &self.scopes,
            returned,
            frames::MutationState {
                deleted: &mut self.deleted_argument_slots,
                definite: &mut self.definite_deleted_argument_slots,
                disconnected: &mut self.disconnected_argument_slots,
                invalidated: &mut self.invalidated_builders,
            },
            &self.mapped_arguments,
            arena::ArenaMut {
                objects: &mut self.argument_objects,
                extras: &mut self.argument_extra_slots,
            },
            &self.captured_bindings,
        );
        let live = reachable::collect(
            &self.scopes,
            self.scopes.len(),
            returned,
            arena::Arena {
                objects: &self.argument_objects,
                extras: &self.argument_extra_slots,
            },
            &self.mapped_arguments,
            &self.captured_bindings,
        );
        self.builder_updates
            .retain(|id, _| live.identities.contains(id));
    }
    pub(super) fn install_disconnected_slots(
        &mut self,
        original: &crate::fx::FxHashSet<(u64, usize)>,
        common: Option<crate::fx::FxHashSet<(u64, usize)>>,
        private: crate::fx::FxHashSet<(u64, usize)>,
    ) {
        let mut final_disconnected = common.unwrap_or(original.clone());
        final_disconnected.extend(private);
        self.disconnected_argument_slots = original.clone();
        for (id, index) in final_disconnected.difference(original) {
            self.disconnect_mapped_slot(*id, *index);
        }
        self.disconnected_argument_slots = final_disconnected;
    }
}
