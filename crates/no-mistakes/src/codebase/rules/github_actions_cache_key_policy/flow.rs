use super::scan::{Field, Step};
use serde_yaml::{Mapping, Value};

pub(super) fn fill_flow(step: &mut Step, line: usize, body: &str) {
    let Ok(value) = serde_yaml::from_str::<Value>(body) else {
        return;
    };
    let Some(map) = value.as_mapping() else {
        return;
    };
    if let Some(text) = map_text(map, "id") {
        step.id = Some(text);
    }
    if let Some(text) = map_text(map, "uses") {
        step.uses = Some(text);
    }
    if let Some(text) = map_text(map, "run") {
        step.run = Some(text);
    }
    if let Some(with) = map
        .get(Value::String("with".into()))
        .and_then(Value::as_mapping)
    {
        fill_with(step, line, with);
    }
}

pub(super) fn fill_with_text(step: &mut Step, line: usize, body: &str) {
    let Ok(value) = serde_yaml::from_str::<Value>(body) else {
        return;
    };
    let Some(map) = value.as_mapping() else {
        return;
    };
    fill_with(step, line, map);
}

fn fill_with(step: &mut Step, line: usize, map: &Mapping) {
    set_field(&mut step.path, line, map, "path");
    set_field(&mut step.key, line, map, "key");
    set_field(&mut step.restore_keys, line, map, "restore-keys");
}

fn set_field(slot: &mut Option<Field>, line: usize, map: &Mapping, key: &str) {
    if let Some(value) = scalar_text(map.get(Value::String(key.into()))) {
        *slot = Some(Field { line, value });
    }
}

fn map_text(map: &Mapping, key: &str) -> Option<String> {
    scalar_text(map.get(Value::String(key.into())))
}

fn scalar_text(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        Value::Sequence(items) => {
            let parts: Vec<_> = items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect();
            (!parts.is_empty()).then(|| parts.join("\n"))
        }
        _ => None,
    }
}

pub(super) fn split_kv(text: &str) -> Option<(&str, &str)> {
    let (key, value) = text.split_once(':')?;
    let plain = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    plain.then_some((key, value.trim()))
}

pub(super) fn list_item(raw: &str) -> Option<(usize, &str)> {
    let indent = leading(raw);
    raw[indent..]
        .trim_end()
        .strip_prefix("- ")
        .map(|rest| (indent, rest.trim()))
}

pub(super) fn mapping_key(raw: &str) -> Option<(usize, &str, &str)> {
    let indent = leading(raw);
    let body = raw[indent..].trim_end();
    if body.starts_with("- ") || body.starts_with('#') {
        return None;
    }
    let (key, value) = body.split_once(':')?;
    let plain = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '/');
    plain.then_some((indent, key, value.trim()))
}

pub(super) fn comment_value(value: &str) -> &str {
    let text = value.trim();
    if text.starts_with(['"', '\'', '|', '>', '{', '[']) {
        return text;
    }
    text.split(" #").next().unwrap_or(text).trim()
}

pub(super) fn block_header(value: &str) -> bool {
    let token = value
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_end_matches('#');
    let mut chars = token.chars();
    let Some(kind) = chars.next() else {
        return false;
    };
    (kind == '|' || kind == '>') && chars.all(|c| c.is_ascii_digit() || c == '+' || c == '-')
}

pub(super) fn leading(raw: &str) -> usize {
    raw.chars().take_while(|c| *c == ' ').count()
}
