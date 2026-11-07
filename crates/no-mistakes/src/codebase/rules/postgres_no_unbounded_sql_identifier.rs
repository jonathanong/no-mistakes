//! Cross-file env/config identifier interpolations without a finite validator.
use super::postgres_sql_cross_file::{
    count, finding, identifier_sources, is_process_env, is_process_specifier, scans_identifier,
    sql_calls, unique_export, unique_namespace_member, walk_expression, Bindings, Provenance,
    RegistryRoutes, SqlCall,
};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::ValueExpression;
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;

pub const RULE_ID: &str = "postgres-no-unbounded-sql-identifier";

pub(crate) fn check_with_graph(
    root: &Path,
    config: &NoMistakesConfig,
    graph: Option<&DepGraph>,
    facts: &CheckFactMap,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) -> Result<Vec<RuleFinding>> {
    let normalized = crate::codebase::ts_resolver::normalize_path(root);
    let root = normalized.as_path();
    let mut findings = Vec::new();
    for (index, application) in config.rules.iter().enumerate().filter(|(_, rule)| {
        rule.enabled && rule.rule == RULE_ID && config.rule_has_effective_target(rule)
    }) {
        let policy = config.effective_rule_analysis(application)?;
        if policy.effort == crate::config::v2::Effort::Low {
            findings.extend(super::postgres_sql_cross_file::low::check(
                root,
                config,
                application,
                index,
                facts,
                RULE_ID,
            )?);
        } else {
            findings.extend(application::check(
                root,
                config,
                application,
                graph.expect("postgres-no-unbounded-sql-identifier requires canonical graph"),
                facts,
                sources,
            )?);
        }
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(
    root: &Path,
    opts: &super::postgres_sql_cross_file::CompiledOptions,
    graph: &DepGraph,
    facts: &CheckFactMap,
    calls: &[SqlCall<'_>],
    returns: &super::postgres_sql_cross_file::IdentifierReturns,
    filter: &super::path_filter::RulePathFilter,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    let mut candidates = 0;
    let mut sinks = 0;
    let bindings = Bindings::with_calls(facts, calls)
        .with_graph(graph)
        .with_trusted_tags(&opts.embedded.trusted_sql_tags)
        .with_numeric_parameter_bounds()
        .with_identifier_returns(returns);
    let mut fragment_proofs = fragments::Proofs::new(graph, opts, facts, &bindings);
    let catalog_proof = catalogs::CatalogProof::new(graph, facts, &bindings);
    for call in calls {
        if !filter.is_match(&call.file) {
            continue;
        }
        candidates += 1;
        if !scans_identifier(call.call) {
            continue;
        }
        for provenance in identifier_sources(graph, facts, call, &bindings) {
            if closed::proven(graph, opts, facts, &bindings, call, &provenance)
                || catalog_proof.proven(call, &provenance)
                || closed_calls::proven(graph, facts, &bindings, call, &provenance)
                || fragment_proofs.proven(call, &provenance)
            {
                continue;
            }
            if let Provenance::Reference {
                name,
                scope,
                offset,
                members,
            } = &provenance
            {
                if bindings.env_alias(&call.file, *scope, name, *offset)
                    || bindings.flows_unbounded(&call.file, *scope, name, *offset)
                    || (!bindings.row_member_overwritten(
                        &call.file, *scope, name, *offset, members, call.span,
                    ) && bindings.query_projection(&call.file, *scope, name, *offset))
                {
                    sinks += 1;
                    findings.push(message(root, call, name, Some(name.clone())));
                    continue;
                }
            }
            if let Provenance::Reference { name, scope, .. } = &provenance {
                if bindings.closed_parameter(&call.file, *scope, name) {
                    continue;
                }
            }
            if let Some(finding) =
                identifier_finding(root, graph, facts, &bindings.routes, call, &provenance)
            {
                sinks += 1;
                findings.push(finding);
            }
        }
    }
    count("postgres.unbounded_sql_identifier.candidates", candidates);
    count("postgres.unbounded_sql_identifier.sinks", sinks);
    findings
}

mod application;
mod findings;
#[cfg(test)]
mod tests;
pub(super) use findings::diagnostic_message;
use findings::{identifier_finding, message};
mod catalogs;
mod closed;
mod closed_calls;
mod local_sources;
use local_sources::{
    imported_unbounded, local_env_alias, parameter, query_result_binding, shadowed_process,
};

mod fragments;
#[cfg(test)]
mod fragments_tests;
