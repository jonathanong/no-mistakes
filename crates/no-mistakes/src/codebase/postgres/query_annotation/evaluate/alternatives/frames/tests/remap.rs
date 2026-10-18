use super::super::super::value::Values;
use super::{Environment, Value};
use crate::fx::FxHashMap;

pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn remap(
    value: &mut Value,
    indices: &FxHashMap<Environment, Environment>,
) {
    if indices.iter().all(|(before, after)| before == after) {
        return;
    }
    remap_changed(value, indices);
}

pub(super) fn remap_changed(value: &mut Value, indices: &FxHashMap<Environment, Environment>) {
    match value {
        Value::Function(_, _, env) => {
            if let Some(index) = indices.get(env) {
                *env = *index;
            }
        }
        Value::Promise(value) | Value::Evaluated(value, _) => remap_changed(value, indices),
        Value::Aggregate(values)
        | Value::References(values)
        | Value::Object(values)
        | Value::Record(values, _)
        | Value::Joined(values)
        | Value::Possible(values)
            if changes_environment(values, indices) =>
        {
            for value in values {
                remap_changed(value, indices);
            }
        }
        _ => {}
    }
}

fn changes_environment(values: &Values, indices: &FxHashMap<Environment, Environment>) -> bool {
    values.contains_environment()
        && values
            .environment_indices()
            .iter()
            .any(|env| indices.get(env).is_some_and(|after| after != env))
}
