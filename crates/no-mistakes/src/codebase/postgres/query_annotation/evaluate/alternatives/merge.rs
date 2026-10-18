use super::super::Value;

fn alternatives(value: Value, values: &mut Vec<Value>) {
    match value {
        // Flatten both wrappers so a nested branch join stays a flat candidate
        // list. Possible carries the implicit unknown that prefix projection
        // already treats as an unsafe-or-unproven outcome.
        Value::Aggregate(children) | Value::Joined(children) | Value::Possible(children) => {
            for child in children {
                alternatives(child, values);
            }
        }
        other => {
            if !values.contains(&other) {
                values.push(other);
            }
        }
    }
}

pub(super) fn value(before: Value, after: Value) -> Value {
    if before == after {
        return before;
    }
    let mut values = Vec::new();
    alternatives(before, &mut values);
    alternatives(after, &mut values);
    // Sparse slots use the same candidate join as dense argument vectors.
    Value::Possible(values.into())
}

#[cfg(test)]
pub(super) mod tests;
