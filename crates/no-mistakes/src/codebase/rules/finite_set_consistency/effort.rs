use super::scan::{scan, ScanInput};
use super::{NoMistakesConfig, Options, RULE_ID};
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::rules::{
    analysis_policy::AnalysisDiagnostics, file_allowed_by_roots_and_skip, path_filter,
    skip_dir_set, sort_findings, suppress_rule_findings_with_sources, target_roots,
};
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub(super) fn diagnostics_with_lookup(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: Option<&dyn TsFactLookup>,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let observer = crate::diagnostics::current();
    let all: Result<Vec<_>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| {
            crate::diagnostics::with_observer(observer.clone(), || {
                let opts: Options = rule.try_rule_options()?;
                let target_roots = target_roots(root, config, rule);
                let skip = skip_dir_set(config);
                let files: Vec<PathBuf> = all_files
                    .iter()
                    .filter(|p| file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
                    .cloned()
                    .collect();
                let files = path_filter::filter_rule_files(root, config, rule, &files)?;
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
        })
        .collect();
    let mut output = AnalysisDiagnostics::default();
    for chunk in all? {
        output.append(chunk);
    }
    if !defer_suppression {
        suppress_rule_findings_with_sources(root, &mut output.findings, sources);
        suppress_rule_findings_with_sources(root, &mut output.advisories, sources);
    }
    sort_findings(&mut output.findings);
    sort_findings(&mut output.advisories);
    Ok(output)
}
