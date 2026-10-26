//! Shared prepared-fact helpers for cross-file SQL identifier provenance.
mod expressions;
mod graph;
pub(crate) mod low;
mod options;
mod provenance;
mod sql;
mod validators;

pub(crate) use expressions::walk_expression;
pub(crate) use graph::{
    graph_plan, registry_route, unique_export, unique_namespace_member, RegistryRoutes,
    UniqueExport,
};
pub(crate) use options::{compile, CompiledOptions, Options};
pub(crate) use provenance::{
    interpolations, is_process_env, is_process_specifier, Bindings, Provenance,
};
pub(crate) use sql::{identifier_placeholder, sql_calls, SqlCall};

use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::DepGraph;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;

pub(crate) fn needs_deep_analysis(config: &NoMistakesConfig) -> bool {
    [
        "postgres-no-mutable-sql-registry",
        "postgres-no-unbounded-sql-identifier",
    ]
    .iter()
    .any(|id| {
        config.rule_applications(id).iter().any(|rule| {
            config.effective_rule_analysis(rule).map_or(true, |policy| {
                policy.effort != crate::config::v2::Effort::Low
            })
        })
    })
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

#[cfg(test)]
mod tests;

/// Borrow one application without copying the request's prepared configuration.
pub(crate) fn check_application(
    root: &Path,
    config: &NoMistakesConfig,
    application: &crate::config::v2::schema::RuleDef,
    graph: &DepGraph,
    facts: &CheckFactMap,
    rule_id: &str,
    visit: impl Fn(&Path, &CompiledOptions, &DepGraph, &CheckFactMap) -> Result<Vec<RuleFinding>>,
) -> Result<Vec<RuleFinding>> {
    let opts: Options = application.try_rule_options()?;
    let compiled = compile(rule_id, &opts)?;
    let filter = super::path_filter::RulePathFilter::new(root, config, application)?;
    let mut next = visit(root, &compiled, graph, facts)?;
    next.retain(|finding| filter.is_match(&root.join(&finding.file)));
    Ok(next)
}
