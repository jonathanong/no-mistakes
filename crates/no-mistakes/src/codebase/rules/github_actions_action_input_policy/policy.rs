use super::scalar::{display_canonical, input_scalar, scalar_text, InputScalar};
use super::{ActionSpec, Options, RULE_ID};
use anyhow::{bail, Result};
use serde_yaml::Mapping;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug)]
pub(super) struct Compiled {
    pub(super) include: Vec<String>,
    pub(super) actions: Vec<Family>,
    pub(super) target_roots: Vec<PathBuf>,
}

#[derive(Debug)]
pub(super) struct Family {
    pub(super) name: String,
    require: BTreeMap<String, String>,
    ban: Vec<String>,
}

pub(super) enum InputFault {
    Missing {
        key: String,
        expected: String,
    },
    Mismatch {
        key: String,
        expected: String,
        got: String,
    },
    Banned {
        key: String,
    },
}

pub(super) fn compile(opts: Options) -> Result<Compiled> {
    let mut families: BTreeMap<String, Family> = BTreeMap::new();
    for action in opts.actions {
        let name = concrete_family(&action)?;
        let family = families.entry(name.clone()).or_insert_with(|| Family {
            name,
            require: BTreeMap::new(),
            ban: Vec::new(),
        });
        merge_action(family, &action)?;
    }
    Ok(Compiled {
        include: opts.include,
        actions: families.into_values().collect(),
        target_roots: Vec::new(),
    })
}

pub(super) fn family_of(uses: &str) -> Option<String> {
    let trimmed = uses.trim().trim_end_matches('/');
    if trimmed.is_empty() || trimmed.contains("${{") {
        return None;
    }
    let family = trimmed
        .split_once('@')
        .map(|(family, _)| family)
        .unwrap_or(trimmed);
    if family.is_empty() {
        None
    } else {
        Some(family.to_string())
    }
}

pub(super) fn matching<'a>(uses: &str, actions: &'a [Family]) -> Option<&'a Family> {
    let family = family_of(uses)?;
    actions.iter().find(|action| action.name == family)
}

pub(super) fn judge(with: Option<&Mapping>, family: &Family) -> Vec<InputFault> {
    let mut faults = Vec::new();
    for (key, expected) in &family.require {
        match input_scalar(with, key) {
            InputScalar::Absent => faults.push(InputFault::Missing {
                key: key.clone(),
                expected: display_canonical(expected),
            }),
            InputScalar::Value(got) if got == *expected => {}
            InputScalar::Value(got) => faults.push(InputFault::Mismatch {
                key: key.clone(),
                expected: display_canonical(expected),
                got: display_canonical(&got),
            }),
            InputScalar::NonLiteral => faults.push(InputFault::Mismatch {
                key: key.clone(),
                expected: display_canonical(expected),
                got: "non-literal".to_string(),
            }),
        }
    }
    for key in &family.ban {
        if !matches!(input_scalar(with, key), InputScalar::Absent) {
            faults.push(InputFault::Banned { key: key.clone() });
        }
    }
    faults
}

fn concrete_family(action: &ActionSpec) -> Result<String> {
    let Some(name) = family_of(&action.uses) else {
        bail!(
            "{RULE_ID}: actions[].uses `{}` must name a concrete action family",
            action.uses.trim()
        );
    };
    Ok(name)
}

fn merge_action(family: &mut Family, action: &ActionSpec) -> Result<()> {
    for key in &action.ban {
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        if family.require.contains_key(key) {
            bail!(
                "{RULE_ID}: action `{}` both requires and bans `{key}`",
                family.name
            );
        }
        if !family.ban.iter().any(|existing| existing == key) {
            family.ban.push(key.to_string());
        }
    }
    for (key, value) in &action.require {
        let key = key.trim();
        if key.is_empty() {
            bail!(
                "{RULE_ID}: action `{}` has an empty require input name",
                family.name
            );
        }
        if family.ban.iter().any(|banned| banned == key) {
            bail!(
                "{RULE_ID}: action `{}` both requires and bans `{key}`",
                family.name
            );
        }
        let Some(text) = scalar_text(value) else {
            bail!(
                "{RULE_ID}: action `{}` require `{key}` must be a string, number, or boolean",
                family.name
            );
        };
        if let Some(existing) = family.require.get(key) {
            if existing != &text {
                bail!(
                    "{RULE_ID}: action `{}` require `{key}` is both {existing} and {text}",
                    family.name
                );
            }
            continue;
        }
        family.require.insert(key.to_string(), text);
    }
    Ok(())
}
