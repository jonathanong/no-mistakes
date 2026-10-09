use super::super::Evaluator;
use super::{frames, reachable};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn prune_snapshot_state(
        &mut self,
    ) {
        frames::compact_modules(
            &mut self.scopes,
            &mut self.modules,
            &mut self.mapped_arguments,
            &mut self.argument_objects,
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
                invalidated: &mut self.invalidated_builders,
            },
            &self.mapped_arguments,
            &mut self.argument_objects,
            &self.captured_bindings,
        );
        let live = reachable::collect_from_roots(
            &self.scopes,
            &self.modules.values().copied().collect::<Vec<_>>(),
            &[],
            &self.argument_objects,
            &self.mapped_arguments,
            &self.captured_bindings,
        );
        self.builder_updates
            .retain(|id, _| live.identities.contains(id));
    }
}
