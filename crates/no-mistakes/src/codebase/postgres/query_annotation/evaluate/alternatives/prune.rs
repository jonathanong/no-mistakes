use super::super::Evaluator;
use super::{frames, reachable};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn prune_snapshot_state(
        &mut self,
        original: usize,
    ) {
        // Cached module identities remain stable across subsequent run projections.
        let original = self
            .modules
            .values()
            .map(|env| env + 1)
            .max()
            .unwrap_or(original)
            .max(original);
        frames::prune_unreachable(
            &mut self.scopes,
            original,
            &mut self.mapped_arguments,
            &mut self.argument_objects,
            &mut self.fresh_mapped_parameters,
            frames::MutationState {
                deleted: &mut self.deleted_argument_slots,
                definite: &mut self.definite_deleted_argument_slots,
                invalidated: &mut self.invalidated_builders,
            },
            &mut self.captured_bindings,
        );
        let live = reachable::collect(
            &self.scopes,
            self.scopes.len(),
            &[],
            &self.argument_objects,
            &self.mapped_arguments,
            &self.captured_bindings,
        );
        self.builder_updates
            .retain(|id, _| live.identities.contains(id));
    }
}
