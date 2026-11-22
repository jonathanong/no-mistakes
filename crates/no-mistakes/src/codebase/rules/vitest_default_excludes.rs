use super::{path_filter::RulePathFilter, RuleFinding};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::codebase::check_outcome::CheckInputFailure;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::Arc,
};

mod extract;
pub(crate) use extract::collect;

pub const RULE_ID: &str = "vitest-default-excludes";
const DEFAULT_GLOBS: [&str; 2] = ["**/node_modules/**", "**/.git/**"];

pub(crate) struct ExcludeArray {
    line: usize,
    globs: FxHashSet<String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    #[serde(default)]
    config: String,
    #[serde(default = "default_globs")]
    required_globs: Vec<String>,
}

fn default_globs() -> Vec<String> {
    DEFAULT_GLOBS.into_iter().map(str::to_owned).collect()
}

struct Application<'a> {
    path: PathBuf,
    required_globs: Vec<String>,
    message: Option<&'a str>,
}

fn applications<'a>(root: &Path, config: &'a NoMistakesConfig) -> Result<Vec<Application<'a>>> {
    let mut applications = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let options: Options = rule.try_rule_options()?;
        anyhow::ensure!(
            !options.config.trim().is_empty(),
            "{RULE_ID} requires options.config naming an explicit Vitest config file"
        );
        anyhow::ensure!(
            options
                .required_globs
                .iter()
                .all(|glob| !glob.trim().is_empty()),
            "{RULE_ID} requiredGlobs entries must be nonempty strings"
        );
        let path = normalize_path(&root.join(options.config));
        if !options.required_globs.is_empty()
            && RulePathFilter::new(root, config, rule)?.is_match(&path)
        {
            anyhow::ensure!(
                crate::codebase::dependencies::extract::is_indexable(&path),
                "{RULE_ID} options.config must name a TypeScript or JavaScript file: {}",
                path.display()
            );
            let mut seen = FxHashSet::default();
            applications.push(Application {
                path,
                required_globs: options
                    .required_globs
                    .into_iter()
                    .filter(|glob| seen.insert(glob.clone()))
                    .collect(),
                message: rule.message.as_deref(),
            });
        }
    }
    Ok(applications)
}

#[doc(hidden)]
pub fn required_files(root: &Path, config: &NoMistakesConfig) -> Result<Arc<BTreeSet<PathBuf>>> {
    Ok(Arc::new(
        applications(root, config)?
            .into_iter()
            .map(|app| app.path)
            .collect(),
    ))
}

/// Check the selected Vitest configs using the supplied file inventory.
pub fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(files);
    let plan = CheckFactPlan {
        vitest_default_exclude_paths: required_files(root, config)?,
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
    let mut findings = check_prepared(root, config, Some(&facts), &sources, None)?;
    super::suppress_rule_findings_with_sources(root, &mut findings, &sources);
    Ok(findings)
}

pub(crate) fn check_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    facts: Option<&CheckFactMap>,
    sources: &SourceStore,
    session: Option<&AnalysisSession>,
) -> Result<Vec<RuleFinding>> {
    let batches: Result<Vec<_>> = applications(root, config)?
        .into_par_iter()
        .map(|app| {
            let result = check_application(root, app, facts, sources);
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
    Ok(file_facts.vitest_default_excludes.iter().filter_map(|array| {
        let missing: Vec<_> = app.required_globs.iter()
            .filter(|glob| !array.globs.contains(glob.as_str()))
            .map(String::as_str)
            .collect();
        (!missing.is_empty()).then(|| RuleFinding {
            causes: Vec::new(),
            rule: RULE_ID.into(),
            file: file.clone(),
            line: array.line,
            message: app.message.map(str::to_owned).unwrap_or_else(|| format!(
                "Add {} to test.exclude. Setting test.exclude replaces Vitest's default exclusions.",
                missing.join(", ")
            )),
            import: None,
            target: None,
        })
    }).collect())
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
