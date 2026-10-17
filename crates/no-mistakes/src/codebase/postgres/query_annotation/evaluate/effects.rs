mod builder_ids;
#[cfg(test)]
mod tests;

use super::{Environment, Evaluator, Value};
use crate::fx::FxHashSet;
use std::path::PathBuf;

impl<F: Fn(&str, &std::path::Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn invalidate_live_builders(&mut self) {
        let values = self.live_builder_values();
        self.invalidate_opaque_builders(&values);
    }

    pub(super) fn invalidate_opaque_builders(&mut self, values: &[Value]) {
        // Opaque hooks can patch intrinsics even when no builder exists yet.
        self.untrusted_array_intrinsics = true;
        self.invalidate_builders(values);
    }

    pub(super) fn live_builder_values(&self) -> Vec<Value> {
        self.scopes
            .iter()
            .flat_map(|scope| scope.values().cloned())
            .collect()
    }

    pub(super) fn invalidate_builders(&mut self, values: &[Value]) {
        let ids = self.collect_builder_ids(values, std::mem::drop);
        self.invalidate_builder_ids(ids);
    }

    fn invalidate_builder_ids(&mut self, ids: FxHashSet<u64>) {
        // Most opaque calls carry no SQL builders. Do not scan every frame
        // (or detach shared snapshots) when there is nothing to invalidate.
        if ids.is_empty() {
            return;
        }
        self.invalidate_mapped_freshness(&ids);
        self.invalidated_builders.extend(ids.iter().copied());
        for scope in &mut self.scopes {
            if !scope.contains_builders(&ids) {
                continue;
            }
            for value in scope.values_mut() {
                if matches!(value, Value::Prefix(_, _, Some(id)) if ids.contains(id)) {
                    *value = Value::Unknown;
                }
            }
        }
    }

    pub(super) fn invalidate_captured(&mut self, env: Environment, function: &super::Function) {
        let mut shadowed = function.params.iter().cloned().collect::<FxHashSet<_>>();
        shadowed.extend(function.self_name.iter().cloned());
        if !function.arrow {
            shadowed.insert("arguments".into());
        }
        for step in &function.body {
            match step {
                crate::codebase::postgres::query_annotation::Step::Bind(name, _)
                | crate::codebase::postgres::query_annotation::Step::Hoisted(name, _)
                | crate::codebase::postgres::query_annotation::Step::Var(name) => {
                    shadowed.insert(name.clone());
                }
                crate::codebase::postgres::query_annotation::Step::Reserve(names) => {
                    shadowed.extend(names.iter().cloned())
                }
                _ => {}
            }
        }
        let values = self.scopes[env]
            .iter()
            .filter(|(name, _)| !shadowed.contains(*name))
            .map(|(_, value)| value.clone())
            .collect::<Vec<_>>();
        self.invalidate_builders(&values);
        for (name, value) in &mut self.scopes[env] {
            if !shadowed.contains(name) && matches!(value, Value::Prefix(_, _, _)) {
                *value = Value::Unknown;
            }
        }
    }
    pub(super) fn replace_builder(&mut self, replacement: &Value) {
        fn identity(value: &Value) -> Option<u64> {
            match value {
                Value::Prefix(_, _, Some(id)) => Some(*id),
                Value::Joined(values) if !values.is_empty() => {
                    let id = identity(&values[0])?;
                    values
                        .iter()
                        .all(|value| identity(value) == Some(id))
                        .then_some(id)
                }
                _ => None,
            }
        }
        let Some(id) = identity(replacement) else {
            return;
        };
        self.builder_updates.insert(id, replacement.clone());
        fn replace(value: &mut Value, id: u64, replacement: &Value) {
            if matches!(value, Value::Prefix(_, _, Some(other)) if *other == id) {
                *value = replacement.clone();
            } else {
                match value {
                    Value::Promise(value) | Value::Evaluated(value, _) => {
                        replace(value, id, replacement)
                    }
                    Value::Aggregate(values)
                    | Value::References(values)
                    | Value::Object(values)
                    | Value::Record(values, _)
                    | Value::Joined(values)
                    | Value::Possible(values) => {
                        for value in values {
                            replace(value, id, replacement);
                        }
                    }
                    _ => {}
                }
            }
        }
        let ids = FxHashSet::from_iter([id]);
        for scope in &mut self.scopes {
            if !scope.contains_builders(&ids) {
                continue;
            }
            for value in scope.values_mut() {
                replace(value, id, replacement);
            }
        }
        // Argument references are identity-only; update each arena slot once.
        for values in self.argument_objects.values_mut() {
            for value in values {
                replace(value, id, replacement);
            }
        }
        for values in self.argument_extra_slots.values_mut() {
            for value in values.values_mut() {
                replace(value, id, replacement);
            }
        }
    }
}
