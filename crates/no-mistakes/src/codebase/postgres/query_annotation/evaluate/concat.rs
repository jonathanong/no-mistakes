use super::Value;

pub(super) fn concat(base: Value, tail: Value) -> Value {
    match (base, tail) {
        (Value::Unsupported, _) => Value::Unsupported,
        (Value::Prefix(base, true, _), Value::Unsupported) if base.trim().is_empty() => {
            Value::Unsupported
        }
        (Value::Prefix(base, false, id), _) => Value::Prefix(base, false, id),
        (Value::Prefix(mut base, true, id), Value::Prefix(tail, complete, _)) => {
            base.push_str(&tail);
            Value::Prefix(base, complete, id)
        }
        (Value::Prefix(base, true, id), _) => Value::Prefix(base, false, id),
        _ => Value::Unknown,
    }
}
