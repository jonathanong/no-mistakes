use super::validators::is_finite_validator;
use super::SqlCall;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{ImportedBindingKind, ValueCall, ValueExpression};
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use std::collections::BTreeSet;
use std::path::Path;

mod env;
pub(crate) use env::is_process_specifier;
use env::{env_from_chain, env_key, process_env_key};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Provenance {
    ProcessEnv(String),
    Named(String),
    Namespace { local: String, exported: String },
}

pub(crate) fn interpolations(
    graph: &DepGraph,
    facts: &CheckFactMap,
    call: &SqlCall<'_>,
) -> BTreeSet<Provenance> {
    let mut out = BTreeSet::new();
    let Some(ts) = facts.get_ts_facts(&call.file) else {
        return out;
    };
    for value_call in &ts.value_flow.calls {
        if !contains(call.span, (value_call.offset, value_call.end)) {
            continue;
        }
        if (value_call.offset, value_call.end) == call.span {
            for argument in value_call.arguments.iter() {
                classify(graph, facts, &call.file, argument, &mut out);
            }
            continue;
        }
        classify_call(graph, facts, &call.file, value_call, &mut out);
    }
    out
}

pub(crate) fn is_process_env(expr: &ValueExpression) -> bool {
    process_env_key(expr).is_some()
}

fn classify(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    expr: &ValueExpression,
    out: &mut BTreeSet<Provenance>,
) {
    if let Some(key) = env_key(facts, file, expr) {
        out.insert(Provenance::ProcessEnv(key.to_string()));
        return;
    }
    match expr {
        ValueExpression::Call(start, end) => {
            if let Some(value_call) = value_call_at(facts, file, *start, *end) {
                classify_call(graph, facts, file, value_call, out);
            }
        }
        ValueExpression::Awaited(inner) => classify(graph, facts, file, inner, out),
        ValueExpression::Member(inner, key) => {
            let mut members = vec![key.clone()];
            let mut current = inner.as_ref();
            while let ValueExpression::Member(next, next_key) = current {
                members.push(next_key.clone());
                current = next.as_ref();
            }
            members.reverse();
            if let Some(key) = env_from_chain(facts, file, current, &members) {
                out.insert(Provenance::ProcessEnv(key));
            } else if let ValueExpression::Named { name, .. } = current {
                out.insert(named_provenance(facts, file, name, &members));
            } else {
                classify(graph, facts, file, current, out);
            }
        }
        ValueExpression::Nullish(left, right)
        | ValueExpression::Default(left, right)
        | ValueExpression::Either(left, right) => {
            classify(graph, facts, file, left, out);
            classify(graph, facts, file, right, out);
        }
        ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => {
            for value in fields.values() {
                classify(graph, facts, file, value, out);
            }
        }
        ValueExpression::Named { name, .. } => {
            out.insert(named_provenance(facts, file, name, &[]));
        }
        _ => {}
    }
}

fn classify_call(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    value_call: &ValueCall,
    out: &mut BTreeSet<Provenance>,
) {
    if is_finite_validator(graph, facts, file, &value_call.callee) {
        return;
    }
    classify(graph, facts, file, &value_call.callee, out);
    for argument in value_call.arguments.iter() {
        classify(graph, facts, file, argument, out);
    }
}

fn named_provenance(
    facts: &CheckFactMap,
    file: &Path,
    local: &str,
    members: &[String],
) -> Provenance {
    if is_namespace_import(facts, file, local) {
        if let Some(exported) = members.first() {
            return Provenance::Namespace {
                local: local.to_string(),
                exported: exported.clone(),
            };
        }
    }
    Provenance::Named(local.to_string())
}

fn is_namespace_import(facts: &CheckFactMap, file: &Path, local: &str) -> bool {
    facts.get_ts_facts(file).is_some_and(|ts| {
        ts.imported_bindings.iter().any(|binding| {
            !binding.is_type_only
                && binding.local == local
                && binding.kind == ImportedBindingKind::Namespace
        })
    })
}

fn value_call_at<'a>(
    facts: &'a CheckFactMap,
    file: &Path,
    start: u32,
    end: u32,
) -> Option<&'a ValueCall> {
    facts
        .get_ts_facts(file)?
        .value_flow
        .calls
        .iter()
        .find(|call| call.offset == start && call.end == end)
}

fn contains(outer: (u32, u32), inner: (u32, u32)) -> bool {
    inner.0 >= outer.0 && inner.1 <= outer.1
}

#[cfg(test)]
mod tests;
