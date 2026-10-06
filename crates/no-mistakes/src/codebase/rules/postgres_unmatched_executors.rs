//! Opt-in `reportUnmatchedExecutorNames` pass shared by the embedded-SQL rules.
//!
//! A configured `executorFactoryNames` / `executorTypeNames` entry that no
//! scanned file imports from `importSpecifier` silently disables checking for
//! every query run through that executor. The per-file facts already record
//! which configured names each file imported; this pass only aggregates them.

use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::{EmbeddedSqlFileFacts, EmbeddedSqlOptions};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    include: Vec<String>,
    exclude: Vec<String>,
    import_specifier: String,
    executor_names: Vec<String>,
    executor_factory_names: Vec<String>,
    executor_type_names: Vec<String>,
    trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    report_unmatched_executor_names: bool,
}

impl Options {
    fn profile(&self) -> EmbeddedSqlOptions {
        EmbeddedSqlOptions::configured(&self.import_specifier, &self.executor_names)
            .with_scoped_executors(&self.executor_factory_names, &self.executor_type_names)
            .with_trusted_sql_tags(&self.trusted_sql_tags)
    }
}

/// Whether any application of `rule_id` opts in to unmatched-name reporting.
pub(crate) fn is_requested(config: &NoMistakesConfig, rule_id: &str) -> Result<bool> {
    for rule in config.rule_applications(rule_id) {
        if rule
            .try_rule_options::<Options>()?
            .report_unmatched_executor_names
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// One finding per configured name that no scanned file imported, for every
/// opted-in application of `rule_id`. Reuses `facts` when the caller owns the
/// request-scoped fact map; otherwise prepares one standalone.
pub(crate) fn check(
    root: &Path,
    config: &NoMistakesConfig,
    rule_id: &str,
    all_files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    if !is_requested(config, rule_id)? {
        return Ok(Vec::new());
    }
    let standalone;
    let facts = match facts {
        Some(facts) => facts,
        None => {
            let profiles =
                crate::codebase::postgres::configured_embedded_sql_options(config, &[rule_id])?;
            standalone = crate::codebase::postgres::prepare_embedded_sql_facts(
                root,
                all_files,
                Arc::clone(sources),
                profiles,
                Vec::new(),
            );
            &standalone
        }
    };
    let config_file = super::no_mistakes_config::config_rel(root, all_files);
    let mut findings = Vec::new();
    for rule in config.rule_applications(rule_id) {
        let opts: Options = rule.try_rule_options()?;
        if !opts.report_unmatched_executor_names {
            continue;
        }
        let include = GlobMatcher::new(&opts.include, &format!("{rule_id} include"))?;
        let exclude = GlobMatcher::new(&opts.exclude, &format!("{rule_id} exclude"))?;
        let target_roots = super::target_roots(root, config, rule);
        let skip = super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots))
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        let profile = opts.profile();
        let mut matched = (BTreeSet::new(), BTreeSet::new());
        let mut scanned = 0usize;
        for path in files.iter().filter(|path| {
            let rel = relative_slash_path(root, path);
            crate::codebase::dependencies::extract::is_indexable(path)
                && (include.is_empty() || include.is_match(&rel))
                && (exclude.is_empty() || !exclude.is_match(&rel))
        }) {
            // A file that failed to parse has no facts; it cannot prove a match.
            let Ok(file) = facts.embedded_sql(path, &profile) else {
                continue;
            };
            scanned += 1;
            record(file, &mut matched);
        }
        let ctx = Context {
            rule_id,
            config_file: &config_file,
            specifier: &opts.import_specifier,
            scanned,
        };
        findings.extend(unmatched(
            &ctx,
            "executorFactoryNames",
            &profile.executor_factory_names,
            &matched.0,
        ));
        findings.extend(unmatched(
            &ctx,
            "executorTypeNames",
            &profile.executor_type_names,
            &matched.1,
        ));
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn record(file: &EmbeddedSqlFileFacts, matched: &mut (BTreeSet<String>, BTreeSet<String>)) {
    matched.0.extend(file.matched_factory_names.iter().cloned());
    matched.1.extend(file.matched_type_names.iter().cloned());
}

struct Context<'a> {
    rule_id: &'a str,
    config_file: &'a str,
    specifier: &'a str,
    scanned: usize,
}

fn unmatched(
    ctx: &Context<'_>,
    option: &str,
    configured: &[String],
    matched: &BTreeSet<String>,
) -> Vec<RuleFinding> {
    configured
        .iter()
        .filter(|name| !matched.contains(*name))
        .map(|name| RuleFinding {
            rule: ctx.rule_id.to_string(),
            file: ctx.config_file.to_string(),
            line: 1,
            message: message(ctx, option, name),
            import: None,
            target: Some("unmatched-executor-name".to_string()),
        })
        .collect()
}

fn message(ctx: &Context<'_>, option: &str, name: &str) -> String {
    let module = if ctx.specifier.is_empty() {
        "any module".to_string()
    } else {
        format!("`{}` or its subpaths", ctx.specifier)
    };
    format!(
        "{rule}: {option} entry `{name}` was not imported from {module} in any of the {scanned} \
         file(s) this rule scanned, so queries run through it are not checked. Fix: correct the \
         spelling to match the exported name; set `importSpecifier` to the module that exports \
         it (a re-export from another path does not match; a relative import matches only when \
         it resolves into that package); or remove the entry if it is unused. This check exists because a \
         typo here silently disables the rule for that executor; set \
         `reportUnmatchedExecutorNames: false` to turn it off.",
        rule = ctx.rule_id,
        scanned = ctx.scanned,
    )
}

#[cfg(test)]
mod tests;
