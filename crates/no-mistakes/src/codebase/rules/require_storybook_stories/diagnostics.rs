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

/// Render only prepared error text; the source identity remains request-owned.
pub(super) fn malformed_finding(root: &Path, path: &Path, error: &str, kind: &str) -> RuleFinding {
    let file = crate::codebase::ts_source::relative_slash_path(root, path);
    let cause = error.lines().next().unwrap_or("").trim();
    let cause = cause.replace(path.to_string_lossy().as_ref(), &file);
    let detail = if cause.is_empty() {
        String::new()
    } else {
        format!(" ({cause})")
    };
    RuleFinding {
        rule: super::RULE_ID.to_string(),
        file,
        line: 1,
        message: format!("{kind} could not be analyzed{detail}, so Storybook coverage is incomplete. Fix the source syntax or read error and rerun the check, exclude this file, or add a documented no-mistakes-disable-file exemption."),
        import: None,
        target: Some("unanalyzable".to_string()),
    }
}

pub(super) fn malformed_stories(
    root: &Path,
    shared: &CheckFactMap,
    stories: &std::collections::BTreeSet<std::path::PathBuf>,
    defer_suppression: bool,
) -> (bool, Vec<RuleFinding>) {
    let mut incomplete = false;
    let mut findings = Vec::new();
    for path in stories {
        let Some(error) = shared
            .ts
            .get(path)
            .and_then(|facts| facts.parse_error.as_deref())
        else {
            continue;
        };
        // A suppressed diagnostic cannot restore facts hidden by a parse/read error.
        incomplete = true;
        if !defer_suppression && story_suppressed(shared, path) {
            continue;
        }
        findings.push(malformed_finding(
            root,
            path,
            error,
            "Selected Storybook story",
        ));
    }
    (incomplete, findings)
}

pub(super) fn visible_story_error(
    root: &Path,
    project_root: &Path,
    shared: &CheckFactMap,
    findings: &[RuleFinding],
    filter: &RulePathFilter,
    allow: &GlobMatcher,
) -> bool {
    findings.iter().any(|finding| {
        let path = root.join(&finding.file);
        filter.is_match(&path)
            && !allow.is_match(&crate::codebase::ts_source::relative_slash_path(
                project_root,
                &path,
            ))
            && !story_suppressed(shared, &path)
    })
}

pub(super) fn incomplete_component(component: super::types::Component) -> RuleFinding {
    RuleFinding {
        rule: super::RULE_ID.to_string(),
        file: component.repo_file,
        line: component.line,
        message: format!("Storybook coverage of `{}` is unknown because a selected story could not be analyzed. Fix the malformed story syntax or read error and rerun the check to establish coverage, or add a documented component exemption.", component.export_name),
        import: None,
        target: Some("unanalyzable".to_string()),
    }
}

fn story_suppressed(shared: &CheckFactMap, path: &Path) -> bool {
    shared
        .ts
        .get(path)
        .and_then(|facts| facts.source.as_deref())
        .is_some_and(|source| {
            crate::codebase::ts_source::has_disable_file_comment(source, super::RULE_ID)
                || crate::codebase::ts_source::has_disable_line_comment(source, 1, super::RULE_ID)
        })
}
