use serde_yaml::{Mapping, Value};

pub(super) enum InputScalar {
    Absent,
    Value(String),
    NonLiteral,
}

pub(super) fn input_scalar(with: Option<&Mapping>, key: &str) -> InputScalar {
    let Some(value) = with.and_then(|with| mapping_get(with, key)) else {
        return InputScalar::Absent;
    };
    match scalar_text(value) {
        Some(text) => InputScalar::Value(text),
        None if value.is_null() => InputScalar::Absent,
        None => InputScalar::NonLiteral,
    }
}

pub(super) fn scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Bool(flag) => Some(flag.to_string()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

pub(super) fn display_canonical(text: &str) -> String {
    if text == "true" || text == "false" || is_plain_number(text) {
        text.to_string()
    } else {
        format!("\"{text}\"")
    }
}

fn mapping_get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(Value::String(key.to_string()))
}

fn is_plain_number(text: &str) -> bool {
    let mut bytes = text.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    let rest_ok = bytes.all(|byte| byte.is_ascii_digit() || byte == b'.');
    rest_ok && (first.is_ascii_digit() || first == b'-')
}
