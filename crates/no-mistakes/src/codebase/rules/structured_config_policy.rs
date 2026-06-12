use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::{Glob, GlobSetBuilder};
use rayon::prelude::*;
use serde::Deserialize;
use serde_yaml::Value;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "structured-config-policy";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) policies: Vec<Policy>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Policy {
    pub(crate) files: Vec<String>,
    pub(crate) required_keys: Vec<String>,
    pub(crate) banned_keys: Vec<String>,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan(root, &opts, &files)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for policy in &opts.policies {
        let matching = matching_files(root, &policy.files, files)?;
        for path in matching {
            let rel = relative_slash_path(root, &path);
            let Ok(source) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(value) = serde_yaml::from_str::<Value>(&source) else {
                continue;
            };
            for key in &policy.required_keys {
                if value_at_key(&value, key).is_none() {
                    findings.push(RuleFinding {
                        rule: RULE_ID.to_string(),
                        file: rel.clone(),
                        line: 1,
                        message: format!("{rel}: required config key `{key}` is missing"),
                        import: None,
                        target: Some(key.clone()),
                    });
                }
            }
            for key in &policy.banned_keys {
                if value_at_key(&value, key).is_some() {
                    findings.push(RuleFinding {
                        rule: RULE_ID.to_string(),
                        file: rel.clone(),
                        line: 1,
                        message: format!("{rel}: banned config key `{key}` is present"),
                        import: None,
                        target: Some(key.clone()),
                    });
                }
            }
        }
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

fn matching_files(root: &Path, patterns: &[String], files: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    let globs = builder.build()?;
    Ok(files
        .iter()
        .filter(|path| globs.is_match(relative_slash_path(root, path)))
        .cloned()
        .collect())
}

fn value_at_key<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    key.split('.')
        .try_fold(value, |current, part| current.get(part))
}

#[cfg(test)]
mod tests;
