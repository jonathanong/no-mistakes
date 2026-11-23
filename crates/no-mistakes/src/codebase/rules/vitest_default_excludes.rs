use super::RuleFinding;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::codebase::check_outcome::CheckInputFailure;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::Arc,
};

mod extract;
mod options;
pub(crate) use extract::collect;
use options::{applications, Application, CheckMode};

pub const RULE_ID: &str = "vitest-default-excludes";

#[derive(Default)]
pub(crate) struct CollectedExcludes {
    pub(crate) default_export: Vec<ExcludeArray>,
    pub(crate) test_literals: Vec<ExcludeArray>,
}

pub(crate) struct ExcludeArray {
    line: usize,
    globs: FxHashSet<String>,
    spreads_config_defaults: bool,
}

#[doc(hidden)]
pub fn required_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Arc<BTreeSet<PathBuf>>> {
    Ok(Arc::new(
        applications(root, config, files)?
            .into_iter()
            .map(|app| app.path)
            .collect(),
    ))
}

/// Check the selected Vitest configs and config factory modules using the supplied file inventory.
pub fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(files);
    let plan = CheckFactPlan {
        vitest_default_exclude_paths: required_files(root, config, files)?,
        ..Default::default()
    };
    let facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            root,
            plan.vitest_default_exclude_paths.iter().cloned().collect(),
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
    let batches: Result<Vec<_>> = applications(&normalized_root, config, files)?
        .into_par_iter()
        .map(|app| {
            let result = check_application(&normalized_root, app, facts, sources);
            match session {
                Some(session) => session.capture_check_result(RULE_ID, result),
                None => result.map(Some),
            }
        })
        .collect();
    let mut findings: Vec<_> = batches?.into_iter().flatten().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn check_application(
    root: &Path,
    app: Application<'_>,
    facts: Option<&CheckFactMap>,
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    let file = relative_slash_path(root, &app.path);
    let Some(file_facts) = facts.and_then(|facts| facts.ts.get(&app.path)) else {
        anyhow::bail!("{RULE_ID}: missing prepared facts for {file}");
    };
    if let Some(message) = &file_facts.parse_error {
        if sources.read_source_path(&app.path).is_ok_and(|source| {
            crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID)
        }) {
            return Ok(Vec::new());
        }
        return Err(CheckInputFailure {
            file: Some(PathBuf::from(file)),
            property: Some("test.exclude".into()),
            message: format!(
                "{message}. Repair the Vitest config so {RULE_ID} can verify test.exclude."
            ),
        }
        .into());
    }
    let arrays = match app.mode {
        CheckMode::DefaultExport => &file_facts.vitest_default_excludes.default_export,
        CheckMode::TestLiterals => &file_facts.vitest_default_excludes.test_literals,
    };
    Ok(arrays
        .iter()
        .filter_map(|array| {
            if array.spreads_config_defaults {
                return None;
            }
            let missing: Vec<_> = app
                .required_globs
                .iter()
                .filter(|glob| !array.globs.contains(glob.as_str()))
                .map(String::as_str)
                .collect();
            (!missing.is_empty()).then(|| RuleFinding {
                causes: Vec::new(),
                rule: RULE_ID.into(),
                file: file.clone(),
                line: array.line,
                message: app.message.map(str::to_owned).unwrap_or_else(|| {
                    format!(
                        "Add {} to test.exclude. Setting test.exclude replaces Vitest's default exclusions.",
                        missing.join(", ")
                    )
                }),
                import: None,
                target: None,
            })
        })
        .collect())
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
