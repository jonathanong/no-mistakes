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
mod load;
mod options;
mod policy;

pub(crate) use extract::collect;
use load::load_policy;
use options::{applications, Application};
pub(crate) use policy::{CspDirective, CspPolicy, ParsedOrigin};

pub const RULE_ID: &str = "csp-allowed-origins";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OriginRef {
    pub(crate) line: usize,
    pub(crate) directive: CspDirective,
    pub(crate) url: String,
}

#[derive(Default)]
pub(crate) struct CollectedOrigins {
    pub(crate) refs: Vec<OriginRef>,
    pub(crate) csp_strings: Vec<String>,
}

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
    let normalized_root = normalize_path(root);
    let apps = applications(&normalized_root, config, files)?;
    if apps.is_empty() {
        return Ok(Vec::new());
    }
    let mut findings = Vec::new();
    let per_app: Result<Vec<Option<Vec<RuleFinding>>>> = apps
        .into_par_iter()
        .map(|app| {
            let outcome = check_application(&normalized_root, app, facts, sources);
            if let Some(session) = session {
                session.capture_check_result(RULE_ID, outcome)
            } else {
                outcome.map(Some)
            }
        })
        .collect();
    for batch in per_app?.into_iter().flatten() {
        findings.extend(batch);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn check_application(
    root: &Path,
    app: Application<'_>,
    facts: Option<&CheckFactMap>,
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    let policy = load_policy(root, &app.csp_files, facts, sources)?;
    let csp_label = app
        .csp_files
        .iter()
        .map(|path| relative_slash_path(root, path))
        .collect::<Vec<_>>()
        .join(", ");
    let mut findings = Vec::new();
    for path in &app.target_files {
        let file = relative_slash_path(root, path);
        let Some(file_facts) = facts.and_then(|facts| facts.ts.get(path)) else {
            anyhow::bail!("{RULE_ID}: missing prepared facts for {file}");
        };
        if file_facts.parse_error.is_some() {
            continue;
        }
        for origin_ref in &file_facts.csp_allowed_origins.refs {
            let Some(origin) = ParsedOrigin::parse(&origin_ref.url) else {
                continue;
            };
            if policy.allows(origin_ref.directive, &origin, &app.self_origins) {
                continue;
            }
            let origin_str = origin.display();
            let directive = origin_ref.directive.as_str();
            findings.push(RuleFinding {
                causes: Vec::new(),
                rule: RULE_ID.into(),
                file: file.clone(),
                line: origin_ref.line,
                message: app.message.map(str::to_owned).unwrap_or_else(|| {
                    format!(
                        "Origin '{origin_str}' used for {directive} is not allowed by CSP in {csp_label}. Add '{origin_str}' to {directive} in {csp_label} or remove the third-party origin."
                    )
                }),
                import: None,
                target: None,
            });
        }
    }
    Ok(findings)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
