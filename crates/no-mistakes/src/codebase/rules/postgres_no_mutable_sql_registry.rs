//! Cross-file member writes that reach SQL identifier interpolations.
use super::postgres_sql_cross_file::{
    check_applications, count, finding, identifier_placeholder, interpolations, sql_calls,
    unique_export, unique_namespace_member, Provenance, SqlCall,
};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::DepGraph;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;

mod writes;
#[cfg(test)]
use writes::member_write;
use writes::{mutated_exports, MutatedExport};

pub const RULE_ID: &str = "postgres-no-mutable-sql-registry";

pub(crate) fn check_with_graph(
    root: &Path,
    config: &NoMistakesConfig,
    graph: &DepGraph,
    facts: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    check_applications(
        root,
        config,
        graph,
        facts,
        RULE_ID,
        |root, opts, graph, facts| Ok(scan(root, opts, graph, facts)),
    )
}

fn scan(
    root: &Path,
    opts: &super::postgres_sql_cross_file::CompiledOptions,
    graph: &DepGraph,
    facts: &CheckFactMap,
) -> Vec<RuleFinding> {
    let writes = mutated_exports(graph, facts);
    count(
        "postgres.mutable_sql_registry.producers",
        writes.len() as u64,
    );
    let mut findings = Vec::new();
    let mut candidates = 0;
    let mut sinks = 0;
    for call in sql_calls(root, opts, facts) {
        candidates += 1;
        if identifier_placeholder(call.call) != Some(true) {
            continue;
        }
        for provenance in interpolations(graph, facts, &call) {
            if let Some(finding) = registry_finding(root, graph, facts, &writes, &call, &provenance)
            {
                sinks += 1;
                findings.push(finding);
            }
        }
    }
    count("postgres.mutable_sql_registry.candidates", candidates);
    count("postgres.mutable_sql_registry.sinks", sinks);
    findings
}

#[cfg(test)]
mod tests;

fn registry_finding(
    root: &Path,
    graph: &DepGraph,
    facts: &CheckFactMap,
    writes: &[MutatedExport],
    call: &SqlCall<'_>,
    provenance: &Provenance,
) -> Option<RuleFinding> {
    let (unique, name) = match provenance {
        Provenance::Named(name) => (
            unique_export(graph, facts, &call.file, name)?,
            name.as_str(),
        ),
        Provenance::Namespace { local, exported } => (
            unique_namespace_member(graph, facts, &call.file, local, exported)?,
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
