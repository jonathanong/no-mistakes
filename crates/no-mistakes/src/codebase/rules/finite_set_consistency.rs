mod comments;
mod comparison;
mod effort;
mod extract;
mod extraction_completeness;
use effort::diagnostics_with_lookup;
mod literals;
mod locate;
mod low;
mod markdown;
mod object;
mod options;
mod scan;
mod ts_array;
mod ts_union;
mod yaml;

use super::RuleFinding;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
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
    pub(crate) strip_prefix: String,
    pub(crate) exclude_prefix: String,
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

// Legacy filesystem APIs return findings only. Check reports use the public
// diagnostics entry point below to retain non-failing advisories separately.
pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: Option<&dyn TsFactLookup>,
) -> Result<Vec<RuleFinding>> {
    diagnostics_with_lookup(root, config, all_files, sources, facts, true)
        .map(|diagnostics| diagnostics.findings)
}

/// Evaluate finite sets with each application's independent effort and residual policy.
/// Returns findings and advisories; legacy filesystem Vec adapters return findings only.
#[doc(hidden)]
pub fn diagnostics_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: Option<&crate::codebase::check_facts::CheckFactMap>,
    defer_suppression: bool,
) -> Result<super::analysis_policy::AnalysisDiagnostics> {
    diagnostics_with_lookup(
        root,
        config,
        all_files,
        sources,
        facts.map(|facts| facts as &dyn TsFactLookup),
        defer_suppression,
    )
}

/// TypeScript files that must have function-call facts prepared for this rule.
mod demand;
pub(crate) use demand::call_site_fact_files_from_inventory;
pub use demand::{required_call_site_fact_files, try_required_call_site_fact_files};

fn finding(
    set: &extract::ExtractedSet,
    comparison: &Comparison,
    fallback: String,
    value: &str,
) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: set.file.clone(),
        line: set.line,
        message: comparison.message.clone().unwrap_or(fallback),
        import: None,
        target: Some(value.to_string()),
    }
}

#[cfg(test)]
#[path = "finite_set_consistency/tests/config_sets.rs"]
mod config_set_tests;
#[cfg(test)]
#[path = "finite_set_consistency/tests/locate.rs"]
mod locate_tests;
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

#[cfg(test)]
#[path = "finite_set_consistency/low/tests.rs"]
mod low_tests;

#[cfg(test)]
mod test_support;
