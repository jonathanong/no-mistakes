use super::scalar::{display_canonical, input_scalar, InputScalar};
use super::Options;
use anyhow::Result;
use serde_yaml::Mapping;
use std::collections::BTreeMap;
use std::path::PathBuf;

mod extra;
mod validation;

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
    allowed: Option<Vec<String>>,
    require_present: Vec<String>,
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
    Unlisted {
        key: String,
    },
    NotPresent {
        key: String,
    },
}

pub(super) fn compile(opts: Options) -> Result<Compiled> {
    let mut families: BTreeMap<String, Family> = BTreeMap::new();
    for action in opts.actions {
        let name = validation::concrete_family(&action)?;
        let family = families.entry(name.clone()).or_insert_with(|| Family {
            name,
            require: BTreeMap::new(),
            ban: Vec::new(),
            allowed: None,
            require_present: Vec::new(),
        });
        validation::merge_action(family, &action)?;
        extra::merge(family, &action);
    }
    for family in families.values() {
        extra::validate(family)?;
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
        Some(canonicalize_family(family))
    }
}

fn canonicalize_family(family: &str) -> String {
    if family.starts_with(['.', '/']) || family.contains("://") {
        return family.to_string();
    }
    let Some((owner, rest)) = family.split_once('/') else {
        return family.to_string();
    };
    let (repo, subpath) = rest.split_once('/').unwrap_or((rest, ""));
    let owner = owner.to_ascii_lowercase();
    let repo = repo.to_ascii_lowercase();
    if subpath.is_empty() {
        format!("{owner}/{repo}")
    } else {
        format!("{owner}/{repo}/{subpath}")
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
    extra::judge(with, family, &mut faults);
    faults
}
