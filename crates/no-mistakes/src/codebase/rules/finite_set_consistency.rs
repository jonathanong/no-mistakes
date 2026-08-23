mod comments;
mod comparison;
mod extract;
mod extraction_completeness;
mod literals;
mod markdown;
mod object;
mod scan;
mod ts_array;
mod ts_union;
mod yaml;

use super::RuleFinding;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use scan::{scan, ScanInput};
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "finite-set-consistency";
pub(crate) const TS_CALL_FIRST_STRING_ARGUMENT: &str = "ts-call-first-string-argument";

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
    pub(crate) key: String,
    pub(crate) min_size: usize,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Comparison {
    pub(crate) left: String,
    pub(crate) right: String,
    pub(crate) mode: String,
    pub(crate) message: Option<String>,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    check_with_files_sources_and_facts(root, config, all_files, sources, None)
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: Option<&dyn TsFactLookup>,
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options()?;
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan(ScanInput {
                root,
                config,
                rule,
                opts: &opts,
                files: &files,
                target_roots: &target_roots,
                sources,
                facts,
            })
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

/// TypeScript files that must have function-call facts prepared for this rule.
///
/// Request boundaries use this before collection so the finite-set rule can
/// borrow the shared fact map instead of parsing its configured files itself.
#[doc(hidden)]
pub fn required_call_site_fact_files(
    root: &Path,
    config: &NoMistakesConfig,
) -> Result<Vec<PathBuf>> {
    let paths: Result<Vec<Vec<PathBuf>>> = config
        .rule_applications(RULE_ID)
        .into_iter()
        .map(|rule| {
            let opts: Options = rule.rule_options()?;
            let target_roots = super::target_roots(root, config, rule);
            Ok(opts
                .sets
                .into_iter()
                .filter(|spec| spec.kind == TS_CALL_FIRST_STRING_ARGUMENT)
                .flat_map(move |spec| extract::resolve_spec_files(root, &spec.file, &target_roots))
                .collect())
        })
        .collect();
    let mut paths = paths?.into_iter().flatten().collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    Ok(paths)
}

pub(super) fn finding(
    file: &str,
    comparison: &Comparison,
    fallback: String,
    value: &str,
) -> RuleFinding {
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
#[path = "finite_set_consistency/tests/config_sets.rs"]
mod config_set_tests;
#[cfg(test)]
#[path = "finite_set_consistency/tests/min_size.rs"]
mod min_size_tests;
#[cfg(test)]
#[path = "finite_set_consistency/tests/object_comment.rs"]
mod object_comment_tests;
#[cfg(test)]
#[path = "finite_set_consistency/tests/object_property.rs"]
mod object_property_tests;
#[cfg(test)]
#[path = "finite_set_consistency/tests/object.rs"]
mod object_tests;
#[cfg(test)]
#[path = "finite_set_consistency/tests/path_regex.rs"]
mod path_regex_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
#[path = "finite_set_consistency/tests/ts_array.rs"]
mod ts_array_tests;
