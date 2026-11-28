use no_mistakes_rule_kernel::file_diagnostics::AnalysisDiagnostics;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

mod options;
mod scan;

pub const RULE_ID: &str = "github-actions-cache-setup-policy";

pub(in crate::codebase::rules) fn validate_options(options: Options) -> Result<()> {
    options::compile(options).map(drop)
}
pub use options::{CacheKeyShape, Options, SetupDeadline};

super::diagnostic_source_rule_entrypoints!();

#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    use super::file_diagnostics::{application, sorted, InputUncertainty};

    let mut output = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let options = options::compile(rule.try_rule_options()?)?;
        let (selected, roots) = super::selected_rule_files_with_roots(root, config, rule, files)?;
        let selected = super::matching_files(root, &options.include, &selected, &roots)?;
        let findings = selected
            .par_iter()
            .flat_map_iter(|path| scan::file(root, path, sources, &options))
            .collect();
        output.append(application(
            root,
            sources,
            config,
            rule,
            findings,
            defer_suppression,
            InputUncertainty::ReadOrParse,
        )?);
    }
    Ok(sorted(output))
}
