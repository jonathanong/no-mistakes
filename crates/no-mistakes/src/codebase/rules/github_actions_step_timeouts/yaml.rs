use super::{RuleFinding, RULE_ID};
use serde_yaml::{Mapping, Value};

pub(super) fn has_key(map: &Mapping, key: &str) -> bool {
    map.contains_key(Value::String(key.into()))
}
pub(super) fn mapping_str<'a>(map: &'a Mapping, key: &str) -> Option<&'a str> {
    map.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
}
pub(super) fn timeout_minutes(value: &Value) -> Option<u64> {
    let minutes = value.as_u64()?;
    (minutes > 0).then_some(minutes)
}

pub(super) fn is_expression(value: &Value) -> bool {
    value.as_str().is_some_and(|text| {
        let text = text.trim();
        text.starts_with("${{") && text.ends_with("}}")
    })
}

pub(super) fn yaml_key_value(line: &str) -> Option<(&str, &str)> {
    let text = line.trim_start();
    let text = text.strip_prefix("- ").unwrap_or(text);
    let (key, value) = text.split_once(':')?;
    let key = key.trim().trim_matches(['\'', '"']);
    (!key.is_empty() && !key.starts_with('#')).then_some((key, value.trim()))
}

pub(super) fn is_inline_value_line(line: &str) -> bool {
    yaml_key_value(line).is_some_and(|(_, value)| {
        let value = if value.starts_with('&') {
            value.split_whitespace().nth(1).unwrap_or("")
        } else {
            value
        };
        value.starts_with(&['*', '[', '{'][..])
    })
}

mod location;
pub(super) use location::step_line_path;

pub(super) fn finding(
    file: &str,
    line: usize,
    message: String,
    target: Option<&str>,
) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.into(),
        file: file.into(),
        line,
        message,
        import: None,
        target: target.map(str::to_owned),
    }
}
