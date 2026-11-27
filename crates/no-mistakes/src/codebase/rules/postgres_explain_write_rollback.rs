use super::{analysis_policy::AnalysisDiagnostics, path_filter::GlobMatcher};
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::{schema::RuleDef, NoMistakesConfig, Unanalyzable};
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
pub(in crate::codebase::rules) struct Options {
    sql_include: Vec<String>,
}

pub(in crate::codebase::rules) fn validate_options(options: &Options) -> Result<GlobMatcher> {
    if options.sql_include.is_empty() {
        bail!("{RULE_ID} requires options.sqlInclude to select standalone SQL scripts");
    }
    GlobMatcher::new(&options.sql_include, &format!("{RULE_ID} sqlInclude"))
}

fn selected_for_rule(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<PathBuf>> {
    let options: Options = rule.try_rule_options()?;
    let include = validate_options(&options)?;
    super::selected_rule_files_matching_with_sources(root, config, rule, files, sources, |path| {
        path.extension().is_some_and(|ext| ext == "sql")
            && include.is_match(&relative_slash_path(root, path))
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
        selected.extend(selected_for_rule(root, config, rule, files, sources)?);
    }
    Ok(Arc::new(selected))
}

fn resolve_unanalyzable(config: &NoMistakesConfig, rule: &RuleDef) -> Result<Unanalyzable> {
    let explicit = rule.options.get("unanalyzable").is_some()
        || rule.options.get("unanalyzableSql").is_some()
        || config.unanalyzable.is_some()
        || config.unanalyzable_sql.is_some();
    if explicit {
        Ok(config.effective_rule_analysis(rule)?.unanalyzable)
    } else {
        Ok(Unanalyzable::Advise)
    }
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
    diagnostics_with_files_sources_and_facts(root, config, files, sources, None, false)
        .map(|diagnostics| diagnostics.findings)
}

#[doc(hidden)]
pub fn diagnostics_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let local_facts = match facts {
        Some(_) => None,
        None => {
            let paths = required_files(root, config, files, sources)?;
            Some(
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
                ),
            )
        }
    };
    let facts = facts.or(local_facts.as_ref()).expect("prepared facts");
    let mut diagnostics = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let policy = resolve_unanalyzable(config, rule)?;
        let selected = selected_for_rule(root, config, rule, files, sources)?;
        let mut findings = Vec::new();
        let mut unanalyzable = Vec::new();
        for path in &selected {
            let file = relative_slash_path(root, path);
            let file_facts = facts.postgres.explain_write_rollback(path)?;
            for plan in &file_facts.writes {
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
            for item in &file_facts.unanalyzable {
                let line = item.location.line.max(1) as usize;
                let column = item.location.column.max(1);
                unanalyzable.push(RuleFinding {
                    rule: RULE_ID.to_owned(),
                    file: file.clone(),
                    line,
                    message: format!(
                        "{file}:{line}:{column}: cannot verify {RULE_ID} ({}). Fix the script so its SQL and transaction state can be statically analyzed, suppress with `-- no-mistakes-disable-next-line {RULE_ID}: reason`, or set options.unanalyzable to advise or ignore.",
                        item.reason
                    ),
                    import: None,
                    target: Some("unanalyzable".to_owned()),
                    causes: Vec::new(),
                });
            }
        }
        if let Some(prefix) = &rule.message {
            for finding in findings.iter_mut().chain(unanalyzable.iter_mut()) {
                finding.message = format!("{prefix} {}", finding.message);
            }
        }
        if !defer_suppression {
            super::suppress_rule_findings_with_sources(root, &mut findings, sources.as_ref());
            super::suppress_rule_findings_with_sources(root, &mut unanalyzable, sources.as_ref());
        }
        diagnostics.findings.extend(findings);
        for finding in unanalyzable {
            diagnostics.unanalyzable(policy, finding);
        }
    }
    super::sort_findings(&mut diagnostics.findings);
    diagnostics.findings.dedup();
    super::sort_findings(&mut diagnostics.advisories);
    diagnostics.advisories.dedup();
    Ok(diagnostics)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
