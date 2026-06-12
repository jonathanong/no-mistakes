mod extract;
mod object;

use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use extract::extract_set;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "finite-set-consistency";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) sets: Vec<SetSpec>,
    pub(crate) comparisons: Vec<Comparison>,
}

#[derive(Clone, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct SetSpec {
    pub(crate) name: String,
    pub(crate) file: String,
    pub(crate) kind: String,
    pub(crate) target: String,
    pub(crate) property: String,
    pub(crate) pattern: String,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Comparison {
    pub(crate) left: String,
    pub(crate) right: String,
    pub(crate) message: Option<String>,
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
    let mut sets = BTreeMap::new();
    for spec in &opts.sets {
        if spec.name.is_empty() {
            continue;
        }
        sets.insert(spec.name.clone(), extract_set(root, spec, files)?);
    }

    let mut findings = Vec::new();
    for comparison in &opts.comparisons {
        let (Some(left), Some(right)) = (sets.get(&comparison.left), sets.get(&comparison.right))
        else {
            continue;
        };
        for value in left.values.difference(&right.values) {
            findings.push(finding(
                &right.file,
                comparison,
                format!(
                    "{} contains `{}` but {} does not",
                    comparison.left, value, comparison.right
                ),
                value,
            ));
        }
        for value in right.values.difference(&left.values) {
            findings.push(finding(
                &left.file,
                comparison,
                format!(
                    "{} contains `{}` but {} does not",
                    comparison.right, value, comparison.left
                ),
                value,
            ));
        }
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

fn finding(file: &str, comparison: &Comparison, fallback: String, value: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message: comparison.message.clone().unwrap_or(fallback),
        import: None,
        target: Some(value.to_string()),
    }
}

#[cfg(test)]
mod tests;
