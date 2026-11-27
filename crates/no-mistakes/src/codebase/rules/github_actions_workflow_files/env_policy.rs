use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::file_diagnostics::{self, InputUncertainty};
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Context, Result};
use rayon::prelude::*;
use serde::Deserialize;
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(in crate::codebase::rules) mod lines;
mod scan;
mod scopes;
pub(in crate::codebase::rules) use scopes::{shape_detail, visit_workflow_scopes, ScopeEvent};

pub const RULE_ID: &str = "github-actions-env-policy";

const DEFAULT_INCLUDE: &[&str] = &[
    ".github/workflows/**/*.yml",
    ".github/workflows/**/*.yaml",
    ".github/actions/**/action.yml",
    ".github/actions/**/action.yaml",
];

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) required: BTreeMap<String, Value>,
    pub(crate) forbidden: Vec<String>,
}

pub(crate) struct Policy {
    pub(crate) required: BTreeMap<String, String>,
    pub(crate) forbidden: Vec<String>,
}

pub(crate) fn scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        Value::Null => Some(String::new()),
        _ => None,
    }
}

pub(in crate::codebase::rules) fn compile(opts: &Options) -> Result<Policy> {
    use crate::codebase::rules::validate_options::checks::{nonblank, option};
    crate::codebase::rules::file_matching::compile_globs(opts.include.iter().map(String::as_str))
        .context("options.include")?;
    for (index, name) in opts.forbidden.iter().enumerate() {
        nonblank(RULE_ID, option("forbidden").index(index), name)?;
    }
    let mut required = BTreeMap::new();
    for (name, value) in &opts.required {
        nonblank(RULE_ID, option("required").key(name), name)?;
        let Some(text) = scalar_text(value) else {
            bail!("{RULE_ID}: required.{name} must be a string, number, or boolean");
        };
        if opts.forbidden.contains(name) {
            bail!("{RULE_ID}: {name} cannot be both required and forbidden");
        }
        required.insert(name.clone(), text);
    }
    Ok(Policy {
        required,
        forbidden: opts.forbidden.clone(),
    })
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources, false)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    diagnostics_with_files_and_sources(root, config, all_files, sources, defer_suppression)
        .map(|output| output.findings)
}

pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut output = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.try_rule_options()?;
        let policy = compile(&opts)?;
        let unanalyzable = config.effective_rule_analysis(rule)?.unanalyzable;
        if policy.required.is_empty() && policy.forbidden.is_empty() {
            continue;
        }
        let roots = super::super::target_roots(root, config, rule);
        let skip = super::super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| super::super::file_allowed_by_roots_and_skip(root, &skip, path, &roots))
            .cloned()
            .collect();
        let files = super::super::path_filter::filter_rule_files(root, config, rule, &files)?;
        let include: Vec<String> = if opts.include.is_empty() {
            DEFAULT_INCLUDE.iter().map(|p| (*p).to_string()).collect()
        } else {
            opts.include.clone()
        };
        let files = super::super::matching_files(root, &include, &files, &roots)?;
        let chunks: Vec<_> = files
            .par_iter()
            .map(|path| {
                scan::check_file(
                    root,
                    path,
                    &policy,
                    sources,
                    defer_suppression,
                    unanalyzable,
                )
            })
            .collect();
        let mut input_failures = Vec::new();
        for chunk in chunks {
            match chunk {
                Ok(diagnostics) => output.append(diagnostics),
                Err(finding) => input_failures.push(*finding),
            }
        }
        output.append(file_diagnostics::application(
            root,
            sources,
            config,
            rule,
            input_failures,
            defer_suppression,
            InputUncertainty::ReadOrParse,
        )?);
    }
    Ok(file_diagnostics::sorted(output))
}

#[cfg(test)]
mod tests;
