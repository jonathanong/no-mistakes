use serde_yaml::{Mapping, Value};

/// Copy YAML 1.1 merge keys into their parent mapping.
///
/// `serde_yaml` resolves aliases and then leaves `<<` as an ordinary key.
/// An explicit key already on the parent wins. In a sequence, an earlier
/// mapping wins over a later one. A merge value that is not a mapping, or a
/// sequence of mappings, is dropped.
pub(super) fn expand_merge_keys(value: Value) -> Value {
    match value {
        Value::Sequence(items) => {
            Value::Sequence(items.into_iter().map(expand_merge_keys).collect())
        }
        Value::Mapping(mapping) => Value::Mapping(expand_mapping(mapping)),
        other => other,
    }
}

fn expand_mapping(mut mapping: Mapping) -> Mapping {
    if let Some(merged) = mapping.shift_remove("<<") {
        apply_merge(&mut mapping, &expand_merge_keys(merged));
    }
    mapping
        .into_iter()
        .map(|(key, child)| (key, expand_merge_keys(child)))
        .collect()
}

fn apply_merge(mapping: &mut Mapping, merged: &Value) {
    match merged {
        Value::Mapping(source) => insert_missing(mapping, source),
        Value::Sequence(items) => insert_missing(mapping, &overlay_sequence(items)),
        _ => {}
    }
}

fn overlay_sequence(items: &[Value]) -> Mapping {
    let mut overlay = Mapping::new();
    for item in items.iter().rev() {
        if let Value::Mapping(source) = item {
            for (key, value) in source {
                overlay.insert(key.clone(), value.clone());
            }
        }
    }
    overlay
}

fn insert_missing(mapping: &mut Mapping, source: &Mapping) {
    for (key, value) in source {
        mapping.entry(key.clone()).or_insert_with(|| value.clone());
    }
}
