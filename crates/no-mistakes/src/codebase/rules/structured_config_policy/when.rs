use super::{value_at_key, PolicyWhen};
use serde_yaml::Value;

pub(super) fn policy_applies(value: &Value, when: &[PolicyWhen]) -> bool {
    when.iter()
        .all(|condition| key_present(value, &condition.key))
}

fn key_present(value: &Value, key: &str) -> bool {
    match value_at_key(value, key) {
        Some(Value::Sequence(items)) => !items.is_empty(),
        Some(Value::String(text)) => !text.is_empty(),
        _ => false,
    }
}
