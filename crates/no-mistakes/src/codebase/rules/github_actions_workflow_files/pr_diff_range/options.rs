use super::RULE_ID;
use crate::codebase::rules::validate_options::{
    format_invalid_option, indexed_rule_applications, push_application_errors,
};
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Options {
    pub(crate) require_consistent_revision_pairs: bool,
    pub(crate) revision_pairs: Vec<RevisionPairSpec>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RevisionPairSpec {
    pub(crate) base_env: Option<String>,
    pub(crate) head_env: Option<String>,
    pub(crate) base_input: Option<String>,
    pub(crate) head_input: Option<String>,
    pub(crate) base_flag: Option<String>,
    pub(crate) head_flag: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CompiledRevisionPair {
    Env { base: String, head: String },
    Input { base: String, head: String },
    Flag { base: String, head: String },
}

#[derive(Debug, Default, Clone)]
pub(super) struct CompiledOptions {
    pub(super) require_consistent_revision_pairs: bool,
    pub(super) revision_pairs: Vec<CompiledRevisionPair>,
}

impl CompiledOptions {
    pub(super) fn strict_enabled(&self) -> bool {
        self.require_consistent_revision_pairs || !self.revision_pairs.is_empty()
    }
}

pub(super) fn compile(opts: &Options) -> Result<CompiledOptions> {
    let mut pairs = Vec::with_capacity(opts.revision_pairs.len());
    for (index, spec) in opts.revision_pairs.iter().enumerate() {
        pairs.push(compile_pair(index, spec)?);
    }
    Ok(CompiledOptions {
        require_consistent_revision_pairs: opts.require_consistent_revision_pairs,
        revision_pairs: pairs,
    })
}

fn compile_pair(index: usize, spec: &RevisionPairSpec) -> Result<CompiledRevisionPair> {
    let env = spec.base_env.is_some() || spec.head_env.is_some();
    let input = spec.base_input.is_some() || spec.head_input.is_some();
    let flag = spec.base_flag.is_some() || spec.head_flag.is_some();
    let kinds = usize::from(env) + usize::from(input) + usize::from(flag);
    if kinds != 1 {
        bail!(
            "{}",
            format_invalid_option(
                RULE_ID,
                &format!("revisionPairs[{index}]"),
                spec,
                "exactly one matching pair of baseEnv/headEnv, baseInput/headInput, or baseFlag/headFlag",
            )
        );
    }
    if env {
        let (base, head) = require_both(index, "baseEnv", "headEnv", &spec.base_env, &spec.head_env)?;
        if !is_env_name(&base) || !is_env_name(&head) {
            bail!(
                "{}",
                format_invalid_option(
                    RULE_ID,
                    &format!("revisionPairs[{index}]"),
                    (&base, &head),
                    "valid ASCII environment variable names",
                )
            );
        }
        return Ok(CompiledRevisionPair::Env { base, head });
    }
    if input {
        let (base, head) =
            require_both(index, "baseInput", "headInput", &spec.base_input, &spec.head_input)?;
        return Ok(CompiledRevisionPair::Input { base, head });
    }
    let (base, head) =
        require_both(index, "baseFlag", "headFlag", &spec.base_flag, &spec.head_flag)?;
    if !is_flag(&base) || !is_flag(&head) {
        bail!(
            "{}",
            format_invalid_option(
                RULE_ID,
                &format!("revisionPairs[{index}]"),
                (&base, &head),
                "non-empty CLI flags starting with `-` and containing no whitespace",
            )
        );
    }
    Ok(CompiledRevisionPair::Flag { base, head })
}

fn require_both(
    index: usize,
    base_key: &str,
    head_key: &str,
    base: &Option<String>,
    head: &Option<String>,
) -> Result<(String, String)> {
    let Some(base) = base.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        bail!(
            "{}",
            format_invalid_option(
                RULE_ID,
                &format!("revisionPairs[{index}].{base_key}"),
                base,
                "a non-empty string",
            )
        );
    };
    let Some(head) = head.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        bail!(
            "{}",
            format_invalid_option(
                RULE_ID,
                &format!("revisionPairs[{index}].{head_key}"),
                head,
                "a non-empty string",
            )
        );
    };
    if base == head {
        bail!(
            "{}",
            format_invalid_option(
                RULE_ID,
                &format!("revisionPairs[{index}]"),
                base,
                "distinct base and head names",
            )
        );
    }
    Ok((base.to_owned(), head.to_owned()))
}

fn is_env_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn is_flag(flag: &str) -> bool {
    flag.starts_with('-') && flag.len() > 1 && !flag.chars().any(char::is_whitespace)
}

pub(crate) fn collect_config_errors(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    errors: &mut Vec<String>,
) {
    for (rule_index, rule) in indexed_rule_applications(config, RULE_ID) {
        let mut app_errors = BTreeSet::new();
        match rule.try_rule_options::<Options>() {
            Ok(opts) => {
                if let Err(error) = compile(&opts) {
                    app_errors.insert(error.to_string());
                }
            }
            Err(error) => {
                app_errors.insert(error.to_string());
            }
        }
        if !app_errors.is_empty() {
            push_application_errors(root, config, rule_index, config_path, app_errors, errors);
        }
    }
}
