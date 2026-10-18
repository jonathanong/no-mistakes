use super::super::Evaluator;
use super::{arena, frames};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn prune_snapshot_state(
        &mut self,
    ) {
        let previous_frames = self.scopes.ids().collect::<Vec<_>>();
        let previous_arguments = self.argument_objects.keys().copied().collect::<Vec<_>>();
        let indices = frames::compact_modules(
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
        for env in previous_frames {
            if self.scopes.get(env).is_none() {
                self.drop_frame_references(env);
                self.drop_frame_callback_keys(env);
            }
        }
        for id in previous_arguments {
            if !self.argument_objects.contains_key(&id) {
                self.drop_argument_state(id);
            }
        }
        self.remap_active_callbacks(0, &indices.indices);
        self.remap_compacted_binding_indexes(0, &indices.indices, indices.captured_changed);
        // Callback remapping and derived indexes do not change live argument
        // or builder IDs. Reuse the module-root walk, including captured cycles.
        frames::prune_mutations(
            frames::MutationState {
                deleted: &mut self.deleted_argument_slots,
                definite: &mut self.definite_deleted_argument_slots,
                disconnected: &mut self.disconnected_argument_slots,
                invalidated: &mut self.invalidated_builders,
                updates: &mut self.builder_updates,
            },
            &indices.arguments,
            &indices.identities,
        );
        self.reclaim_dead_state();
    }
}

#[cfg(test)]
mod tests;
