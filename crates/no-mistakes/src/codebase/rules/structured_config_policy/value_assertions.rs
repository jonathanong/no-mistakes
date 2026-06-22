use super::{value_at_key, AssertionKind, ValueAssertion, RULE_ID};
use crate::codebase::rules::RuleFinding;
use anyhow::Result;
use globset::Glob;
use serde_yaml::Value;

pub(super) fn assert_value(
    rel: &str,
    root: &Value,
    assertion: &ValueAssertion,
) -> Result<Vec<RuleFinding>> {
    let Some(kind) = assertion.kind else {
        return Ok(Vec::new());
    };
    if assertion.key.is_empty() {
        return Ok(Vec::new());
    }
    let selected = values_at_selector(root, &assertion.key);
    let mut findings = Vec::new();
    if selected.has_missing {
        findings.push(assertion_finding(
            rel,
            assertion,
            format!(
                "{rel}: config value `{}` required by assertion is missing",
                assertion.key
            ),
        ));
    }
    if selected.values.is_empty() {
        return Ok(findings);
    }
    for value in selected.values {
        let violation = match kind {
            AssertionKind::Boolean => {
                (!matches!(value, Value::Bool(_))).then(|| "must be a strict boolean".to_string())
            }
            AssertionKind::RecordOfBoolean => (!record_of_boolean(value))
                .then(|| "must be an object with strict boolean values".to_string()),
            AssertionKind::PositiveNumber => {
                (!positive_number(value)).then(|| "must be a positive number".to_string())
            }
            AssertionKind::StringArray => {
                (!string_array(value)).then(|| "must be an array of strings".to_string())
            }
            AssertionKind::StringPrefix => string_value(value)
                .filter(|text| text.starts_with(&assertion.prefix))
                .is_none()
                .then(|| format!("must be a string starting with `{}`", assertion.prefix)),
            AssertionKind::StringGlob => string_glob_violation(value, assertion),
            AssertionKind::NotSingleFile => string_value(value)
                .filter(|text| !single_file_entry(text))
                .is_none()
                .then(|| "must not be a single-file entry".to_string()),
            AssertionKind::Equals => assertion
                .value
                .as_ref()
                .is_some_and(|expected| expected != value)
                .then(|| "must equal the configured value".to_string()),
            AssertionKind::ObjectShape => object_shape_violation(value, assertion),
        };
        if let Some(reason) = violation {
            findings.push(assertion_finding(
                rel,
                assertion,
                format!("{rel}: config value `{}` {reason}", assertion.key),
            ));
        }
    }
    Ok(findings)
}

fn string_glob_violation(value: &Value, assertion: &ValueAssertion) -> Option<String> {
    match Glob::new(&assertion.glob) {
        Ok(glob) => {
            let matcher = glob.compile_matcher();
            string_value(value)
                .filter(|text| matcher.is_match(text))
                .is_none()
                .then(|| format!("must match glob `{}`", assertion.glob))
        }
        Err(_) => Some(format!("uses invalid glob `{}`", assertion.glob)),
    }
}

struct SelectorValues<'a> {
    values: Vec<&'a Value>,
    has_missing: bool,
}

fn values_at_selector<'a>(value: &'a Value, selector: &str) -> SelectorValues<'a> {
    let mut current = vec![Some(value)];
    let mut has_missing = false;
    for part in selector.split('.') {
        let mut next = Vec::new();
        if part == "[]" {
            for value in current {
                match value {
                    Some(Value::Sequence(items)) => {
                        next.extend(items.iter().map(Some));
                    }
                    Some(_) | None => {
                        has_missing = true;
                        next.push(None);
                    }
                }
            }
        } else if let Ok(index) = part.parse::<usize>() {
            for value in current {
                match value {
                    Some(Value::Sequence(items)) => match items.get(index) {
                        Some(item) => next.push(Some(item)),
                        None => {
                            has_missing = true;
                            next.push(None);
                        }
                    },
                    Some(_) | None => {
                        has_missing = true;
                        next.push(None);
                    }
                }
            }
        } else {
            for value in current {
                match value.and_then(|value| value.get(part)) {
                    Some(child) => next.push(Some(child)),
                    None => {
                        has_missing = true;
                        next.push(None);
                    }
                }
            }
        }
        current = next;
    }
    SelectorValues {
        values: current.into_iter().flatten().collect(),
        has_missing,
    }
}

fn positive_number(value: &Value) -> bool {
    value.as_f64().is_some_and(|number| number > 0.0)
}

fn string_array(value: &Value) -> bool {
    matches!(value, Value::Sequence(items) if items.iter().all(|item| matches!(item, Value::String(_))))
}

fn record_of_boolean(value: &Value) -> bool {
    matches!(value, Value::Mapping(items) if items.values().all(|item| matches!(item, Value::Bool(_))))
}

fn string_value(value: &Value) -> Option<&str> {
    match value {
        Value::String(text) => Some(text),
        _ => None,
    }
}

fn single_file_entry(value: &str) -> bool {
    !value
        .chars()
        .any(|ch| matches!(ch, '*' | '?' | '[' | ']' | '{' | '}'))
}

fn object_shape_violation(value: &Value, assertion: &ValueAssertion) -> Option<String> {
    if !matches!(value, Value::Mapping(_)) {
        return Some("must be an object".to_string());
    }
    for key in &assertion.required_keys {
        if value_at_key(value, key).is_none() {
            return Some(format!("must contain object key `{key}`"));
        }
    }
    for (key, expected) in &assertion.required_values {
        match value_at_key(value, key) {
            Some(actual) if actual == expected => {}
            Some(_) => return Some(format!("must contain `{key}` with the configured value")),
            None => return Some(format!("must contain object key `{key}`")),
        }
    }
    None
}

fn assertion_finding(rel: &str, assertion: &ValueAssertion, fallback: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line: 1,
        message: assertion.message.clone().unwrap_or(fallback),
        import: None,
        target: Some(assertion.key.clone()),
    }
}
