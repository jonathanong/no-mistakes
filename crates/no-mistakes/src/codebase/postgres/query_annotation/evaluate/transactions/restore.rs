mod values;
use super::{Cell, Saved};
use crate::codebase::postgres::query_annotation::evaluate::Evaluator;
use std::path::{Path, PathBuf};
fn membership<T: Eq + std::hash::Hash>(set: &mut crate::fx::FxHashSet<T>, key: T, present: bool) {
    if present {
        set.insert(key);
    } else {
        set.remove(&key);
    }
}
fn callback_membership<T: Clone + Eq + std::hash::Hash>(
    set: &mut Option<crate::fx::FxHashSet<T>>,
    key: &T,
    present: bool,
) {
    if present {
        set.get_or_insert_with(Default::default).insert(key.clone());
    } else if let Some(seen) = set {
        seen.remove(key);
    }
}
fn callback_enabled<T>(set: &mut Option<T>, present: bool)
where
    T: Default,
{
    if present {
        set.get_or_insert_with(Default::default);
    } else {
        *set = None;
    }
}
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn restore_cell(
        &mut self,
        cell: &Cell,
        old: &Saved,
    ) {
        match (cell, old) {
            (Cell::Binding(env, name), Saved::Value(value)) => {
                self.restore_saved_binding(env, name, value);
            }
            (Cell::Dense(id, index), Saved::Value(value)) => {
                self.restore_saved_dense(id, index, value);
            }
            (Cell::Length(id), Saved::Length(length)) => {
                self.restore_saved_length(id, length);
            }
            (Cell::Extra(id, index), Saved::Value(value)) => {
                self.restore_saved_extra(id, index, value);
            }
            (Cell::Builder(id), Saved::Value(value)) => {
                self.restore_saved_builder(id, value);
            }
            (Cell::Invalidated(id), Saved::Flag(present)) => {
                membership(&mut self.invalidated_builders, *id, *present)
            }
            (Cell::Deleted(id, index), Saved::Flag(present)) => {
                membership(&mut self.deleted_argument_slots, (*id, *index), *present)
            }
            (Cell::Definite(id, index), Saved::Flag(present)) => membership(
                &mut self.definite_deleted_argument_slots,
                (*id, *index),
                *present,
            ),
            (Cell::Disconnected(id, index), Saved::Flag(present)) => membership(
                &mut self.disconnected_argument_slots,
                (*id, *index),
                *present,
            ),
            (Cell::Recreated(id, index), Saved::Flag(present)) => {
                membership(&mut self.recreated_argument_slots, (*id, *index), *present)
            }
            (Cell::Fresh(env, name), Saved::Flag(present)) => {
                self.restore_saved_fresh(env, name, present);
            }
            (Cell::CallbackSeen(key), Saved::Flag(present)) => {
                callback_membership(&mut self.active_callback_functions, key, *present);
            }
            (Cell::CallbackRunning(key), Saved::Flag(present)) => {
                callback_membership(&mut self.active_callback_executions, key, *present);
            }
            (Cell::CallbackEnabled, Saved::Flag(present)) => {
                callback_enabled(&mut self.active_callback_functions, *present);
            }
            (Cell::ExecutionsEnabled, Saved::Flag(present)) => {
                callback_enabled(&mut self.active_callback_executions, *present);
            }
            (Cell::Module(path), Saved::Environment(value)) => {
                if let Some(env) = value {
                    self.modules.insert(path.clone(), *env);
                } else {
                    self.modules.remove(path);
                }
            }
            (Cell::Trust, Saved::Flag(value)) => self.untrusted_array_intrinsics = *value,
            _ => unreachable!("transaction location and payload agree"),
        }
        self.note_reference_cell(cell);
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn apply_cell(
        &mut self,
        cell: &Cell,
        value: &Saved,
    ) {
        if self.read_cell(cell) != *value {
            if let Cell::Dense(id, 0) = cell {
                self.save_cell(Cell::Length(*id));
            }
            self.save_cell(cell.clone());
            if let (Cell::Length(id), Saved::Length(length)) = (cell, value) {
                for index in *length..self.argument_objects[id].len() {
                    self.save_cell(Cell::Dense(*id, index));
                }
            }
            self.restore_cell(cell, value);
        }
        // A caller may invalidate an already-expired proof. Remove any stale
        // derived reader without recreating the absent primary proof.
        self.restore_transaction_index(cell);
    }
}
