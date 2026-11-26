use super::{AssertionKind, PolicyWhen, ValueAssertion, value_at_key};
use crate::codebase::structured_value::PartialYaml;
use serde_yaml::Value;

pub(super) fn key<'a>(
    mut value: &'a Value,
    path: &str,
    partial: Option<&PartialYaml>,
) -> Option<Option<&'a Value>> {
    let Some(partial) = partial else {
        return Some(value_at_key(value, path));
    };
    for part in path.split('.') {
        match child(partial, value, part)? {
            Some(selected) => value = selected,
            None => return Some(None),
        }
    }
    Some(Some(value))
}

pub(super) fn conditions_known(partial: &PartialYaml, conditions: &[PolicyWhen]) -> bool {
    conditions
        .iter()
        .all(|condition| key(partial.value(), &condition.key, Some(partial)).is_some())
}

fn child<'a>(partial: &PartialYaml, value: &'a Value, name: &str) -> Option<Option<&'a Value>> {
    let selected = value.get(name);
    if selected.is_none() && partial.mapping_unknown(value) {
        return None;
    }
    Some(selected)
}

pub(super) fn assertion_known(partial: &PartialYaml, assertion: &ValueAssertion) -> bool {
    if assertion.kind == Some(AssertionKind::AncestorOverrideSubset) {
        return super::ancestor_override_subset::inputs_known(partial, assertion);
    }
    let shape_only = matches!(
        assertion.kind,
        Some(
            AssertionKind::Boolean
                | AssertionKind::PositiveNumber
                | AssertionKind::StringArray
                | AssertionKind::StringPrefix
                | AssertionKind::StringGlob
                | AssertionKind::NotSingleFile
        )
    );
    walk_selector(partial, &assertion.key, shape_only)
}

pub(super) fn selector_known(partial: &PartialYaml, selector: &str) -> bool {
    walk_selector(partial, selector, false)
}

fn walk_selector(partial: &PartialYaml, selector: &str, shape_only: bool) -> bool {
    let path: Vec<_> = selector
        .split('.')
        .filter(|part| !part.is_empty())
        .collect();
    walk_known(partial, partial.value(), &path, shape_only)
}

fn walk_known(partial: &PartialYaml, value: &Value, path: &[&str], shape_only: bool) -> bool {
    let Some((part, rest)) = path.split_first() else {
        return shape_only || !partial.contains_unknown(value);
    };
    if *part == "[]" {
        return match value {
            Value::Sequence(items) => items
                .iter()
                .all(|item| walk_known(partial, item, rest, shape_only)),
            _ => true,
        };
    }
    if let Ok(index) = part.parse::<usize>() {
        return match value {
            Value::Sequence(items) => items
                .get(index)
                .is_none_or(|item| walk_known(partial, item, rest, shape_only)),
            _ => true,
        };
    }
    child(partial, value, part)
        .is_some_and(|item| item.is_none_or(|item| walk_known(partial, item, rest, shape_only)))
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
