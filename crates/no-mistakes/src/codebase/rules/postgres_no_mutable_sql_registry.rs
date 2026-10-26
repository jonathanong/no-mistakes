//! Cross-file member writes that reach SQL identifier interpolations.
use super::postgres_sql_cross_file::{
    count, finding, identifier_placeholder, interpolations, registry_route, sql_calls,
    unique_export, unique_namespace_member, Bindings, Provenance, RegistryRoutes, SqlCall,
};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::DepGraph;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;

mod write_index;
mod writes;
use writes::{mutated_exports, MutatedExport};

pub const RULE_ID: &str = "postgres-no-mutable-sql-registry";

pub(crate) fn check_with_graph(
    root: &Path,
    config: &NoMistakesConfig,
    graph: Option<&DepGraph>,
    facts: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut findings = Vec::new();
    for (index, application) in config.rules.iter().enumerate().filter(|(_, rule)| {
        rule.enabled && rule.rule == RULE_ID && config.rule_has_effective_target(rule)
    }) {
        let policy = config.effective_rule_analysis(application)?;
        if policy.effort == crate::config::v2::Effort::Low {
            findings.extend(super::postgres_sql_cross_file::low::check(
                &root,
                config,
                application,
                index,
                facts,
                RULE_ID,
            )?);
        } else {
            findings.extend(super::postgres_sql_cross_file::check_application(
                &root,
                config,
                application,
                graph.expect("postgres-no-mutable-sql-registry requires canonical graph"),
                facts,
                RULE_ID,
                |root, opts, graph, facts| Ok(scan(root, opts, graph, facts)),
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
) -> Vec<RuleFinding> {
    let bindings = Bindings::new(facts);
    let writes = mutated_exports(graph, facts, &bindings.routes);
    count(
        "postgres.mutable_sql_registry.producers",
        writes.len() as u64,
    );
    let mut findings = Vec::new();
    let mut candidates = 0;
    let mut sinks = 0;
    let calls = sql_calls(root, opts, facts);
    for call in &calls {
        candidates += 1;
        if identifier_placeholder(call.call) != Some(true) {
            continue;
        }
        for provenance in interpolations(graph, facts, call, &bindings) {
            if let Some(finding) = registry_finding(
                root,
                graph,
                facts,
                &bindings.routes,
                &writes,
                call,
                &provenance,
            ) {
                sinks += 1;
                findings.push(finding);
            }
        }
    }
    count("postgres.mutable_sql_registry.candidates", candidates);
    count("postgres.mutable_sql_registry.sinks", sinks);
    findings
}

#[cfg(all(test, feature = "test-instrumentation"))]
mod index_tests;
#[cfg(test)]
mod tests;

fn registry_finding(
    root: &Path,
    graph: &DepGraph,
    facts: &CheckFactMap,
    routes: &RegistryRoutes<'_>,
    writes: &[MutatedExport],
    call: &SqlCall<'_>,
    provenance: &Provenance,
) -> Option<RuleFinding> {
    let (unique, name) = match provenance {
        Provenance::Reference { name, members, .. } => (
            registry_route(graph, routes, &call.file, name, members)
                .or_else(|| unique_export(graph, facts, routes, &call.file, name))?,
            name.as_str(),
        ),
        Provenance::Namespace {
            local,
            exported,
            members,
        } => (
            registry_route(graph, routes, &call.file, local, members)
                .or_else(|| unique_namespace_member(graph, facts, &call.file, local, exported))?,
            local.as_str(),
        ),
        Provenance::ProcessEnv(_) => return None,
    };
    let matched = writes
        .iter()
        .find(|write| write.producer == unique.producer && write.exported == unique.exported)?;
    let producer = relative_slash_path(root, &matched.producer);
    let writer = relative_slash_path(root, &matched.writer);
    Some(finding(
        root,
        RULE_ID,
        &call.file,
        call.call.line,
        format!(
            "Cross-file write to exported registry `{name}` in {producer} ({writer}) reaches SQL identifier interpolation in this executor call. Member assignment after export is not a declared literal map. Consolidate the literal registry in {producer} and import it; const does not freeze members."
        ),
        Some(name.to_string()),
        Some(producer),
    ))
}
