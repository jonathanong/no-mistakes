use super::super::super::value::Values;
use super::super::super::{Environment, Scope, Value};
use crate::fx::FxHashMap;
use std::sync::Arc;

pub(super) fn remap_scope(scope: &Scope, remap: &[Environment]) -> Scope {
    remap_scope_with(scope, &|env| remap.get(env).copied().unwrap_or(env))
}

pub(in crate::codebase::postgres::query_annotation) fn remap_captured_scope(
    scope: &Scope,
    old_env: Environment,
    new_env: Environment,
    prefix: usize,
) -> Option<Scope> {
    if !scope
        .values()
        .all(|value| envs_within(value, old_env, prefix))
    {
        return None;
    }
    Some(remap_scope_with(scope, &|env| {
        if env == old_env {
            new_env
        } else {
            env
        }
    }))
}

pub(super) fn remap_scope_strict(
    scope: &Scope,
    remap: &FxHashMap<Environment, Environment>,
) -> Option<Scope> {
    if !scope.values().all(|value| envs_in_map(value, remap)) {
        return None;
    }
    Some(remap_scope_with(scope, &|env| {
        remap.get(&env).copied().unwrap_or(env)
    }))
}

fn envs_in_map(value: &Value, remap: &FxHashMap<Environment, Environment>) -> bool {
    match value {
        Value::Function(_, _, env) => remap.contains_key(env),
        Value::Promise(inner) | Value::Evaluated(inner, _) => envs_in_map(inner, remap),
        Value::Aggregate(items)
        | Value::References(items)
        | Value::Object(items)
        | Value::Record(items, _)
        | Value::Joined(items)
        | Value::Possible(items) => items.iter().all(|item| envs_in_map(item, remap)),
        _ => true,
    }
}

fn remap_scope_with(scope: &Scope, remap: &impl Fn(Environment) -> Environment) -> Scope {
    if !scope.values().any(|value| needs_remap(value, remap)) {
        return scope.clone();
    }
    let values = scope
        .iter()
        .map(|(name, value)| (name.clone(), remap_value(value, remap)))
        .collect::<FxHashMap<_, _>>();
    Scope::from(values)
}

fn envs_within(value: &Value, old_env: Environment, prefix: usize) -> bool {
    match value {
        Value::Function(_, _, env) => *env == old_env || *env < prefix,
        Value::Promise(inner) | Value::Evaluated(inner, _) => envs_within(inner, old_env, prefix),
        Value::Aggregate(items)
        | Value::References(items)
        | Value::Object(items)
        | Value::Record(items, _)
        | Value::Joined(items)
        | Value::Possible(items) => items.iter().all(|item| envs_within(item, old_env, prefix)),
        _ => true,
    }
}

fn needs_remap(value: &Value, remap: &impl Fn(Environment) -> Environment) -> bool {
    match value {
        Value::Function(_, _, env) => remap(*env) != *env,
        Value::Promise(inner) | Value::Evaluated(inner, _) => needs_remap(inner, remap),
        Value::Aggregate(items)
        | Value::References(items)
        | Value::Object(items)
        | Value::Record(items, _)
        | Value::Joined(items)
        | Value::Possible(items) => items.iter().any(|item| needs_remap(item, remap)),
        _ => false,
    }
}

fn remap_values(items: &Values, remap: &impl Fn(Environment) -> Environment) -> Values {
    Values::from(
        items
            .iter()
            .map(|item| remap_value(item, remap))
            .collect::<Vec<_>>(),
    )
}

fn remap_value(value: &Value, remap: &impl Fn(Environment) -> Environment) -> Value {
    match value {
        Value::Function(function, path, env) => {
            Value::Function(Arc::clone(function), Arc::clone(path), remap(*env))
        }
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
