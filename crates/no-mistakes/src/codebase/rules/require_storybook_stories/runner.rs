pub(super) use super::diagnostics::check_with_resolver;
use super::suppression::unsuppressed_component_keys;
use super::types::RuleCheck;
use super::{
    all_react_component_keys, colocated_test_covered_components, component_disabled,
    directly_covered_components, dynamic_or_mock_boundary_files, effective_story_patterns,
    file_disabled, namespace_import_findings, reachable_story_files, selected_components,
    stale_or_blank_allow_findings, transitive_covered_components, GlobMatcher, Options, RULE_ID,
};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::rules::{
    analysis_policy::AnalysisDiagnostics, path_filter::RulePathFilter, sort_findings,
};
use crate::codebase::ts_resolver::{normalize_path, ImportResolution};
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::{bail, Result};
use std::collections::HashSet;
use std::path::Path;
pub(super) fn check_diagnostics_with_resolver(
    root: &Path,
    config: &NoMistakesConfig,
    shared: &CheckFactMap,
    resolver: &dyn ImportResolution,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
    defer_suppression: bool,
    source_context: (
        &crate::codebase::ts_source::SourceStore,
        Option<&crate::codebase::analysis_session::AnalysisSession>,
    ),
) -> Result<AnalysisDiagnostics> {
    let (sources, session) = source_context;
    let root = normalize_path(root);
    let mut diagnostics = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        if rule.projects.len() != 1 || rule.applies_to_repository() {
            bail!("{RULE_ID} requires exactly one project target");
        }
        let Some(project) = config.projects.get(&rule.projects[0]) else {
            continue;
        };
        let project_root = project
            .root
            .as_deref()
            .map(|path| root.join(path))
            .unwrap_or_else(|| root.to_path_buf());
        let project_root = normalize_path(&project_root);
        diagnostics.append(check_rule(RuleCheck {
            session,
            root: &root,
            project_root: &project_root,
            config,
            rule,
            shared,
            resolver,
            inferred_roots,
            defer_suppression,
            sources,
        })?);
    }
    sort_findings(&mut diagnostics.findings);
    sort_findings(&mut diagnostics.advisories);
    Ok(diagnostics)
}
mod allow;
mod execution;
use execution::check_rule;
