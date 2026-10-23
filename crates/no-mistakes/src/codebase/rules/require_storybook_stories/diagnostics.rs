use super::{GlobMatcher, NoMistakesConfig, RuleFinding};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::rules::{analysis_policy::AnalysisDiagnostics, path_filter::RulePathFilter};
use crate::codebase::ts_resolver::ImportResolution;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::Unanalyzable;
use anyhow::Result;
use std::path::Path;

pub(super) fn check_with_resolver(
    root: &Path,
    config: &NoMistakesConfig,
    shared: &CheckFactMap,
    resolver: &dyn ImportResolution,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
    defer_suppression: bool,
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    super::runner::check_diagnostics_with_resolver(
        root,
        config,
        shared,
        resolver,
        inferred_roots,
        defer_suppression,
        sources,
    )
    .map(|diagnostics| diagnostics.findings)
}

pub(super) fn append_unanalyzable(
    diagnostics: &mut AnalysisDiagnostics,
    findings: Vec<RuleFinding>,
    root: &Path,
    project_root: &Path,
    rule_filter: &RulePathFilter,
    allow_files: &GlobMatcher,
    policy: Unanalyzable,
) {
    for finding in findings {
        let file = root.join(&finding.file);
        let project_file = crate::codebase::ts_source::relative_slash_path(project_root, &file);
        if rule_filter.is_match(&file) && !allow_files.is_match(&project_file) {
            diagnostics.unanalyzable(policy, finding);
        }
    }
}
