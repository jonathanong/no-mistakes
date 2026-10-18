//! Legacy state-level unit tests may seed tables without evaluating saved sources.
use super::{transactions::Cell, Evaluator};
use std::path::{Path, PathBuf};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn initialize_reference_index_for_tests(
        &mut self,
    ) {
        assert!(self.transactions.frames.is_empty());
        assert!(self.active_module_initials.is_empty());
        self.transactions.reclamation = Default::default();
        let frames = self.scopes.ids().collect::<Vec<_>>();
        for env in frames {
            self.register_frame_references(env);
            self.register_mapping_references(env);
        }
        let arguments = self.argument_objects.keys().copied().collect::<Vec<_>>();
        for id in arguments {
            self.register_argument_references(id);
        }
        let mut masks = Vec::new();
        masks.extend(
            self.invalidated_builders
                .iter()
                .copied()
                .map(Cell::Invalidated),
        );
        masks.extend(
            self.deleted_argument_slots
                .iter()
                .map(|(id, index)| Cell::Deleted(*id, *index)),
        );
        masks.extend(
            self.definite_deleted_argument_slots
                .iter()
                .map(|(id, index)| Cell::Definite(*id, *index)),
        );
        masks.extend(
            self.disconnected_argument_slots
                .iter()
                .map(|(id, index)| Cell::Disconnected(*id, *index)),
        );
        masks.extend(
            self.recreated_argument_slots
                .iter()
                .map(|(id, index)| Cell::Recreated(*id, *index)),
        );
        for cell in masks {
            self.note_reference_cell(&cell);
        }
    }
}
