use super::RuleFinding;
use super::RULE_ID;
use serde_yaml::Value;

pub(super) fn timeout_minutes(value: &Value) -> Option<u64> {
    match value {
        Value::Number(number) => number
            .as_u64()
            .or_else(|| number.as_i64().and_then(|value| u64::try_from(value).ok())),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
}

pub(super) fn step_label(step: &serde_yaml::Mapping, index: usize) -> String {
    mapping_string(step, "name")
        .or_else(|| mapping_string(step, "id"))
        .unwrap_or_else(|| format!("step #{}", index + 1))
}

pub(super) fn mapping_string(step: &serde_yaml::Mapping, key: &str) -> Option<String> {
    step.get(Value::String(key.to_string()))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub(super) fn key_line(source: &str, key: &str) -> usize {
    source
        .lines()
        .position(|line| {
            let trimmed = line.split('#').next().unwrap_or(line).trim();
            trimmed == format!("{key}:") || trimmed.starts_with(&format!("{key}:"))
        })
        .map(|index| index + 1)
        .unwrap_or(1)
}

pub(super) fn finding(file: &str, line: usize, message: String, target: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message,
        import: None,
        target: Some(target.to_string()),
    }
}
