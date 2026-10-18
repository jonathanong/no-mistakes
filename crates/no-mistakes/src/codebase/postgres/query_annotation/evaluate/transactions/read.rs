use super::{Cell, Saved};
use crate::codebase::postgres::query_annotation::evaluate::Evaluator;
use std::path::{Path, PathBuf};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn read_cell(
        &self,
        cell: &Cell,
    ) -> Saved {
        match cell {
            Cell::Binding(env, name) => Saved::Value(self.scopes[*env].get(name).cloned()),
            Cell::Dense(id, index) => Saved::Value(
                self.argument_objects
                    .get(id)
                    .and_then(|v| v.get(*index))
                    .cloned(),
            ),
            Cell::Length(id) => Saved::Length(self.argument_objects[id].len()),
            Cell::Extra(id, index) => Saved::Value(
                self.argument_extra_slots
                    .get(id)
                    .and_then(|v| v.get(index))
                    .cloned(),
            ),
            Cell::Invalidated(id) => Saved::Flag(self.invalidated_builders.contains(id)),
            Cell::Builder(id) => Saved::Value(self.builder_updates.get(id).cloned()),
            Cell::Deleted(id, index) => {
                Saved::Flag(self.deleted_argument_slots.contains(&(*id, *index)))
            }
            Cell::Definite(id, index) => Saved::Flag(
                self.definite_deleted_argument_slots
                    .contains(&(*id, *index)),
            ),
            Cell::Disconnected(id, index) => {
                Saved::Flag(self.disconnected_argument_slots.contains(&(*id, *index)))
            }
            Cell::Recreated(id, index) => {
                Saved::Flag(self.recreated_argument_slots.contains(&(*id, *index)))
            }
            Cell::Fresh(env, name) => Saved::Flag(
                self.fresh_mapped_parameters
                    .get(env)
                    .is_some_and(|names| names.contains(name)),
            ),
            Cell::Trust => Saved::Flag(self.untrusted_array_intrinsics),
            Cell::CallbackSeen(key) => Saved::Flag(
                self.active_callback_functions
                    .as_ref()
                    .is_some_and(|seen| seen.contains(key)),
            ),
            Cell::CallbackRunning(key) => Saved::Flag(
                self.active_callback_executions
                    .as_ref()
                    .is_some_and(|seen| seen.contains(key)),
            ),
            Cell::ExecutionsEnabled => Saved::Flag(self.active_callback_executions.is_some()),
            Cell::CallbackEnabled => Saved::Flag(self.active_callback_functions.is_some()),
            Cell::Module(path) => Saved::Environment(self.modules.get(path).copied()),
        }
    }
}
