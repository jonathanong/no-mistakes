use super::{Evaluator, Value};
use crate::fx::FxHashSet;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn failed_scope_array(
        &mut self,
        id: u64,
        unsupported: &Value,
        depth: u8,
        seen: &mut FxHashSet<u64>,
    ) {
        if !seen.insert(id) {
            return;
        }
        // Metadata weakening preserves shared identity, sparse keys and array length.
        let dense = self.argument_objects[&id].clone();
        let extras = self.argument_extra_slots.get(&id).cloned();
        let dense = dense
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                self.failed_array_slot(id, index, value, unsupported, depth, seen)
            })
            .collect();
        self.replace_dense_object(id, dense);
        if let Some(extras) = extras {
            let extras: Vec<_> = extras
                .into_iter()
                .map(|(index, value)| {
                    (
                        index,
                        self.failed_array_slot(id, index, value, unsupported, depth, seen),
                    )
                })
                .collect();
            for (index, value) in extras {
                self.write_sparse_slot(id, index, Some(value));
            }
        }
        // Deletion, recreation and shape uncertainty already use this same ID.
    }

    fn failed_array_slot(
        &mut self,
        id: u64,
        index: usize,
        value: Value,
        unsupported: &Value,
        depth: u8,
        seen: &mut FxHashSet<u64>,
    ) -> Value {
        if self.definite_deleted_argument_slots.contains(&(id, index)) {
            unsupported.clone()
        } else {
            self.failed_scope_value(value, unsupported, depth, seen)
        }
    }
}
