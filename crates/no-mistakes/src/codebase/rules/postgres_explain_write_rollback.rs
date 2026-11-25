use super::{path_filter::GlobMatcher, RuleFinding};
use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::Arc,
};

pub const RULE_ID: &str = "postgres-explain-write-rollback";

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    sql_include: Vec<String>,
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
        let options: Options = rule.try_rule_options()?;
        if options.sql_include.is_empty() {
            bail!("{RULE_ID} requires options.sqlInclude to select standalone SQL scripts");
        }
        let include = GlobMatcher::new(&options.sql_include, &format!("{RULE_ID} sqlInclude"))?;
        selected.extend(super::selected_rule_files_matching_with_sources(
            root,
            config,
            rule,
            files,
            sources,
            |path| {
                path.extension().is_some_and(|ext| ext == "sql")
                    && include.is_match(&relative_slash_path(root, path))
            },
        )?);
    }
    Ok(Arc::new(selected))
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    super::check_with_source_store(root, config, files, check_with_files_and_sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
) -> Result<Vec<RuleFinding>> {
    let paths = required_files(root, config, files, sources)?;
    let facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            root,
            paths.iter().cloned().collect(),
            Vec::new(),
            CheckFactPlan {
                postgres_explain_write_rollback_paths: paths,
                ..Default::default()
            },
            None,
            Arc::clone(sources),
        );
    check_with_files_sources_and_facts(root, config, files, sources, &facts)
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    _config: &NoMistakesConfig,
    files: &[PathBuf],
    _sources: &SourceStore,
    facts: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let paths = crate::codebase::check_facts::PathMembership::new(files);
    let mut findings = Vec::new();
    for path in facts
        .postgres
        .explain_write_rollback
        .keys()
        .filter(|path| paths.contains(path))
    {
        let file = relative_slash_path(root, path);
        for plan in facts.postgres.explain_write_rollback(path)? {
            let line = plan.location.line as usize;
            let column = plan.location.column;
            let kind = plan.kind;
            let outcome = if plan.autocommit {
                "with autocommit enabled"
            } else {
                "inside a transaction that commits"
            };
            findings.push(RuleFinding {
                rule: RULE_ID.to_owned(),
                file: file.clone(),
                line,
                message: format!(
                    "{file}:{line}:{column}: EXPLAIN ANALYZE executes {kind} {outcome}. Run this write plan inside a transaction ending with ROLLBACK, or remove ANALYZE to inspect the plan without executing the write."
                ),
                import: None,
                target: Some(kind.to_owned()),
                causes: Vec::new(),
            });
        }
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}
