mod literals;
mod scan;

use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use rayon::prelude::*;
use scan::scan_file;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "test-fixture-size-cap";

const SCRIPT_EXTENSIONS: &[&str] = &[".cjs", ".cts", ".js", ".jsx", ".mjs", ".mts", ".ts", ".tsx"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(in crate::codebase::rules) struct Options {
    pub(super) threshold: Option<u64>,
    pub(super) exemptions: Vec<u64>,
    pub(super) each_calls: Vec<String>,
    pub(super) each_table_mode: EachTableMode,
    pub(super) scopes: Scopes,
}

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum EachTableMode {
    #[default]
    PerElement,
    AllElements,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub(super) struct Scopes {
    pub(super) tests: Vec<String>,
    pub(super) helpers: Vec<String>,
}

pub(in crate::codebase::rules) struct Compiled {
    pub(super) threshold: Option<u64>,
    pub(super) exemptions: BTreeSet<u64>,
    pub(super) each_calls: Vec<(String, String)>,
    pub(super) each_table_mode: EachTableMode,
    tests: GlobMatcher,
    helpers: GlobMatcher,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources, false)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    super::selected_files::run_parallel_rule_applications(config, RULE_ID, &|rule| {
        let opts: Options = rule.try_rule_options()?;
        let compiled = compile_options(&opts)?;
        if compiled.threshold.is_none() {
            return Ok(Vec::new());
        }
        let target_roots = super::target_roots(root, config, rule);
        let skip = super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots))
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        let mut findings: Vec<_> = files
            .par_iter()
            .flat_map(|path| scan_file(root, path, &compiled, sources))
            .collect();
        if let Some(message) = rule.message.as_deref() {
            for finding in &mut findings {
                finding.message = format!("{message} {}", finding.message);
            }
        }
        if !defer_suppression {
            super::suppression::suppress_rule_findings_with_sources(root, &mut findings, sources);
        }
        Ok(findings)
    })
}

pub(in crate::codebase::rules) fn compile_options(opts: &Options) -> Result<Compiled> {
    let each_calls = opts
        .each_calls
        .iter()
        .map(|call| split_each_call(call))
        .collect::<Result<Vec<_>>>()?;
    Ok(Compiled {
        threshold: opts.threshold,
        exemptions: opts.exemptions.iter().copied().collect(),
        each_calls,
        each_table_mode: opts.each_table_mode,
        tests: GlobMatcher::new(&opts.scopes.tests, &format!("{RULE_ID} scopes.tests"))?,
        helpers: GlobMatcher::new(&opts.scopes.helpers, &format!("{RULE_ID} scopes.helpers"))?,
    })
}

fn split_each_call(call: &str) -> Result<(String, String)> {
    let Some((object, property)) = call.split_once('.') else {
        bail!("{RULE_ID} eachCalls entry `{call}` must be object.property");
    };
    if !is_ident(object) || !is_ident(property) {
        bail!("{RULE_ID} eachCalls entry `{call}` must be object.property");
    }
    Ok((object.to_string(), property.to_string()))
}

fn is_ident(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first == '$' || first.is_ascii_alphabetic())
        && chars.all(|character| {
            character == '_' || character == '$' || character.is_ascii_alphanumeric()
        })
}

pub(super) fn in_scope(policy: &Compiled, rel: &str) -> bool {
    policy.tests.is_match(rel) || policy.helpers.is_match(rel)
}

pub(super) fn is_script(rel: &str) -> bool {
    SCRIPT_EXTENSIONS.iter().any(|ext| rel.ends_with(ext))
}

pub(super) fn relative_file(root: &Path, path: &Path) -> String {
    relative_slash_path(root, path)
}

#[cfg(test)]
mod tests;
