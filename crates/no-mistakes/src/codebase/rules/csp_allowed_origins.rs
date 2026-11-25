use super::analysis_policy::AnalysisDiagnostics;
use super::RuleFinding;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod extract;
mod findings;
mod load;
mod options;
mod policy;

pub(crate) use extract::collect;
use findings::{csp_unanalyzable_finding, origin_finding, unanalyzable_finding};
pub(crate) use findings::{CollectedOrigins, OriginRef, UnanalyzableReason, UnanalyzableRef};
use load::load_policy;
use options::{applications, Application};
pub(crate) use policy::{CspDirective, CspPolicySet, ParsedOrigin};

pub const RULE_ID: &str = "csp-allowed-origins";

#[doc(hidden)]
pub fn required_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Arc<BTreeSet<PathBuf>>> {
    let mut paths = BTreeSet::new();
    for app in applications(root, config, files)? {
        for csp_file in app.csp_files {
            if crate::codebase::dependencies::extract::is_indexable(&csp_file) {
                paths.insert(csp_file);
            }
        }
        paths.extend(app.target_files);
    }
    Ok(Arc::new(paths))
}

/// Check configured JS/TS/JSX/TSX files for third-party browser origins missing from CSP.
pub fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    diagnostics_with_files(root, config, files).map(|diagnostics| diagnostics.findings)
}

#[doc(hidden)]
pub fn diagnostics_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<AnalysisDiagnostics> {
    let paths = required_files(root, config, files)?;
    let sources = super::source_store_for_files(files);
    let target_paths: Vec<PathBuf> = paths.iter().cloned().collect();
    let plan = CheckFactPlan {
        csp_allowed_origins_paths: paths,
        ..Default::default()
    };
    let facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            root,
            target_paths,
            Vec::new(),
            plan,
            None,
            Arc::clone(&sources),
        );
    diagnostics_prepared(root, config, files, Some(&facts), &sources, None, false)
}

pub(crate) fn check_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    facts: Option<&CheckFactMap>,
    sources: &SourceStore,
    session: Option<&AnalysisSession>,
) -> Result<Vec<RuleFinding>> {
    diagnostics_prepared(root, config, files, facts, sources, session, true)
        .map(|diagnostics| diagnostics.findings)
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
    let normalized_root = normalize_path(root);
    let apps = applications(&normalized_root, config, files)?;
    if apps.is_empty() {
        return Ok(AnalysisDiagnostics::default());
    }
    let per_app: Result<Vec<Option<AnalysisDiagnostics>>> = apps
        .into_par_iter()
        .map(|app| {
            let outcome = check_application(&normalized_root, app, facts, sources);
            match session {
                Some(session) => session.capture_check_result(RULE_ID, outcome),
                None => outcome.map(Some),
            }
        })
        .collect();
    let mut diagnostics = AnalysisDiagnostics::default();
    for batch in per_app?.into_iter().flatten() {
        diagnostics.append(batch);
    }
    if !defer_suppression {
        super::suppress_rule_findings_with_sources(
            &normalized_root,
            &mut diagnostics.findings,
            sources,
        );
        super::suppress_rule_findings_with_sources(
            &normalized_root,
            &mut diagnostics.advisories,
            sources,
        );
    }
    Ok(super::file_diagnostics::sorted(diagnostics))
}

fn check_application(
    root: &Path,
    app: Application<'_>,
    facts: Option<&CheckFactMap>,
    sources: &SourceStore,
) -> Result<AnalysisDiagnostics> {
    let loaded = load_policy(root, &app.csp_files, facts, sources)?;
    let csp_label = app
        .csp_files
        .iter()
        .map(|path| relative_slash_path(root, path))
        .collect::<Vec<_>>()
        .join(", ");
    let mut diagnostics = AnalysisDiagnostics::default();
    for (csp_file, line) in &loaded.unanalyzable {
        diagnostics.unanalyzable(
            app.unanalyzable,
            csp_unanalyzable_finding(csp_file, *line, app.message),
        );
    }
    for path in &app.target_files {
        let file = relative_slash_path(root, path);
        let Some(file_facts) = facts.and_then(|facts| facts.ts.get(path)) else {
            anyhow::bail!("{RULE_ID}: missing prepared facts for {file}");
        };
        if file_facts.parse_error.is_some() {
            continue;
        }
        if !loaded.policy.is_empty() {
            for origin_ref in &file_facts.csp_allowed_origins.refs {
                if !loaded
                    .policy
                    .allows(origin_ref.directive, &origin_ref.origin, &app.self_origins)
                {
                    diagnostics.findings.push(origin_finding(
                        &file,
                        origin_ref.line,
                        origin_ref.directive,
                        &origin_ref.origin,
                        &csp_label,
                        app.message,
                    ));
                }
            }
        }
        for item in &file_facts.csp_allowed_origins.unanalyzable {
            diagnostics.unanalyzable(
                app.unanalyzable,
                unanalyzable_finding(&file, item, &csp_label, app.message),
            );
        }
    }
    Ok(diagnostics)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
