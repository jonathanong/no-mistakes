use super::{selector_file, GITHUB_MAX_MINUTES, RULE_ID};
use crate::config::v2::Unanalyzable;
use anyhow::{bail, ensure, Result};
use serde::Deserialize;
use serde_yaml::Mapping;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(in crate::codebase::rules) struct Options {
    pub(super) include: Vec<String>,
    pub(super) max_minutes: Option<u64>,
    pub(super) allow: Vec<AllowEntry>,
    pub(super) action_caps: Vec<ActionCapEntry>,
    pub(super) only_configured_actions: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct AllowEntry {
    pub(super) step: String,
    pub(super) reason: String,
    pub(super) max_minutes: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct ActionCapEntry {
    pub(super) uses: String,
    pub(super) max_minutes: u64,
}

pub(super) enum UsesMatch {
    None,
    Matched { family: String, cap: u64 },
    Dynamic { raw: String },
}

pub(super) struct CompiledOptions {
    pub(super) max_minutes: u64,
    pub(super) allow: BTreeMap<String, AllowEntry>,
    pub(super) allow_by_file: BTreeMap<String, Vec<String>>,
    pub(super) duplicate_allow: BTreeSet<String>,
    pub(super) duplicate_by_file: BTreeMap<String, Vec<String>>,
    pub(super) action_caps: BTreeMap<String, u64>,
    pub(super) min_action_cap: Option<u64>,
    pub(super) only_configured_actions: bool,
    pub(super) unanalyzable: Unanalyzable,
    pub(super) defer_suppression: bool,
}

impl CompiledOptions {
    pub(super) fn match_action_cap(&self, step: &Mapping) -> UsesMatch {
        if self.action_caps.is_empty() {
            return UsesMatch::None;
        }
        let Some(raw) = step.get("uses") else {
            return UsesMatch::None;
        };
        let Some(uses) = raw.as_str() else {
            return UsesMatch::Dynamic {
                raw: "non-string".to_string(),
            };
        };
        let trimmed = uses.trim();
        if trimmed.contains("${{") {
            return UsesMatch::Dynamic {
                raw: trimmed.to_string(),
            };
        }
        let Some(family) = family_of(trimmed) else {
            return UsesMatch::None;
        };
        self.action_caps
            .get(&family)
            .copied()
            .map_or(UsesMatch::None, |cap| UsesMatch::Matched { family, cap })
    }
}

pub(super) fn compile_options(
    opts: Options,
    unanalyzable: Unanalyzable,
    defer_suppression: bool,
) -> Result<CompiledOptions> {
    let mut action_caps = BTreeMap::new();
    for (index, entry) in opts.action_caps.into_iter().enumerate() {
        let Some(family) = family_of(&entry.uses) else {
            bail!(
                "{RULE_ID}: options.actionCaps[{index}].uses `{}` must name a concrete action family",
                entry.uses.trim()
            );
        };
        ensure!(
            entry.max_minutes >= 1 && entry.max_minutes <= GITHUB_MAX_MINUTES,
            "{RULE_ID}: options.actionCaps[{index}].maxMinutes must be between 1 and {GITHUB_MAX_MINUTES}"
        );
        ensure!(
            action_caps.insert(family.clone(), entry.max_minutes).is_none(),
            "{RULE_ID}: options.actionCaps[{index}] duplicates action family `{family}`"
        );
    }
    if opts.only_configured_actions {
        ensure!(
            !action_caps.is_empty(),
            "{RULE_ID}: options.onlyConfiguredActions requires at least one entry in options.actionCaps"
        );
    }
    let min_action_cap = action_caps.values().copied().min();
    let mut allow = BTreeMap::new();
    let mut duplicate_allow = BTreeSet::new();
    for entry in opts.allow {
        let key = entry.step.clone();
        if allow.insert(key.clone(), entry).is_some() {
            duplicate_allow.insert(key);
        }
    }
    let allow_by_file = group_by_file(allow.keys());
    let duplicate_by_file = group_by_file(duplicate_allow.iter());
    Ok(CompiledOptions {
        max_minutes: opts
            .max_minutes
            .unwrap_or(GITHUB_MAX_MINUTES)
            .min(GITHUB_MAX_MINUTES),
        allow,
        allow_by_file,
        duplicate_allow,
        duplicate_by_file,
        action_caps,
        min_action_cap,
        only_configured_actions: opts.only_configured_actions,
        unanalyzable,
        defer_suppression,
    })
}

fn group_by_file<'a>(keys: impl Iterator<Item = &'a String>) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for key in keys {
        if let Some(file) = selector_file(key) {
            out.entry(file.to_string()).or_default().push(key.clone());
        }
    }
    out
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
    if family.is_empty() || family.contains(['*', '?', '[']) {
        return None;
    }
    Some(canonicalize_family(family))
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
