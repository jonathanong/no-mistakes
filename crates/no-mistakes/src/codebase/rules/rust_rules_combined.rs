use super::{
    rust_max_lines_per_file, rust_no_inline_allows, rust_no_inline_tests, RuleFinding,
    RUST_MAX_LINES_PER_FILE, RUST_NO_INLINE_ALLOWS, RUST_NO_INLINE_TESTS,
};
use crate::codebase::ts_source::has_disable_file_comment;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod scan;
mod selection;
use selection::{add_inline_allows_work, add_inline_tests_work, add_max_lines_work};
pub(super) mod standalone;

use super::rust_max_lines_per_file::Options as MaxLinesOptions;

#[derive(Default)]
pub(super) struct RustWork<'a> {
    pub(super) applications: Vec<(&'a crate::config::v2::schema::RuleDef, Option<usize>)>,
    pub(super) max_limits: Vec<usize>,
    pub(super) inline_tests: bool,
    pub(super) inline_allows: bool,
}

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    diagnostics_with_files_and_sources(root, config, all_files, sources, defer_suppression)
        .map(|output| output.findings)
}

#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<super::analysis_policy::AnalysisDiagnostics> {
    diagnostics_with_parser(
        root,
        config,
        all_files,
        sources,
        defer_suppression,
        syn::parse_file,
    )
}

pub(super) fn diagnostics_with_parser(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
    parse: impl Fn(&str) -> syn::Result<syn::File> + Sync,
) -> Result<super::analysis_policy::AnalysisDiagnostics> {
    let mut work = BTreeMap::<PathBuf, RustWork>::new();
    add_max_lines_work(root, config, all_files, &mut work)?;
    add_inline_tests_work(root, config, all_files, &mut work)?;
    add_inline_allows_work(root, config, all_files, &mut work)?;

    let outputs: Result<Vec<_>> = work
        .par_iter()
        .map(|(path, work)| {
            let findings =
                scan::scan_file_outputs(root, path, work, sources, defer_suppression, &parse);
            let mut diagnostics = super::analysis_policy::AnalysisDiagnostics::default();
            for (rule, limit) in &work.applications {
                let selected = findings
                    .iter()
                    .filter(|(candidate_limit, finding)| {
                        finding.rule == rule.rule && candidate_limit == limit
                    })
                    .map(|(_, finding)| finding.clone())
                    .collect();
                diagnostics.append(super::file_diagnostics::route_application(
                    config,
                    rule,
                    selected,
                    |_| true,
                )?);
            }
            Ok(diagnostics)
        })
        .collect();
    let mut diagnostics = super::analysis_policy::AnalysisDiagnostics::default();
    for output in outputs? {
        diagnostics.append(output);
    }
    Ok(super::file_diagnostics::sorted(diagnostics))
}

#[cfg(test)]
mod tests;
#[cfg(test)]
pub(crate) mod tests_support;
