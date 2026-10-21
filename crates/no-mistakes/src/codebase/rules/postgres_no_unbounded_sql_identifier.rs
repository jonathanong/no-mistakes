//! Cross-file env/config identifier interpolations without a finite validator.
use super::postgres_sql_cross_file::{
    check_applications, count, finding, identifier_placeholder, interpolations, is_process_env,
    sql_calls, unique_export, walk_expression, Provenance, SqlCall,
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
    let mut findings = Vec::new();
    let mut candidates = 0;
    let mut sinks = 0;
    for call in sql_calls(root, opts, facts) {
        candidates += 1;
        if identifier_placeholder(call.call) != Some(true) {
            continue;
        }
        for provenance in interpolations(&call) {
            if let Some(finding) = identifier_finding(root, graph, facts, &call, &provenance) {
                sinks += 1;
                findings.push(finding);
            }
        }
    }
    count("postgres.unbounded_sql_identifier.candidates", candidates);
    count("postgres.unbounded_sql_identifier.sinks", sinks);
    findings
}

fn identifier_finding(
    root: &Path,
    graph: &DepGraph,
    facts: &CheckFactMap,
    call: &SqlCall<'_>,
    provenance: &Provenance,
) -> Option<RuleFinding> {
    match provenance {
        Provenance::ProcessEnv(key) => {
            if shadowed_process(facts, &call.file) {
                return None;
            }
            Some(message(
                root,
                call,
                &format!("process.env.{key}"),
                Some(format!("process.env.{key}")),
            ))
        }
        Provenance::Named(name) => {
            if local_env_alias(facts, &call.file, name) {
                return Some(message(root, call, "process.env", Some(name.clone())));
            }
            let unique = unique_export(graph, facts, &call.file, name)?;
            if unique.producer == call.file
                || !export_is_unbounded(facts, &unique.producer, &unique.exported)
            {
                return None;
            }
            let producer = relative_slash_path(root, &unique.producer);
            Some(message(root, call, &producer, Some(name.clone())))
        }
    }
}

fn message(root: &Path, call: &SqlCall<'_>, producer: &str, import: Option<String>) -> RuleFinding {
    finding(
        root,
        RULE_ID,
        &call.file,
        call.call.line,
        format!(
            "Unbounded identifier source `{producer}` reaches SQL relation interpolation in this executor call without a runtime finite-domain validator. Map accepted values to literal schemas with a switch whose default throws; a type assertion is not validation. Bound values remain allowed."
        ),
        import,
        Some(producer.to_string()),
    )
}

fn shadowed_process(facts: &CheckFactMap, file: &Path) -> bool {
    facts.get_ts_facts(file).is_some_and(|ts| {
        ts.imported_bindings
            .iter()
            .any(|binding| !binding.is_type_only && binding.local == "process")
    })
}

#[cfg(test)]
mod tests;

fn local_env_alias(facts: &CheckFactMap, file: &Path, name: &str) -> bool {
    facts.get_ts_facts(file).is_some_and(|ts| {
        ts.value_flow
            .bindings
            .iter()
            .any(|binding| binding.name == name && expression_unbounded(&binding.value))
    })
}

fn export_is_unbounded(facts: &CheckFactMap, producer: &Path, exported: &str) -> bool {
    let Some(ts) = facts.get_ts_facts(producer) else {
        return false;
    };
    ts.value_flow.bindings.iter().any(|binding| {
        binding.owner.is_none() && binding.name == exported && expression_unbounded(&binding.value)
    })
}

fn expression_unbounded(expr: &ValueExpression) -> bool {
    let mut found = false;
    walk_expression(expr, &mut |value| {
        found |= is_process_env(value);
    });
    found
}
