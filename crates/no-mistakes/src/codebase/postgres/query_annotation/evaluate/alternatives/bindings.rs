use super::super::Value;

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
pub(super) mod tests;

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
