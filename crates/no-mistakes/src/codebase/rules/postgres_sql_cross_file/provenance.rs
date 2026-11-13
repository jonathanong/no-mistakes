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
mod classification;
mod classification_calls;
use classification::{classify, classify_value};
use classification_calls::classify_call;
mod text_tag;
pub(crate) use text_tag::is_text_tag;
mod bindings;
mod validated;
pub(crate) use validated::IdentifierReturns;
mod env;
mod identifier_holes;
mod identifier_positions;
mod interpolations;
mod outside;
mod search;
pub(crate) use bindings::{literal_array, Bindings};
pub(crate) use env::is_process_specifier;
use env::{env_from_chain, env_key, process_env_key};
pub(crate) use identifier_positions::{
    delimit_markers as delimit_identifier_markers, identifier_status, scans_identifier,
};
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
    call: &SqlCall<'_>,
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

mod helpers;
use helpers::{contains, named_provenance, value_call_at};

#[cfg(test)]
#[path = "provenance/coverage_boundary_tests.rs"]
mod coverage_boundary_tests;
#[cfg(test)]
mod tests;
