use super::super::scalar::scalar_text;
use super::super::{ActionSpec, RULE_ID};
use super::{family_of, Family};
use crate::codebase::check_outcome::CheckInputFailure;
use anyhow::{bail, Result};

pub(super) fn invalid(
    option: &str,
    value: impl std::fmt::Debug,
    allowed: impl std::fmt::Display,
) -> CheckInputFailure {
    CheckInputFailure::invalid_rule_option(RULE_ID, option, value, allowed)
}

pub(super) fn concrete_family(action: &ActionSpec) -> Result<String> {
    family_of(&action.uses)
        .ok_or_else(|| invalid("actions[].uses", &action.uses, "a concrete action family").into())
}

pub(super) fn merge_action(family: &mut Family, action: &ActionSpec) -> Result<()> {
    let name = &family.name;
    for key in &action.ban {
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        if family.require.contains_key(key) {
            bail!(invalid(
                "actions[].ban",
                key,
                format_args!("an input key absent from require for action `{name}`")
            ));
        }
        if !family.ban.iter().any(|existing| existing == key) {
            family.ban.push(key.to_string());
        }
    }
    for (raw_key, value) in &action.require {
        let key = raw_key.trim();
        if key.is_empty() {
            bail!(invalid(
                "actions[].require",
                raw_key,
                format_args!("a non-empty input key for action `{name}`")
            ));
        }
        if family.ban.iter().any(|banned| banned == key) {
            bail!(invalid(
                "actions[].require",
                key,
                format_args!("an input key absent from ban for action `{name}`")
            ));
        }
        let Some(text) = scalar_text(value) else {
            bail!(invalid(
                &format!("actions[].require.{key}"),
                value,
                format_args!("a string, number, or boolean for action `{name}`")
            ));
        };
        if let Some(existing) = family.require.get(key) {
            if existing != &text {
                bail!(invalid(
                    &format!("actions[].require.{key}"),
                    value,
                    format_args!(
                        "the configured value {} for action `{name}`",
                        super::display_canonical(existing)
                    )
                ));
            }
            continue;
        }
        family.require.insert(key.to_string(), text);
    }
    Ok(())
}
