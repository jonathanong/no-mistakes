use super::{Evaluator, Value};
use crate::fx::{fx_set, FxHashSet};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn collect_builder_ids(
        &self,
        values: &[Value],
        visit: impl FnMut(()),
    ) -> FxHashSet<u64> {
        self.collect_builder_ids_from(values, visit)
    }

    pub(super) fn collect_builder_ids_from<'a>(
        &self,
        values: impl IntoIterator<Item = &'a Value>,
        mut visit: impl FnMut(()),
    ) -> FxHashSet<u64> {
        let mut ids = fx_set();
        let mut visited = fx_set();
        for value in values {
            self.collect_builder_value(value, &mut ids, &mut visited, &mut visit);
        }
        ids
    }

    pub(super) fn collect_builder_value(
        &self,
        value: &Value,
        ids: &mut FxHashSet<u64>,
        visited: &mut FxHashSet<*const ()>,
        visit: &mut impl FnMut(()),
    ) {
        match value {
            Value::Prefix(_, _, Some(id)) => {
                ids.insert(*id);
            }
            Value::Promise(value) | Value::Evaluated(value, _) => {
                self.collect_builder_value(value, ids, visited, visit)
            }
            Value::Aggregate(values)
            | Value::References(values)
            | Value::Object(values)
            | Value::Record(values, _)
            | Value::Joined(values)
            | Value::Possible(values) => {
                // Values are immutable for this read-only collection. Mutable
                // argument/array slots retain their independent identity rules.
                if visited.insert(values.identity()) {
                    visit(());
                    for value in values {
                        self.collect_builder_value(value, ids, visited, visit);
                    }
                }
            }
            Value::Arguments(id) | Value::Array(id) if ids.insert(*id) => {
                for (index, value) in self.argument_slots(*id) {
                    if !self.definite_deleted_argument_slots.contains(&(*id, index)) {
                        self.collect_builder_value(value, ids, visited, visit);
                    }
                }
            }
            _ => {}
        }
    }
}
