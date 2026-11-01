use super::validators::is_finite_validator;
use super::SqlCall;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{
    unique_closed_collection, ImportedBindingKind, RegistryKind, ValueCall, ValueExpression,
};
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use std::collections::BTreeSet;
use std::path::Path;

mod binding_tag;
mod bindings;
mod env;
mod identifier_holes;
mod identifier_positions;
mod interpolations;
mod outside;
mod search;
pub(crate) use bindings::Bindings;
pub(crate) use env::is_process_specifier;
use env::{env_from_chain, env_key, process_env_key};
pub(crate) use identifier_positions::scans_identifier;
pub(crate) use interpolations::interpolations;

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

pub(crate) fn identifier_sources<'a>(
    graph: &DepGraph,
    facts: &'a CheckFactMap,
    call: &SqlCall<'a>,
    bindings: &Bindings<'a>,
) -> BTreeSet<Provenance> {
    if let Some((holes, outside_holes)) = identifier_holes::split(facts, call, bindings) {
        let mut out = BTreeSet::new();
        for hole in holes {
            classify(graph, facts, &call.file, hole, &mut out, bindings);
        }
        outside::retain(graph, facts, call, bindings, &outside_holes, &mut out);
        return out;
    }
    interpolations(graph, facts, call, bindings)
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
    if binding_tag::is_bind_tag(bindings, file, value_call)
        || is_finite_validator(graph, facts, &bindings.routes, file, &value_call.callee)
        || closed_map_lookup(facts, file, value_call)
    {
        return;
    }
    classify(graph, facts, file, &value_call.callee, out, bindings);
    for argument in value_call.arguments.iter() {
        classify(graph, facts, file, argument, out, bindings);
    }
}

fn closed_map_lookup(facts: &CheckFactMap, file: &Path, value_call: &ValueCall) -> bool {
    let ValueExpression::Member(inner, method) = &value_call.callee else {
        return false;
    };
    if method != "get" {
        return false;
    }
    let ValueExpression::Named { name, .. } = inner.as_ref() else {
        return false;
    };
    facts
        .get_ts_facts(file)
        .is_some_and(|ts| unique_closed_collection(&ts.value_flow, name, RegistryKind::Map))
}

mod helpers;
use helpers::{contains, named_provenance, value_call_at};

#[cfg(test)]
mod tests;
