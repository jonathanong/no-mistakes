//! Checks native HTML controls whose JSX ancestry is visible within one file.
mod extract;

use super::RuleFinding;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::codebase::check_outcome::CheckInputFailure;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::schema::{NoMistakesConfig, RuleDef};
use anyhow::{Context, Result};
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
    plan.nested_control_paths.contains(path).then(|| extract::collect(source, program))
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
    let mut findings = check_prepared(root, config, files, Some(&facts), &sources, None)?;
    super::suppress_rule_findings_with_sources(root, &mut findings, &sources);
    Ok(findings)
}

pub(crate) fn check_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    facts: Option<&CheckFactMap>,
    sources: &SourceStore,
    session: Option<&AnalysisSession>,
) -> Result<Vec<RuleFinding>> {
    super::run_rule_applications(config, RULE_ID, |rule| {
        let mut findings = Vec::new();
        for path in selected_files(root, config, rule, files, sources)? {
            let result = check_file(root, &path, facts, sources, rule.message.as_deref());
            if let Some(session) = session {
                findings.extend(session.capture_check_result(RULE_ID, result)?.into_iter().flatten());
            } else {
                findings.extend(result?);
            }
        }
        Ok(findings)
    })
}

fn check_file(
    root: &Path,
    path: &Path,
    facts: Option<&CheckFactMap>,
    sources: &SourceStore,
    custom_message: Option<&str>,
) -> Result<Vec<RuleFinding>> {
    let file = relative_slash_path(root, path);
    let file_facts = facts.and_then(|facts| facts.ts.get(path))
        .with_context(|| format!("{RULE_ID}: missing prepared JSX facts for {file}"))?;
    if let Some(message) = &file_facts.parse_error {
        if sources.read_source_path(path).is_ok_and(|source| {
            crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID)
        }) {
            return Ok(Vec::new());
        }
        return Err(CheckInputFailure {
            file: Some(PathBuf::from(file)),
            property: None,
            message: format!("{message}. Repair the JSX so {RULE_ID} can verify control nesting."),
        }.into());
    }
    let locations = file_facts.nested_controls.as_ref()
        .with_context(|| format!("{RULE_ID}: missing prepared nesting facts for {file}"))?;
    Ok(locations.iter().map(|location| RuleFinding {
        rule: RULE_ID.into(),
        file: file.clone(),
        line: location.line,
        message: custom_message.map(str::to_owned).unwrap_or_else(|| format!(
            "<{}> is nested inside <{}>. Move the inner control outside the outer control and render them as sibling elements.",
            location.inner, location.outer,
        )),
        import: None,
        target: None,
        causes: Vec::new(),
    }).collect())
}
