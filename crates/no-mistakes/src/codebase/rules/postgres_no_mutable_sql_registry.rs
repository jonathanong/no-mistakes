//! Cross-file member writes that reach SQL identifier interpolations.
use super::postgres_sql_cross_file::{
    check_applications, count, finding, identifier_placeholder, interpolations, sql_calls,
    unique_export, Provenance, SqlCall,
};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

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
        for provenance in interpolations(&call) {
            let Provenance::Named(name) = provenance else {
                continue;
            };
            if let Some(finding) = registry_finding(root, graph, facts, &writes, &call, &name) {
                sinks += 1;
                findings.push(finding);
            }
        }
    }
    count("postgres.mutable_sql_registry.candidates", candidates);
    count("postgres.mutable_sql_registry.sinks", sinks);
    findings
}

struct MutatedExport {
    producer: PathBuf,
    exported: String,
    writer: PathBuf,
}

fn mutated_exports(graph: &DepGraph, facts: &CheckFactMap) -> Vec<MutatedExport> {
    let mut out = Vec::new();
    for writer in facts.graph_file_universe() {
        let Some(ts) = facts.get_ts_facts(writer) else {
            continue;
        };
        if ts.parse_error.is_some() || ts.operational_error.is_some() {
            continue;
        }
        for binding in &ts.imported_bindings {
            if binding.is_type_only || !member_write(facts, writer, &binding.local) {
                continue;
            }
            let Some(unique) = unique_export(graph, facts, writer, &binding.local) else {
                continue;
            };
            if unique.producer == *writer {
                continue;
            }
            out.push(MutatedExport {
                producer: unique.producer,
                exported: unique.exported,
                writer: writer.clone(),
            });
        }
    }
    out.sort_by(|left, right| {
        left.producer
            .cmp(&right.producer)
            .then(left.exported.cmp(&right.exported))
            .then(left.writer.cmp(&right.writer))
    });
    out.dedup_by(|left, right| {
        left.producer == right.producer
            && left.exported == right.exported
            && left.writer == right.writer
    });
    out
}

#[cfg(test)]
mod tests;

fn member_write(facts: &CheckFactMap, file: &Path, local: &str) -> bool {
    let Some(ts) = facts.get_ts_facts(file) else {
        return false;
    };
    let Some(histories) = &ts.value_flow.receiver_mutations else {
        return false;
    };
    histories.values().any(|by_name| {
        by_name.get(local).is_some_and(|history| {
            // Exact named writes live on `assigned.members`. Dynamic-only keys
            // stay on `assigned.dynamic` and must abstain.
            history
                .assigned
                .as_ref()
                .is_some_and(|assigned| !assigned.members.is_empty())
        })
    })
}

fn registry_finding(
    root: &Path,
    graph: &DepGraph,
    facts: &CheckFactMap,
    writes: &[MutatedExport],
    call: &SqlCall<'_>,
    name: &str,
) -> Option<RuleFinding> {
    let unique = unique_export(graph, facts, &call.file, name)?;
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
