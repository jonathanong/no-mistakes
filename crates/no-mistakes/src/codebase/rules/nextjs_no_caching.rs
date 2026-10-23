mod rendering;
use rendering::{findings_for_file, project_finding};

use super::RuleFinding;
use crate::codebase::ts_source::{
    has_disable_comment, has_disable_file_comment, relative_slash_path,
};
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::{bail, Result};
pub(crate) use ast::{extract_program, finish_visitor, prepare_visitor};
use rayon::prelude::*;
use serde::Serialize;
use std::path::{Path, PathBuf};

mod ast;
pub(crate) mod effort;
pub(crate) mod low;
pub use effort::configure_effort_plan;
mod bindings;
mod cache_imports;
mod config;
mod const_values;
mod patterns;
mod visitor;
mod visitor_scope;
mod visitor_visit;
pub(crate) use visitor::NextjsCachingVisitor;

pub const RULE_ID: &str = "nextjs-no-caching";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NextjsCachingFinding {
    pub(crate) line: usize,
    pub(crate) message: String,
    #[serde(skip)]
    pub(crate) unanalyzable: bool,
}

#[cfg(not(test))]
macro_rules! record_aggregate_scope_check {
    () => {};
}

#[cfg(test)]
#[macro_use]
mod tests;

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    super::check_indexed_standalone(root, config, check_files)
}

pub(crate) fn diagnostics_with_facts_for_aggregate(
    root: &Path,
    config: &NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
    defer_suppression: bool,
) -> Result<super::analysis_policy::AnalysisDiagnostics> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut diagnostics = super::analysis_policy::AnalysisDiagnostics::default();
    let mut inferred_roots = inferred_roots.cloned().unwrap_or_default();
    for rule in config.rule_applications(RULE_ID) {
        let policy = config.effective_rule_analysis(rule)?;
        let low = policy.effort == crate::config::v2::Effort::Low;
        let target_roots =
            super::target_roots_with_inferred(&root, config, rule, &mut inferred_roots);
        let filter = super::path_filter::RulePathFilter::new_with_inferred(
            &root,
            config,
            rule,
            &mut inferred_roots,
        );
        let filter = filter?;
        for path in shared.files() {
            let Some(facts) = shared.ts.get(path) else {
                continue;
            };
            let selected = if low {
                &facts.nextjs_caching_low
            } else {
                &facts.nextjs_caching
            };
            // Complete empty facts cannot produce findings or advisories. Keep
            // incomplete facts on the ordered scope/error path below.
            if facts.source.is_some() && selected.as_ref().is_some_and(Vec::is_empty) {
                continue;
            }
            if !target_roots
                .iter()
                .any(|target_root| path.starts_with(target_root))
                || !{
                    record_aggregate_scope_check!();
                    filter.is_match(path)
                }
            {
                continue;
            }
            if facts.parse_error.is_some() {
                continue;
            }
            let Some(source) = facts.source.as_ref() else {
                bail!("{} requires source facts for {}", RULE_ID, path.display());
            };
            let Some(cache_facts) = selected.as_ref() else {
                bail!(
                    "{} requires Next.js caching facts for {}",
                    RULE_ID,
                    path.display()
                );
            };
            let file = relative_slash_path(&root, path);
            if !defer_suppression && has_disable_file_comment(source, RULE_ID) {
                continue;
            }
            for raw in cache_facts.iter().filter(|raw| {
                defer_suppression || !has_disable_comment(source, raw.line as u32, RULE_ID)
            }) {
                let finding = project_finding(&file, raw);
                if raw.unanalyzable {
                    diagnostics.unanalyzable(policy.unanalyzable, finding);
                } else {
                    diagnostics.findings.push(finding);
                }
            }
        }
    }
    super::sort_findings(&mut diagnostics.findings);
    super::sort_findings(&mut diagnostics.advisories);
    Ok(diagnostics)
}

fn check_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    let mut inferred_roots = crate::codebase::config::InferredRoots::default();
    for rule in config.rule_applications(RULE_ID) {
        let policy = config.effective_rule_analysis(rule)?;
        let target_roots =
            super::target_roots_with_inferred(root, config, rule, &mut inferred_roots);
        let filter = super::path_filter::RulePathFilter::new_with_inferred(
            root,
            config,
            rule,
            &mut inferred_roots,
        );
        let filter = filter?;
        findings.extend(
            files
                .par_iter()
                .filter(|path| {
                    target_roots
                        .iter()
                        .any(|target_root| path.starts_with(target_root))
                        && filter.is_match(path)
                })
                .flat_map(|path| {
                    let Ok(source) = std::fs::read_to_string(path) else {
                        return Vec::new();
                    };
                    let Ok(cache_facts) = (if policy.effort == crate::config::v2::Effort::Low {
                        crate::ast::with_program(path, &source, |program, _| {
                            low::extract_program(path, &source, program)
                        })
                    } else {
                        extract(path, &source)
                    }) else {
                        return Vec::new();
                    };
                    let selected = cache_facts
                        .into_iter()
                        .filter(|fact| {
                            !fact.unanalyzable
                                || policy.unanalyzable == crate::config::v2::Unanalyzable::Report
                        })
                        .collect::<Vec<_>>();
                    findings_for_file(root, path, &source, &selected, false)
                })
                .collect::<Vec<_>>(),
        );
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

pub(crate) fn extract(path: &Path, source: &str) -> Result<Vec<NextjsCachingFinding>> {
    crate::ast::with_program(path, source, |program, _| {
        extract_program(path, source, program)
    })
}
