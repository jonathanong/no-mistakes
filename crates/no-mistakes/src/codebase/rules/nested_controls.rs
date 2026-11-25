//! Checks native HTML controls whose JSX ancestry is visible within one file.
mod extract;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;

use super::analysis_policy::AnalysisDiagnostics;
use super::RuleFinding;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::codebase::check_outcome::CheckInputFailure;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::schema::{NoMistakesConfig, RuleDef};
use crate::config::v2::Unanalyzable;
use anyhow::{Context, Result};
use extract::{Kind, UnanalyzableReason};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const RULE_ID: &str = "react-no-nested-buttons-and-links";
pub(crate) use extract::Location;

fn selected_files(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<PathBuf>> {
    super::selected_rule_files_matching_with_sources(root, config, rule, files, sources, |path| {
        matches!(path.extension().and_then(|ext| ext.to_str()), Some("jsx" | "tsx"))
    })
}

#[doc(hidden)]
pub fn required_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Arc<BTreeSet<PathBuf>>> {
    let mut selected = BTreeSet::new();
    for rule in config.rule_applications(RULE_ID) {
        selected.extend(selected_files(root, config, rule, files, sources)?);
    }
    Ok(Arc::new(selected))
}

pub(crate) fn collect(
    path: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    plan: &CheckFactPlan,
) -> Option<Vec<Location>> {
    plan.nested_control_paths
        .contains(path)
        .then(|| extract::collect(source, program))
}

/// Check scoped JSX and TSX files for native buttons and links nested in one another.
pub fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(files);
    let plan = CheckFactPlan {
        nested_control_paths: required_files(root, config, files, &sources)?,
        ..Default::default()
    };
    let facts = crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
        root,
        plan.nested_control_paths.iter().cloned().collect(),
        Vec::new(),
        plan,
        None,
        Arc::clone(&sources),
    );
    Ok(diagnostics_prepared(root, config, files, Some(&facts), &sources, None, false)?.findings)
}

pub(crate) fn check_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    facts: Option<&CheckFactMap>,
    sources: &SourceStore,
    session: Option<&AnalysisSession>,
) -> Result<Vec<RuleFinding>> {
    Ok(diagnostics_prepared(root, config, files, facts, sources, session, true)?.findings)
}

#[doc(hidden)]
pub fn diagnostics_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    facts: Option<&CheckFactMap>,
    sources: &SourceStore,
    session: Option<&AnalysisSession>,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut diagnostics = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let policy = config.effective_rule_analysis(rule)?.unanalyzable;
        for path in selected_files(root, config, rule, files, sources)? {
            let result = check_file(root, &path, facts, sources, rule.message.as_deref(), policy);
            if let Some(session) = session {
                if let Some(chunk) = session.capture_check_result(RULE_ID, result)? {
                    diagnostics.append(chunk);
                }
            } else {
                diagnostics.append(result?);
            }
        }
    }
    if !defer_suppression {
        super::suppress_rule_findings_with_sources(root, &mut diagnostics.findings, sources);
        super::suppress_rule_findings_with_sources(root, &mut diagnostics.advisories, sources);
    }
    super::sort_findings(&mut diagnostics.findings);
    super::sort_findings(&mut diagnostics.advisories);
    Ok(diagnostics)
}

fn check_file(
    root: &Path,
    path: &Path,
    facts: Option<&CheckFactMap>,
    sources: &SourceStore,
    custom_message: Option<&str>,
    policy: Unanalyzable,
) -> Result<AnalysisDiagnostics> {
    let file = relative_slash_path(root, path);
    let file_facts = facts
        .and_then(|facts| facts.ts.get(path))
        .with_context(|| format!("{RULE_ID}: missing prepared JSX facts for {file}"))?;
    if let Some(message) = &file_facts.parse_error {
        if sources.read_source_path(path).is_ok_and(|source| {
            crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID)
        }) {
            return Ok(AnalysisDiagnostics::default());
        }
        return Err(CheckInputFailure {
            file: Some(PathBuf::from(file)),
            property: None,
            message: format!("{message}. Repair the JSX so {RULE_ID} can verify control nesting."),
        }
        .into());
    }
    let locations = file_facts
        .nested_controls
        .as_ref()
        .with_context(|| format!("{RULE_ID}: missing prepared nesting facts for {file}"))?;
    let mut diagnostics = AnalysisDiagnostics::default();
    for location in locations {
        let (target, default_message) = match &location.kind {
            Kind::Nested { inner, outer } => (
                None,
                format!("<{inner}> is nested inside <{outer}>. Move the inner control outside the outer control and render them as sibling elements."),
            ),
            Kind::Unanalyzable(reason) => (Some("unanalyzable".into()), unanalyzable_message(reason)),
        };
        let finding = RuleFinding {
            rule: RULE_ID.into(),
            file: file.clone(),
            line: location.line,
            message: custom_message.map(str::to_owned).unwrap_or(default_message),
            import: None,
            target,
            causes: Vec::new(),
        };
        if matches!(location.kind, Kind::Unanalyzable(_)) {
            diagnostics.unanalyzable(policy, finding);
        } else {
            diagnostics.findings.push(finding);
        }
    }
    Ok(diagnostics)
}

fn unanalyzable_message(reason: &UnanalyzableReason) -> String {
    match reason {
        UnanalyzableReason::UncertainHref => "<a> has a non-literal or spread-overwritten href, so control nesting cannot be proven. Use a literal href string, remove the trailing spread, or suppress with a reason.".into(),
        UnanalyzableReason::ChildExpression => "JSX child expression inside a button or link cannot be proven free of nested controls. Render static sibling elements or suppress with a reason.".into(),
        UnanalyzableReason::ChildSpread => "JSX spread inside a button or link cannot be proven free of nested controls. Render static sibling elements or suppress with a reason.".into(),
        UnanalyzableReason::WrapperTag(Some(name)) => format!("<{name}> inside a button or link is outside the supported static wrapper tags (a, button, div, em, span, strong), so nested controls cannot be proven absent. Use a supported native wrapper, move controls to siblings, or suppress with a reason."),
        UnanalyzableReason::WrapperTag(None) => "Unresolved JSX element inside a button or link cannot be proven free of nested controls. Use a supported native wrapper, move controls to siblings, or suppress with a reason.".into(),
    }
}
