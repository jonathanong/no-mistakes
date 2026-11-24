use super::super::scalar::scalar_text;
use super::super::{ActionSpec, RULE_ID};
use super::{family_of, Family};
use crate::codebase::rules::validate_options::format_invalid_option;
use std::collections::BTreeSet;

pub(super) fn invalid(
    option: &str,
    value: impl std::fmt::Debug,
    allowed: impl std::fmt::Display,
) -> String {
    format_invalid_option(RULE_ID, option, value, allowed)
}

pub(super) fn concrete_family(action: &ActionSpec, errors: &mut BTreeSet<String>) -> Option<String> {
    let found = family_of(&action.uses);
    if found.is_none() {
        errors.insert(invalid(
            "actions[].uses",
            &action.uses,
            "a concrete action family",
        ));
    }
    found
}

pub(super) fn merge_action(
    family: &mut Family,
    action: &ActionSpec,
    errors: &mut BTreeSet<String>,
) {
    let name = &family.name;
    for key in &action.ban {
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        if family.require.contains_key(key) {
            errors.insert(invalid(
                "actions[].ban",
                key,
                format_args!("an input key absent from require for action `{name}`"),
            ));
            continue;
        }
        if !family.ban.iter().any(|existing| existing == key) {
            family.ban.push(key.to_string());
        }
    }
    for (raw_key, value) in &action.require {
        let key = raw_key.trim();
        if key.is_empty() {
            errors.insert(invalid(
                "actions[].require",
                raw_key,
                format_args!("a non-empty input key for action `{name}`"),
            ));
            continue;
        }
        let banned = family.ban.iter().any(|banned| banned == key);
        if banned {
            errors.insert(invalid(
                "actions[].require",
                key,
                format_args!("an input key absent from ban for action `{name}`"),
            ));
        }
        let Some(text) = scalar_text(value) else {
            errors.insert(invalid(
                &format!("actions[].require.{key}"),
                value,
                format_args!("a string, number, or boolean for action `{name}`"),
            ));
            continue;
        };
        if banned {
            continue;
        }
        if let Some(existing) = family.require.get(key) {
            if existing != &text {
                errors.insert(invalid(
                    &format!("actions[].require.{key}"),
                    value,
                    format_args!(
                        "the configured value {} for action `{name}`",
                        super::display_canonical(existing)
                    ),
                ));
            }
            continue;
        }
        family.require.insert(key.to_string(), text);
    }
}
