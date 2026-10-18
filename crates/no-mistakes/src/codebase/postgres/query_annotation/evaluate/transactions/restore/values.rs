use super::super::Cell;
use crate::codebase::postgres::query_annotation::evaluate::{Environment, Evaluator, Value};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn restore_saved_binding(
        &mut self,
        env: &Environment,
        name: &str,
        value: &Option<Value>,
    ) {
        if let Some(value) = value {
            self.scopes[*env].restore_binding(name.to_owned(), Some(value.clone()));
        } else {
            self.scopes[*env].restore_binding(name.to_owned(), None);
        }
    }
    pub(super) fn restore_saved_dense(&mut self, id: &u64, index: &usize, value: &Option<Value>) {
        if let Some(value) = value {
            let slots = self.argument_objects.get_mut(id).expect("incoming object");
            slots.resize(slots.len().max(index + 1), Value::Unknown);
            slots[*index] = value.clone();
        }
    }
    pub(super) fn restore_saved_length(&mut self, id: &u64, length: &usize) {
        let old_length = self.argument_objects[id].len();
        self.argument_objects
            .get_mut(id)
            .expect("incoming object")
            .resize(*length, Value::Unknown);
        for index in *length..old_length {
            self.note_reference_cell(&Cell::Dense(*id, index));
        }
    }
    pub(super) fn restore_saved_extra(&mut self, id: &u64, index: &usize, value: &Option<Value>) {
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
    pub(super) fn restore_saved_builder(&mut self, id: &u64, value: &Option<Value>) {
        if let Some(value) = value {
            self.builder_updates.insert(*id, value.clone());
        } else {
            self.builder_updates.remove(id);
        }
    }
    pub(super) fn restore_saved_fresh(&mut self, env: &Environment, name: &str, present: &bool) {
        if *present {
            self.fresh_mapped_parameters
                .entry(*env)
                .or_default()
                .insert(name.to_owned());
        } else if let Some(names) = self.fresh_mapped_parameters.get_mut(env) {
            names.remove(name);
            if names.is_empty() {
                self.fresh_mapped_parameters.remove(env);
            }
        }
    }
}
