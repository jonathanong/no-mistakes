use serde_yaml::{Mapping, Value};

pub(super) fn has_event(doc: &Mapping, event: &str) -> bool {
    match doc.get("on") {
        Some(Value::String(name)) => name == event,
        Some(Value::Sequence(items)) => items.iter().any(|item| item.as_str() == Some(event)),
        Some(Value::Mapping(map)) => map.keys().any(|key| key.as_str() == Some(event)),
        _ => false,
    }
}
