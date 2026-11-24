use serde_yaml::{Mapping, Value};
use std::collections::BTreeMap;

pub(super) fn guards(condition: Option<&Value>) -> bool {
    condition
        .and_then(Value::as_str)
        .is_some_and(|text| text.contains("github.event_name"))
}

pub(super) fn events(value: &Value) -> Vec<String> {
    let Some(on) = value
        .get("on")
        .or_else(|| value.as_mapping()?.get(Value::Bool(true)))
    else {
        return Vec::new();
    };
    match on {
        Value::String(name) => vec![name.clone()],
        Value::Sequence(names) => names.iter().filter_map(string).collect(),
        Value::Mapping(map) => map.keys().filter_map(string).collect(),
        _ => Vec::new(),
    }
}

fn string(value: &Value) -> Option<String> {
    value.as_str().map(str::to_string)
}

pub(super) fn env_of(value: Option<&Value>) -> BTreeMap<String, String> {
    let Some(map) = value.and_then(Value::as_mapping) else {
        return BTreeMap::new();
    };
    map.iter()
        .filter_map(|(key, value)| Some((key.as_str()?.to_string(), value.as_str()?.to_string())))
        .collect()
}

pub(super) fn get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(Value::String(key.to_owned()))
}
