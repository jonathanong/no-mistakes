use super::{Cell, Saved};
use crate::codebase::postgres::query_annotation::evaluate::{Evaluator, Value};
use std::path::{Path, PathBuf};
fn membership<T: Eq + std::hash::Hash>(set: &mut crate::fx::FxHashSet<T>, key: T, present: bool) {
    if present {
        set.insert(key);
    } else {
        set.remove(&key);
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
                if let Some(value) = value {
                    self.scopes[*env].restore_binding(name.clone(), Some(value.clone()));
                } else {
                    self.scopes[*env].restore_binding(name.clone(), None);
                }
            }
            (Cell::Dense(id, index), Saved::Value(value)) => {
                if let Some(value) = value {
                    let slots = self.argument_objects.get_mut(id).expect("incoming object");
                    slots.resize(slots.len().max(index + 1), Value::Unknown);
                    slots[*index] = value.clone();
                }
            }
            (Cell::Length(id), Saved::Length(length)) => {
                let old_length = self.argument_objects[id].len();
                self.argument_objects
                    .get_mut(id)
                    .expect("incoming object")
                    .resize(*length, Value::Unknown);
                for index in *length..old_length {
                    self.note_reference_cell(&Cell::Dense(*id, index));
                }
            }
            (Cell::Extra(id, index), Saved::Value(value)) => {
                if let Some(value) = value {
                    self.argument_extra_slots
                        .entry(*id)
                        .or_default()
                        .insert(*index, value.clone());
                } else if let Some(slots) = self.argument_extra_slots.get_mut(id) {
                    slots.remove(index);
                    if slots.is_empty() {
                        self.argument_extra_slots.remove(id);
                    }
                }
            }
            (Cell::Builder(id), Saved::Value(value)) => {
                if let Some(value) = value {
                    self.builder_updates.insert(*id, value.clone());
                } else {
                    self.builder_updates.remove(id);
                }
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
                if *present {
                    self.fresh_mapped_parameters
                        .entry(*env)
                        .or_default()
                        .insert(name.clone());
                } else if let Some(names) = self.fresh_mapped_parameters.get_mut(env) {
                    names.remove(name);
                    if names.is_empty() {
                        self.fresh_mapped_parameters.remove(env);
                    }
                }
            }
            (Cell::CallbackSeen(key), Saved::Flag(present)) => {
                if *present {
                    self.active_callback_functions
                        .get_or_insert_with(Default::default)
                        .insert(key.clone());
                } else if let Some(seen) = &mut self.active_callback_functions {
                    seen.remove(key);
                }
            }
            (Cell::CallbackRunning(key), Saved::Flag(present)) => {
                if *present {
                    self.active_callback_executions
                        .get_or_insert_with(Default::default)
                        .insert(key.clone());
                } else if let Some(seen) = &mut self.active_callback_executions {
                    seen.remove(key);
                }
            }
            (Cell::CallbackEnabled, Saved::Flag(present)) => {
                if *present {
                    self.active_callback_functions
                        .get_or_insert_with(Default::default);
                } else {
                    self.active_callback_functions = None;
                }
            }
            (Cell::ExecutionsEnabled, Saved::Flag(present)) => {
                if *present {
                    self.active_callback_executions
                        .get_or_insert_with(Default::default);
                } else {
                    self.active_callback_executions = None;
                }
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
