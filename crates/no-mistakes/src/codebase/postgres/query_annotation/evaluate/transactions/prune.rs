use super::{Cell, Checkpoint};
use crate::codebase::postgres::query_annotation::evaluate::Evaluator;
use crate::fx::FxHashSet;
use std::path::{Path, PathBuf};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn prune_transaction_new_identities(
        &mut self,
        cp: Checkpoint,
        arguments: &FxHashSet<u64>,
        identities: &FxHashSet<u64>,
    ) {
        let frame = &self.transactions.frames[cp.0];
        for cell in &frame.writes {
            match cell {
                Cell::Invalidated(id) if *id >= frame.identities && !identities.contains(id) => {
                    self.invalidated_builders.remove(id);
                }
                Cell::Builder(id) if *id >= frame.identities && !identities.contains(id) => {
                    self.builder_updates.remove(id);
                }
                Cell::Deleted(id, index) if *id >= frame.identities && !arguments.contains(id) => {
                    self.deleted_argument_slots.remove(&(*id, *index));
                }
                Cell::Definite(id, index) if *id >= frame.identities && !arguments.contains(id) => {
                    self.definite_deleted_argument_slots.remove(&(*id, *index));
                }
                Cell::Disconnected(id, index)
                    if *id >= frame.identities && !arguments.contains(id) =>
                {
                    self.disconnected_argument_slots.remove(&(*id, *index));
                }
                Cell::Recreated(id, index)
                    if *id >= frame.identities && !arguments.contains(id) =>
                {
                    self.recreated_argument_slots.remove(&(*id, *index));
                }
                _ => {}
            }
        }
    }
}
