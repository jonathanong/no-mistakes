use super::{Evaluator, Value};
use crate::fx::{fx_set, FxHashSet};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn collect_live_builder_ids(&self, mut visit: impl FnMut(())) -> FxHashSet<u64> {
        let mut ids = fx_set();
        let mut visited = fx_set();
        for scope in &self.scopes {
            if let Some(edges) = scope
                .cached_references()
                .filter(|edges| edges.arguments.iter().all(|id| ids.contains(id)))
            {
                // Prior scopes have completed. Already-seen argument IDs
                // suppress every mutable descent, leaving commuting prefixes.
                ids.extend(edges.identities.iter().copied());
            } else if let Some(values) = scope.cached_reference_bindings() {
                for value in values {
                    super::hooks::record_live_binding!();
                    self.collect_builder_value(value, &mut ids, &mut visited, &mut visit);
                }
            } else {
                for value in scope.values() {
                    super::hooks::record_live_binding!();
                    self.collect_builder_value(value, &mut ids, &mut visited, &mut visit);
                }
            }
        }
        ids
    }

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

    fn collect_builder_value(
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
                // Scalar/callback-only trees cannot contribute builder IDs.
                // Values are immutable for this read-only collection. Mutable
                // argument/array slots retain their independent identity rules.
                if values.contains_reference() && visited.insert(values.identity()) {
                    visit(());
                    for value in values {
                        self.collect_builder_value(value, ids, visited, visit);
                    }
                }
            }
            Value::Arguments(id) | Value::Array(id) if ids.insert(*id) => {
                if self.borrow_dense_builder_ids(*id, ids) {
                    return;
                }
                for (index, value) in self.argument_slots(*id) {
                    if !self.definite_deleted_argument_slots.contains(&(*id, index)) {
                        self.collect_builder_value(value, ids, visited, visit);
                    }
                }
            }
            _ => {}
        }
    }

    fn borrow_dense_builder_ids(&self, id: u64, ids: &mut FxHashSet<u64>) -> bool {
        if !self.definite_deleted_argument_slots.is_empty()
            || self
                .argument_extra_slots
                .get(&id)
                .is_some_and(|slots| !slots.is_empty())
        {
            return false;
        }
        let Some(edges) = self
            .argument_objects
            .get(&id)
            .and_then(|slots| slots.cached_references())
        else {
            return false;
        };
        // Prefix-only inserts commute. Argument edges do not: their mutable
        // recursion and shared ID namespace require the original DFS order.
        if !edges.arguments.is_empty() {
            return false;
        }
        ids.extend(edges.identities.iter().copied());
        true
    }
}
