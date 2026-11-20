use serde_yaml::{Mapping, Value};

pub(super) fn flatten_mapping(map: &Mapping) -> Mapping {
    let mut out = Mapping::new();
    if let Some(merged) = mapping_get(map, "<<") {
        apply_merge(&mut out, merged);
    }
    for (key, value) in map {
        if key.as_str() == Some("<<") {
            continue;
        }
        out.insert(key.clone(), value.clone());
    }
    out
}

fn apply_merge(out: &mut Mapping, value: &Value) {
    match value {
        Value::Mapping(map) => {
            if let Some(nested) = mapping_get(map, "<<") {
                apply_merge(out, nested);
            }
            for (key, value) in map {
                if key.as_str() == Some("<<") {
                    continue;
                }
                out.insert(key.clone(), value.clone());
            }
        }
        Value::Sequence(items) => {
            // YAML 1.1: an earlier mapping overrides a later one.
            for item in items.iter().rev() {
                apply_merge(out, item);
            }
        }
        _ => {}
    }
}

fn mapping_get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(Value::String(key.to_string()))
}
