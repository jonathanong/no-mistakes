use super::RULE_ID;
use crate::codebase::rules::validate_options::{
    format_invalid_option, indexed_rule_applications, push_application_errors,
};
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct Options {
    pub include: Vec<String>,
    pub caches: Vec<CacheKeyShape>,
    pub setup: Vec<SetupDeadline>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CacheKeyShape {
    pub uses: String,
    pub path: String,
    pub key_pattern: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SetupDeadline {
    pub uses: String,
    pub max_minutes: u64,
}

pub(super) struct Compiled {
    pub include: Vec<String>,
    pub caches: BTreeMap<String, BTreeMap<String, Regex>>,
    pub setup: BTreeMap<String, u64>,
}

pub(super) fn compile(options: Options) -> Result<Compiled> {
    if options.caches.is_empty() && options.setup.is_empty() {
        bail!(
            "{}",
            invalid(
                "caches/setup",
                "both empty",
                "at least one cache or setup policy"
            )
        );
    }
    let include = if options.include.is_empty() {
        [
            ".github/workflows/*.yml",
            ".github/workflows/*.yaml",
            ".github/actions/**/action.yml",
            ".github/actions/**/action.yaml",
        ]
        .map(str::to_owned)
        .to_vec()
    } else {
        options.include
    };
    for (index, pattern) in include.iter().enumerate() {
        if pattern.trim().is_empty() || globset::Glob::new(pattern).is_err() {
            bail!(
                "{}",
                invalid(&format!("include[{index}]"), pattern, "a nonempty glob")
            );
        }
    }
    let mut caches = BTreeMap::<String, BTreeMap<String, Regex>>::new();
    for (index, cache) in options.caches.into_iter().enumerate() {
        let field = format!("caches[{index}]");
        let uses = option_family(&field, &cache.uses)?;
        let path = normalized_path(&cache.path);
        if path.is_empty() || path.contains(['\n', '\r']) || path.contains("${{") {
            bail!(
                "{}",
                invalid(
                    &format!("{field}.path"),
                    &cache.path,
                    "one literal cache path"
                )
            );
        }
        let entries = caches.entry(uses).or_default();
        if entries.contains_key(path) {
            bail!(
                "{}",
                invalid(
                    &format!("{field}.path"),
                    &cache.path,
                    "a unique path for this action"
                )
            );
        }
        let pattern = Regex::new(&format!(r"\A(?:{})\z", cache.key_pattern))
            .ok()
            .filter(|_| !cache.key_pattern.trim().is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "{}",
                    invalid(
                        &format!("{field}.keyPattern"),
                        &cache.key_pattern,
                        "a nonempty Rust regular expression"
                    )
                )
            })?;
        entries.insert(path.to_owned(), pattern);
    }
    let mut setup = BTreeMap::new();
    for (index, step) in options.setup.into_iter().enumerate() {
        let field = format!("setup[{index}]");
        let uses = option_family(&field, &step.uses)?;
        if !(1..=360).contains(&step.max_minutes) {
            bail!(
                "{}",
                invalid(
                    &format!("{field}.maxMinutes"),
                    step.max_minutes,
                    "an integer from 1 through 360"
                )
            );
        }
        if setup.insert(uses, step.max_minutes).is_some() {
            bail!(
                "{}",
                invalid(
                    &format!("{field}.uses"),
                    &step.uses,
                    "a unique action family"
                )
            );
        }
    }
    Ok(Compiled {
        include,
        caches,
        setup,
    })
}

fn invalid(field: &str, value: impl std::fmt::Debug, allowed: &str) -> String {
    format_invalid_option(RULE_ID, field, value, allowed)
}

fn option_family(field: &str, uses: &str) -> Result<String> {
    let error = || {
        anyhow::anyhow!(
            "{}",
            invalid(
                &format!("{field}.uses"),
                uses,
                "a literal action family without an @ref"
            )
        )
    };
    if uses.contains('@') || uses.chars().any(char::is_whitespace) {
        return Err(error());
    }
    super::super::github_actions_action_input_policy::policy::family_of(uses).ok_or_else(error)
}

pub(super) fn normalized_path(path: &str) -> &str {
    let path = path.trim();
    path.strip_prefix("./")
        .unwrap_or(path)
        .trim_end_matches('/')
}

pub(crate) fn collect_config_errors(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    output: &mut Vec<String>,
) {
    for (index, rule) in indexed_rule_applications(config, RULE_ID) {
        if let Err(error) = rule.try_rule_options::<Options>().and_then(compile) {
            push_application_errors(
                root,
                config,
                index,
                config_path,
                BTreeSet::from([error.to_string()]),
                output,
            );
        }
    }
}
