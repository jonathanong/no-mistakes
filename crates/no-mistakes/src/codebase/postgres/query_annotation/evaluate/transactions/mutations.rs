use super::{Cell, Saved};
use crate::codebase::postgres::query_annotation::evaluate::{Environment, Evaluator, Value};
use std::path::{Path, PathBuf};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn write_binding(
        &mut self,
        env: Environment,
        name: String,
        value: Value,
    ) {
        if self.scopes[env].get(&name) == Some(&value) {
            return;
        }
        self.save_cell(Cell::Binding(env, name.clone()));
        self.scopes[env].restore_binding(name.clone(), Some(value));
        self.note_reference_cell(&Cell::Binding(env, name));
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn reserve_binding(
        &mut self,
        env: Environment,
        name: String,
        value: Value,
    ) {
        if !self.scopes[env].contains_key(&name) {
            self.write_binding(env, name, value);
        }
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn write_dense_slot(
        &mut self,
        id: u64,
        index: usize,
        value: Value,
    ) {
        if self.argument_objects[&id].get(index) == Some(&value) {
            return;
        }
        if index == 0 {
            self.save_cell(Cell::Length(id));
        }
        self.save_cell(Cell::Dense(id, index));
        self.argument_objects.get_mut(&id).expect("existing object")[index] = value;
        self.note_reference_cell(&Cell::Dense(id, index));
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn write_sparse_slot(
        &mut self,
        id: u64,
        index: usize,
        value: Option<Value>,
    ) {
        if self
            .argument_extra_slots
            .get(&id)
            .and_then(|slots| slots.get(&index))
            == value.as_ref()
        {
            return;
        }
        let cell = Cell::Extra(id, index);
        self.save_cell(cell.clone());
        self.restore_cell(&cell, &Saved::Value(value));
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn replace_dense_object(
        &mut self,
        id: u64,
        values: Vec<Value>,
    ) {
        if self.argument_objects.get(&id) == Some(&values) {
            return;
        }
        if let Some(old) = self.argument_objects.get(&id) {
            let length = old.len();
            self.save_cell(Cell::Length(id));
            for index in 0..length.max(values.len()) {
                self.save_cell(Cell::Dense(id, index));
            }
        }
        self.argument_objects.insert(id, values);
        self.register_argument_references(id);
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn append_dense_slots(
        &mut self,
        id: u64,
        values: &[Value],
    ) {
        if values.is_empty() {
            return;
        }
        self.save_cell(Cell::Length(id));
        let start = self.argument_objects[&id].len();
        for index in start..start + values.len() {
            self.save_cell(Cell::Dense(id, index));
        }
        self.argument_objects
            .get_mut(&id)
            .expect("existing object")
            .extend_from_slice(values);
        for index in start..start + values.len() {
            self.note_reference_cell(&Cell::Dense(id, index));
        }
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn set_transaction_flag(
        &mut self,
        cell: Cell,
        value: bool,
    ) {
        self.apply_cell(&cell, &Saved::Flag(value));
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn write_builder(
        &mut self,
        id: u64,
        value: Value,
    ) {
        if self.builder_updates.get(&id) == Some(&value) {
            return;
        }
        let cell = Cell::Builder(id);
        self.save_cell(cell.clone());
        self.builder_updates.insert(id, value);
        self.note_reference_cell(&cell);
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn create_argument_object(
        &mut self,
        id: u64,
        values: Vec<Value>,
    ) {
        self.argument_objects.insert(id, values);
        self.register_argument_references(id);
    }
}
