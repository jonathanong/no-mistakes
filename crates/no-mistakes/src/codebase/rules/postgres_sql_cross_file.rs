//! Shared prepared-fact helpers for cross-file SQL identifier provenance.
mod expressions;
mod graph;
mod options;
mod provenance;
mod sql;
mod validators;

pub(crate) use expressions::walk_expression;
pub(crate) use graph::{graph_plan, unique_export, unique_namespace_member, UniqueExport};
pub(crate) use options::{compile, CompiledOptions, Options};
pub(crate) use provenance::{interpolations, is_process_env, is_process_specifier, Provenance};
pub(crate) use sql::{identifier_placeholder, sql_calls, SqlCall};

use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::DepGraph;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;

pub(crate) fn enabled(config: &NoMistakesConfig) -> bool {
    config.rule_configured("postgres-no-mutable-sql-registry")
        || config.rule_configured("postgres-no-unbounded-sql-identifier")
}

pub(crate) fn count(metric: &'static str, amount: u64) {
    if amount == 0 {
        return;
    }
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment(metric, amount);
    }
}

pub(crate) fn finding(
    root: &Path,
    rule: &str,
    file: &Path,
    line: u32,
    message: String,
    import: Option<String>,
    target: Option<String>,
) -> RuleFinding {
    RuleFinding {
        rule: rule.to_string(),
        file: relative_slash_path(root, file),
        line: line as usize,
        message,
        import,
        target,
    }
}

pub(crate) fn check_applications(
    root: &Path,
    config: &NoMistakesConfig,
    graph: &DepGraph,
    facts: &CheckFactMap,
    rule_id: &str,
    visit: impl Fn(&Path, &CompiledOptions, &DepGraph, &CheckFactMap) -> Result<Vec<RuleFinding>>,
) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut findings = Vec::new();
    for application in config.rule_applications(rule_id) {
        let opts: Options = application.try_rule_options()?;
        let compiled = compile(rule_id, &opts)?;
        let filter = super::path_filter::RulePathFilter::new(&root, config, application)?;
        let mut next = visit(&root, &compiled, graph, facts)?;
        next.retain(|finding| filter.is_match(&root.join(&finding.file)));
        findings.extend(next);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

#[cfg(test)]
mod tests;
