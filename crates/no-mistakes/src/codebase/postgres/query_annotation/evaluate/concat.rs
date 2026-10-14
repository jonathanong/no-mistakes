use super::Value;
use crate::codebase::postgres::embedded::{count_placeholders, renumber_placeholders};

pub(super) fn concat(base: Value, tail: Value) -> Value {
    let base = base.exposed();
    let tail = tail.exposed();
    match (base, tail) {
        (Value::Failure(reason), _) | (_, Value::Failure(reason)) => Value::Failure(reason),
        (Value::Joined(bases), tail) => joined(
            bases
                .into_iter()
                .map(|base| concat(base, tail.clone()))
                .collect(),
        ),
        (base, Value::Joined(tails)) => joined(
            tails
                .into_iter()
                .map(|tail| concat(base.clone(), tail))
                .collect(),
        ),
        (base @ Value::Prefix(_, _, _), Value::Number(number)) => {
            concat(base, Value::Prefix(number.to_string(), true, None))
        }
        (base @ Value::Prefix(_, _, _), Value::Boolean(boolean)) => {
            concat(base, Value::Prefix(boolean.to_string(), true, None))
        }
        (Value::Boolean(boolean), tail @ Value::Prefix(_, _, None)) => {
            concat(Value::Prefix(boolean.to_string(), true, None), tail)
        }
        (Value::Unsupported, _) => Value::Unsupported,
        (Value::Prefix(base, true, _), Value::Unsupported) if base.trim().is_empty() => {
            Value::Unsupported
        }
        (Value::Prefix(base, false, id), _) => Value::Prefix(base, false, id),
        (Value::Prefix(mut base, true, id), Value::Prefix(tail, complete, _)) => {
            base.push_str(&renumber_placeholders(&tail, count_placeholders(&base)));
            Value::Prefix(base, complete, id)
        }
        (Value::Prefix(base, true, id), _) => Value::Prefix(base, false, id),
        _ => Value::Unknown,
    }
}

pub(super) fn joined(values: Vec<Value>) -> Value {
    let mut unique = Vec::new();
    for value in values {
        let values = match value.exposed() {
            Value::Joined(values) => values.into_iter().collect(),
            value => vec![value],
        };
        for value in values {
            if !unique.contains(&value) {
                unique.push(value);
            }
            if unique.len() > crate::codebase::postgres::MAX_EMBEDDED_SQL_VARIANTS {
                return Value::Failure("variant-limit");
            }
        }
    }
    if unique.len() == 1 {
        unique.pop().unwrap()
    } else {
        Value::Joined(unique.into())
    }
}

pub(super) fn alternative_result(returned: Vec<Value>, precise: bool) -> Value {
    if precise {
        let handled = returned
            .iter()
            .all(|value| matches!(value, Value::Evaluated(_, true)));
        let value = joined(returned);
        if handled {
            Value::Evaluated(Box::new(value), true)
        } else {
            value
        }
    } else {
        Value::Aggregate(returned.into())
    }
}
