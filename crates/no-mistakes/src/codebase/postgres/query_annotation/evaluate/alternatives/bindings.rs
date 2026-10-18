use super::super::Value;
#[cfg(test)]
use crate::codebase::postgres::query_annotation::evaluate::Scope;
#[cfg(test)]
use crate::fx::FxHashMap;

fn scalar(value: &Value) -> bool {
    match value {
        Value::Unknown | Value::Prefix(_, _, None) => true,
        Value::Evaluated(inner, _) => scalar(inner),
        Value::Aggregate(values)
        | Value::References(values)
        | Value::Object(values)
        | Value::Record(values, _)
        | Value::Joined(values) => values.iter().all(scalar),
        _ => false,
    }
}

fn annotated_builder(value: &Value, id: u64) -> bool {
    matches!(value, Value::Prefix(text, true, Some(current)) if *current == id && text.trim_start().starts_with("/*"))
}

fn candidates(value: Value, joined: &mut Vec<Value>) {
    match value {
        Value::Joined(values) => {
            for value in values {
                candidates(value, joined);
            }
        }
        value => {
            if !joined.contains(&value) {
                joined.push(value);
            }
        }
    }
}

#[cfg(test)]
const UNPROVEN: Value = Value::Unknown;

/// A binding absent from one arm is not SQL proof. Callers compare this instead
/// of indexing a divergent scope.
#[cfg(test)]
pub(super) fn binding_or_unproven<'a>(
    scope: &'a FxHashMap<String, Value>,
    name: &str,
) -> &'a Value {
    scope.get(name).unwrap_or(&UNPROVEN)
}

/// Joins preserve an annotated builder prefix only when the original scope and
/// both arms still have that builder. A missing arm or original binding is
/// unproven and must not keep or synthesize a SQL prefix.
#[cfg(test)]
pub(super) fn join_with_sql(
    joined: &mut [Scope],
    current: &[Scope],
    original: &[Scope],
    whole_sql: bool,
) {
    for ((before, after), initial) in joined.iter_mut().zip(current).zip(original) {
        // Inspect shared snapshots before taking a mutable view: unchanged
        // branches must not clone every binding in the captured environment.
        let changed = before
            .iter()
            .filter(|(name, value)| {
                after.get(*name) != Some(*value) || !initial.contains_key(*name)
            })
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        for name in changed {
            let value = before.get_mut(&name).expect("existing binding");
            *value = join_value(initial.get(&name), value, after.get(&name), whole_sql);
        }
    }
}

#[cfg(test)]
mod tests;

pub(super) fn join_value(
    initial: Option<&Value>,
    before: &Value,
    after: Option<&Value>,
    whole_sql: bool,
) -> Value {
    let (Some(initial), Some(after)) = (initial, after) else {
        return Value::Unknown;
    };
    if before == after {
        return before.clone();
    }
    if let Value::Prefix(_, _, Some(id)) = initial {
        if !whole_sql
            && annotated_builder(initial, *id)
            && annotated_builder(before, *id)
            && annotated_builder(after, *id)
        {
            return initial.clone();
        }
    }
    if !whole_sql && scalar(before) && scalar(after) {
        Value::Unknown
    } else {
        let mut values = Vec::new();
        candidates(before.clone(), &mut values);
        candidates(after.clone(), &mut values);
        super::super::concat::joined(values)
    }
}
