use super::{Environment, Evaluator, Value};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn poison_scope(
        &mut self,
        env: Environment,
        unsupported: &Value,
        depth: u8,
        precise: bool,
    ) {
        let values = self.scopes[env]
            .iter()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect::<Vec<_>>();
        for (name, value) in values {
            let value = if precise {
                self.failed_scope_value(value, unsupported, depth)
            } else {
                unsupported.clone()
            };
            self.scopes[env].insert(name, value);
        }
    }

    fn failed_scope_value(&mut self, value: Value, unsupported: &Value, depth: u8) -> Value {
        let Some(depth) = depth.checked_sub(1) else {
            return self.failed_scope_callbacks(value);
        };
        match value {
            Value::Function(mut function, owner, captured) => {
                std::sync::Arc::make_mut(&mut function).supported = false;
                Value::Function(function, owner, captured)
            }
            Value::Record(values, keys) => {
                let values = values
                    .into_iter()
                    .map(|value| self.failed_scope_value(value, unsupported, depth))
                    .collect::<Vec<_>>()
                    .into();
                Value::Record(values, keys)
            }
            Value::References(values) => self.failed_scope_callbacks(Value::References(values)),
            Value::Array(id) => {
                let mut values = Vec::new();
                for (index, value) in self.argument_slots(id) {
                    // Keep authored positions: deleted slots must not shift a callback.
                    values.resize(index + 1, unsupported.clone());
                    if !self.definite_deleted_argument_slots.contains(&(id, index)) {
                        values[index] = value.clone();
                    }
                }
                let values = values
                    .into_iter()
                    .map(|value| self.failed_scope_value(value, unsupported, depth))
                    .collect();
                let id = self.next_builder;
                self.next_builder += 1;
                self.argument_objects.insert(id, values);
                Value::Array(id)
            }
            _ => unsupported.clone(),
        }
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn failed_scope_callbacks(
        &self,
        value: Value,
    ) -> Value {
        // Retain callable identities only; cycles spend no recursive stack.
        let mut pending = vec![value];
        let mut seen = crate::fx::FxHashSet::default();
        let mut callbacks = Vec::new();
        while let Some(value) = pending.pop() {
            match value {
                Value::Array(id) if seen.insert(id) => pending.extend(self.live_callbacks(id)),
                Value::Record(values, _) | Value::References(values) => pending.extend(values),
                Value::Function(mut function, owner, captured) => {
                    std::sync::Arc::make_mut(&mut function).supported = false;
                    callbacks.push(Value::Function(function, owner, captured));
                }
                _ => {}
            }
        }
        Value::References(callbacks.into())
    }
}
