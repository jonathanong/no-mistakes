use super::{Environment, Evaluator, Value};
use crate::fx::FxHashSet;
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
        let mut seen = FxHashSet::default();
        for (name, value) in values {
            let value = if precise {
                self.failed_scope_value(value, unsupported, depth, &mut seen)
            } else {
                unsupported.clone()
            };
            self.scopes[env].insert(name, value);
        }
    }

    pub(super) fn failed_scope_value(
        &mut self,
        value: Value,
        unsupported: &Value,
        depth: u8,
        seen: &mut FxHashSet<u64>,
    ) -> Value {
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
                    .map(|value| self.failed_scope_value(value, unsupported, depth, seen))
                    .collect::<Vec<_>>()
                    .into();
                Value::Record(values, keys)
            }
            Value::Object(values) => Value::Object(
                values
                    .into_iter()
                    .map(|value| self.failed_scope_value(value, unsupported, depth, seen))
                    .collect::<Vec<_>>()
                    .into(),
            ),
            Value::References(values) => self.failed_scope_callbacks(Value::References(values)),
            Value::Array(id) => {
                self.failed_scope_array(id, unsupported, depth, seen);
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
                Value::Record(values, _) | Value::Object(values) | Value::References(values) => {
                    pending.extend(values)
                }
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
