use super::validators::is_finite_validator;
use super::SqlCall;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{ImportedBindingKind, ValueCall, ValueExpression};
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use std::collections::BTreeSet;
use std::path::Path;

mod bindings;
mod env;
mod search;
pub(crate) use bindings::Bindings;
pub(crate) use env::is_process_specifier;
use env::{env_from_chain, env_key, process_env_key};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Provenance {
    ProcessEnv(String),
    Reference {
        name: String,
        scope: usize,
        offset: u32,
        members: Vec<String>,
    },
    Namespace {
        local: String,
        exported: String,
        members: Vec<String>,
    },
}

pub(crate) fn interpolations(
    graph: &DepGraph,
    facts: &CheckFactMap,
    call: &SqlCall<'_>,
    bindings: &Bindings<'_>,
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
                classify(graph, facts, &call.file, argument, &mut out, bindings);
            }
            continue;
        }
        classify_call(graph, facts, &call.file, value_call, &mut out, bindings);
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
    bindings: &Bindings<'_>,
) {
    if let Some(key) = env_key(facts, file, expr).map(str::to_owned).or_else(|| {
        matches!(expr, ValueExpression::Member(_, _))
            .then(|| bindings.env(file, expr, &[]))
            .flatten()
    }) {
        out.insert(Provenance::ProcessEnv(key.to_string()));
        return;
    }
    match expr {
        ValueExpression::Derived(parts) => {
            for part in parts.iter() {
                classify(graph, facts, file, part, out, bindings);
            }
        }
        ValueExpression::Call(start, end) => {
            if let Some(value_call) = value_call_at(facts, file, *start, *end) {
                classify_call(graph, facts, file, value_call, out, bindings);
            }
        }
        ValueExpression::Awaited(inner) => classify(graph, facts, file, inner, out, bindings),
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
            } else if let ValueExpression::Named {
                name,
                scope,
                offset,
                ..
            } = current
            {
                out.insert(named_provenance(
                    facts, file, name, *scope, *offset, &members,
                ));
            } else {
                classify(graph, facts, file, current, out, bindings);
            }
        }
        ValueExpression::Nullish(left, right)
        | ValueExpression::Default(left, right)
        | ValueExpression::Either(left, right) => {
            classify(graph, facts, file, left, out, bindings);
            classify(graph, facts, file, right, out, bindings);
        }
        ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => {
            for value in fields.values() {
                classify(graph, facts, file, value, out, bindings);
            }
        }
        ValueExpression::Named {
            name,
            scope,
            offset,
            ..
        } => {
            out.insert(named_provenance(facts, file, name, *scope, *offset, &[]));
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
    bindings: &Bindings<'_>,
) {
    if is_finite_validator(graph, facts, file, &value_call.callee) {
        return;
    }
    classify(graph, facts, file, &value_call.callee, out, bindings);
    for argument in value_call.arguments.iter() {
        classify(graph, facts, file, argument, out, bindings);
    }
}

mod helpers;
use helpers::{contains, named_provenance, value_call_at};

#[cfg(test)]
mod tests;
