use super::{
    path_filter::{filter_rule_files, RulePathFilter},
    RuleFinding,
};
use crate::{
    codebase::{
        check_facts::CheckFactMap,
        ts_source::{relative_slash_path, SourceStore},
    },
    config::v2::NoMistakesConfig,
};
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::Arc,
};
mod comments;
mod options;
pub(crate) use comments::{collect, Suppression};
pub const RULE_ID: &str = "forbidden-lint-suppressions";

/// Exact configured scope, shared by fact preparation and finding projection.
#[doc(hidden)]
pub fn required_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Arc<BTreeSet<PathBuf>>> {
    let applications = config.rule_applications(RULE_ID);
    if applications.is_empty() {
        return Ok(Arc::new(BTreeSet::new()));
    }
    let (root, files) = normalized_inputs(root, files);
    let root = root.as_path();
    let mut selected = BTreeSet::new();
    for rule in applications {
        let options: options::Options = rule.try_rule_options()?;
        let matcher = options.validate()?;
        let filter = RulePathFilter::new(root, config, rule)?;
        selected.extend(
            files
                .iter()
                .filter(|path| {
                    crate::codebase::dependencies::extract::is_indexable(path)
                        && filter.is_match(path)
                        && matcher.is_match(&relative_slash_path(root, path))
                })
                .map(|path| crate::codebase::ts_resolver::normalize_path(path)),
        );
    }
    Ok(Arc::new(selected))
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(files);
    let plan = crate::codebase::check_facts::CheckFactPlan {
        lint_suppression_paths: required_files(root, config, files)?,
        ..Default::default()
    };
    let facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            root,
            plan.lint_suppression_paths.iter().cloned().collect(),
            Vec::new(),
            plan,
            None,
            Arc::clone(&sources),
        );
    check_prepared(root, config, files, &sources, Some(&facts), false)
}

pub(crate) fn check_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    facts: Option<&CheckFactMap>,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    let (root, files) = normalized_inputs(root, files);
    let root = root.as_path();
    let files = files.as_slice();
    let batches: Result<Vec<Vec<RuleFinding>>> = config.rule_applications(RULE_ID).into_par_iter().map(|rule| {
        let options: options::Options = rule.try_rule_options()?;
        let matcher = options.validate()?;
        let files = filter_rule_files(root, config, rule, files)?;
        let mut findings = Vec::new();
        for path in files.iter().filter(|path| crate::codebase::dependencies::extract::is_indexable(path)
            && matcher.is_match(&relative_slash_path(root, path))) {
            let file = relative_slash_path(root, path);
            let comments = facts.and_then(|facts| facts.lint_suppressions(path)).with_context(|| format!("{RULE_ID}: missing prepared comment facts for {file}"))?;
            for comment in comments.iter().filter(|c| options.enabled(&c.directive)) {
                let targets: Vec<&str> = if comment.directive.starts_with('@') { vec![&comment.directive] }
                    else if comment.rules.is_empty() && options.ban_blanket_disable { vec!["*"] }
                    else { comment.rules.iter().filter(|v| options.banned(v)).map(String::as_str).collect() };
                for target in targets {
                    let explanation = options.message.as_deref().unwrap_or("These lint rules must remain enforced; remove the suppression and fix the underlying code.");
                    findings.push(RuleFinding { rule: RULE_ID.into(), file: file.clone(), line: comment.line,
                        message: format!("{file}:{}: `{}` disables forbidden rule `{target}`. {explanation}", comment.line, comment.directive),
                        import: None, target: Some(target.into()) });
                }
            }
        }
        if !defer_suppression { super::suppression::suppress_rule_findings_with_sources(root, &mut findings, sources); }
        Ok(findings)
    }).collect();
    let mut findings: Vec<_> = batches?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}
// Normalize at the rule boundary; strict prepared maps keep one canonical key.
fn normalized_inputs(root: &Path, files: &[PathBuf]) -> (PathBuf, Vec<PathBuf>) {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let files = files
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    (root, files)
}

#[cfg(test)]
mod tests;
