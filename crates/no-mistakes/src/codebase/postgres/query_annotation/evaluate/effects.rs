use super::{Environment, Evaluator, Value};
use crate::fx::{fx_set, FxHashSet};
use std::path::PathBuf;

impl<F: Fn(&str, &std::path::Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn invalidate_builders(&mut self, values: &[Value]) {
        fn collect(value: &Value, ids: &mut FxHashSet<u64>) {
            match value {
                Value::Prefix(_, _, Some(id)) => {
                    ids.insert(*id);
                }
                Value::Promise(value) => collect(value, ids),
                Value::Aggregate(values) | Value::Arguments(_, values) => {
                    // Opaque code can replace argument slots even when their
                    // current values are immutable strings.
                    if let Value::Arguments(id, _) = value {
                        ids.insert(*id);
                    }
                    for value in values {
                        collect(value, ids);
                    }
                }
                _ => {}
            }
        }
        let mut ids = fx_set();
        for value in values {
            collect(value, &mut ids);
        }
        self.invalidated_builders.extend(ids.iter().copied());
        for scope in &mut self.scopes {
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
        let Value::Prefix(_, _, Some(id)) = replacement else {
            return;
        };
        fn replace(value: &mut Value, id: u64, replacement: &Value) {
            if matches!(value, Value::Prefix(_, _, Some(other)) if *other == id) {
                *value = replacement.clone();
            } else {
                match value {
                    Value::Promise(value) => replace(value, id, replacement),
                    Value::Arguments(_, values) | Value::Aggregate(values) => {
                        for value in values {
                            replace(value, id, replacement);
                        }
                    }
                    _ => {}
                }
            }
        }
        for scope in &mut self.scopes {
            for value in scope.values_mut() {
                replace(value, *id, replacement);
            }
        }
    }
}
