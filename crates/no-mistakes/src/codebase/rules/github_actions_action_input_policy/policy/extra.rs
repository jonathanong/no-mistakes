use super::super::scalar::{input_scalar, InputScalar};
use super::super::{ActionSpec, RULE_ID};
use super::{Family, InputFault};
use anyhow::{bail, Result};
use serde_yaml::{Mapping, Value};

pub(super) fn merge(family: &mut Family, action: &ActionSpec) {
    if let Some(keys) = &action.allowed_inputs {
        push_unique(family.allowed.get_or_insert_with(Vec::new), keys);
    }
    push_unique(&mut family.require_present, &action.required_present);
}

fn push_unique(into: &mut Vec<String>, keys: &[String]) {
    for key in keys
        .iter()
        .map(|key| key.trim())
        .filter(|key| !key.is_empty())
    {
        if !into.iter().any(|existing| existing == key) {
            into.push(key.to_string());
        }
    }
}

pub(super) fn validate(family: &Family) -> Result<()> {
    let name = &family.name;
    for key in &family.require_present {
        if family.ban.contains(key) || family.require.contains_key(key) {
            bail!("{RULE_ID}: action `{name}` requiredPresent `{key}` is also banned or required");
        }
    }
    let Some(allowed) = &family.allowed else {
        return Ok(());
    };
    for key in &family.ban {
        if allowed.contains(key) {
            bail!("{RULE_ID}: action `{name}` both allows and bans `{key}`");
        }
    }
    for key in family.require.keys().chain(&family.require_present) {
        if !allowed.contains(key) {
            bail!("{RULE_ID}: action `{name}` requires `{key}` but allowedInputs omits it");
        }
    }
    Ok(())
}

pub(super) fn judge(with: Option<&Mapping>, family: &Family, faults: &mut Vec<InputFault>) {
    for key in &family.require_present {
        if matches!(input_scalar(with, key), InputScalar::Absent) {
            faults.push(InputFault::NotPresent { key: key.clone() });
        }
    }
    let (Some(allowed), Some(with)) = (&family.allowed, with) else {
        return;
    };
    for key in with.keys().filter_map(Value::as_str) {
        let banned = family.ban.iter().any(|ban| ban == key);
        if !banned && !allowed.iter().any(|name| name == key) {
            faults.push(InputFault::Unlisted {
                key: key.to_string(),
            });
        }
    }
}
