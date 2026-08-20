use super::super::{value_at_key, AssertionKind, ValueAssertion};
use globset::Glob;
use serde_yaml::Value;

pub(super) fn kind_violation(
    value: &Value,
    assertion: &ValueAssertion,
    kind: AssertionKind,
) -> Option<String> {
    match kind {
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
        AssertionKind::EqualsFile => None,
        AssertionKind::ObjectShape => object_shape_violation(value, assertion),
    }
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
    let stripped = value.strip_prefix("**/").unwrap_or(value);
    !stripped
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
    for key in &assertion.forbidden_keys {
        if value_at_key(value, key).is_some() {
            return Some(format!("must not contain object key `{key}`"));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codebase::rules::structured_config_policy::{AssertionKind, ValueAssertion};

    #[test]
    fn equals_file_is_not_evaluated_per_value() {
        let assertion = ValueAssertion {
            kind: Some(AssertionKind::EqualsFile),
            ..Default::default()
        };
        assert!(kind_violation(&Value::Null, &assertion, AssertionKind::EqualsFile).is_none());
    }
}
