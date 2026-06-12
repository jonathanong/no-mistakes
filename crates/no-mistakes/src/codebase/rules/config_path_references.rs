use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::Glob;
use rayon::prelude::*;
use serde::Deserialize;
use serde_yaml::Value;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "config-path-references";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) files: Vec<String>,
    pub(crate) keys: Vec<String>,
    pub(crate) base_dir: BaseDir,
    pub(crate) allow_globs: bool,
}

#[derive(Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum BaseDir {
    #[default]
    ConfigFile,
    Root,
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
    let config_files = super::matching_files(root, &opts.files, files)?;
    let rel_files = files
        .iter()
        .map(|path| relative_slash_path(root, path))
        .collect::<Vec<_>>();
    let mut findings = Vec::new();
    for path in config_files {
        let rel = relative_slash_path(root, &path);
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(value) = serde_yaml::from_str::<Value>(&source) else {
            continue;
        };
        for key in &opts.keys {
            for reference in values_at_key(&value, key) {
                if !reference_exists(root, &path, opts, &reference, &rel_files)? {
                    findings.push(RuleFinding {
                        rule: RULE_ID.to_string(),
                        file: rel.clone(),
                        line: 1,
                        message: format!(
                            "{rel}: config path `{reference}` from `{key}` does not exist"
                        ),
                        import: None,
                        target: Some(reference),
                    });
                }
            }
        }
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

fn values_at_key(value: &Value, key: &str) -> Vec<String> {
    let Some(value) = key
        .split('.')
        .try_fold(value, |current, part| current.get(part))
    else {
        return Vec::new();
    };
    match value {
        Value::String(value) => vec![value.clone()],
        Value::Sequence(values) => values
            .iter()
            .filter_map(|value| value.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

fn reference_exists(
    root: &Path,
    config_file: &Path,
    opts: &Options,
    reference: &str,
    rel_files: &[String],
) -> Result<bool> {
    if opts.allow_globs
        && (reference.contains('*')
            || reference.contains('?')
            || reference.contains('[')
            || reference.contains('{'))
    {
        let pattern = reference_pattern(root, config_file, opts, reference);
        let glob = Glob::new(&pattern)?;
        let matcher = glob.compile_matcher();
        return Ok(rel_files.iter().any(|rel| matcher.is_match(rel)));
    }
    let base = if opts.base_dir == BaseDir::Root {
        root.to_path_buf()
    } else {
        config_file.parent().unwrap_or(root).to_path_buf()
    };
    Ok(base.join(reference).exists())
}

fn reference_pattern(root: &Path, config_file: &Path, opts: &Options, reference: &str) -> String {
    if opts.base_dir == BaseDir::Root {
        return reference.to_string();
    }
    let Some(parent) = config_file.parent() else {
        return reference.to_string();
    };
    let dir = relative_slash_path(root, parent);
    if dir.is_empty() {
        reference.to_string()
    } else {
        format!("{dir}/{reference}")
    }
}

#[cfg(test)]
mod tests;
