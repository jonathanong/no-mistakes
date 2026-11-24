use super::super::super::value::Values;
use super::super::super::{Environment, Scope, Value};
use crate::fx::FxHashMap;
use std::sync::Arc;

pub(super) fn remap_scope(scope: &Scope, remap: &[Environment]) -> Scope {
    if !scope.values().any(|value| needs_remap(value, remap)) {
        return scope.clone();
    }
    let values = scope
        .iter()
        .map(|(name, value)| (name.clone(), remap_value(value, remap)))
        .collect::<FxHashMap<_, _>>();
    Scope::from(values)
}

fn needs_remap(value: &Value, remap: &[Environment]) -> bool {
    match value {
        Value::Function(_, _, env) => remap
            .get(*env)
            .copied()
            .is_some_and(|mapped| mapped != *env),
        Value::Promise(inner) | Value::Evaluated(inner, _) => needs_remap(inner, remap),
        Value::Aggregate(items)
        | Value::References(items)
        | Value::Object(items)
        | Value::Record(items, _)
        | Value::Joined(items)
        | Value::Possible(items) => items.iter().any(|item| needs_remap(item, remap)),
        Value::Prefix(..)
        | Value::Arguments(_)
        | Value::Array(_)
        | Value::Number(_)
        | Value::Boolean(_)
        | Value::Failure(_)
        | Value::Unknown
        | Value::Primitive
        | Value::Unsupported
        | Value::SlotDeletion => false,
    }
}

fn remap_values(items: &Values, remap: &[Environment]) -> Values {
    Values::from(
        items
            .iter()
            .map(|item| remap_value(item, remap))
            .collect::<Vec<_>>(),
    )
}

fn remap_value(value: &Value, remap: &[Environment]) -> Value {
    match value {
        Value::Function(function, path, env) => Value::Function(
            Arc::clone(function),
            Arc::clone(path),
            remap.get(*env).copied().unwrap_or(*env),
        ),
        Value::Promise(inner) => Value::Promise(Box::new(remap_value(inner, remap))),
        Value::Evaluated(inner, exact) => {
            Value::Evaluated(Box::new(remap_value(inner, remap)), *exact)
        }
        Value::Aggregate(items) => Value::Aggregate(remap_values(items, remap)),
        Value::References(items) => Value::References(remap_values(items, remap)),
        Value::Object(items) => Value::Object(remap_values(items, remap)),
        Value::Record(items, keys) => Value::Record(remap_values(items, remap), keys.clone()),
        Value::Joined(items) => Value::Joined(remap_values(items, remap)),
        Value::Possible(items) => Value::Possible(remap_values(items, remap)),
        _ => value.clone(),
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "remap_tests.rs"]
mod tests;
