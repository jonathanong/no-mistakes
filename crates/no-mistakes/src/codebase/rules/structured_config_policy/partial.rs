use super::{value_at_key, AssertionKind, PolicyWhen, ValueAssertion};
use crate::codebase::structured_value::contains_merge;
use serde_yaml::Value;

pub(super) fn key<'a>(mut value: &'a Value, path: &str, partial: bool) -> Option<Option<&'a Value>> {
    if !partial {
        return Some(value_at_key(value, path));
    }
    for part in path.split('.') {
        match child(value, part)? {
            Some(selected) => value = selected,
            None => return Some(None),
        }
    }
    Some(Some(value))
}

pub(super) fn conditions_known(value: &Value, conditions: &[PolicyWhen]) -> bool {
    conditions.iter().all(|condition| key(value, &condition.key, true).is_some())
}

fn child<'a>(value: &'a Value, name: &str) -> Option<Option<&'a Value>> {
    let selected = value.get(name);
    if (selected.is_none() || name == "<<") && value.get("<<").is_some() {
        return None;
    }
    Some(selected)
}

pub(super) fn assertion_known(value: &Value, assertion: &ValueAssertion) -> bool {
    if assertion.kind == Some(AssertionKind::AncestorOverrideSubset) {
        return !contains_merge(value);
    }
    let path: Vec<_> = assertion.key.split('.').filter(|part| !part.is_empty()).collect();
    selector_known(value, &path)
}

fn selector_known(value: &Value, path: &[&str]) -> bool {
    let Some((part, rest)) = path.split_first() else {
        return !contains_merge(value);
    };
    if *part == "[]" {
        return value.as_sequence().is_none_or(|items| {
            items.iter().all(|item| selector_known(item, rest))
        });
    }
    if let Ok(index) = part.parse::<usize>() {
        return value.as_sequence().and_then(|items| items.get(index))
            .is_none_or(|item| selector_known(item, rest));
    }
    child(value, part).is_some_and(|item| item.is_none_or(|item| selector_known(item, rest)))
}
