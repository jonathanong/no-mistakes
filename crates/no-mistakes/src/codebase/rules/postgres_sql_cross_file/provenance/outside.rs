//! Holes that sit outside an identifier position.
use super::super::SqlCall;
use super::bindings::Bindings;
use super::classify_value;
use super::Provenance;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::ValueExpression;
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use crate::fx::FxHashSet;
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn retain(
    graph: &DepGraph,
    facts: &CheckFactMap,
    call: &SqlCall<'_>,
    bindings: &Bindings<'_>,
    holes: &[&ValueExpression],
    out: &mut BTreeSet<Provenance>,
) {
    for hole in holes {
        let mut extra = BTreeSet::new();
        classify_value(graph, facts, &call.file, hole, &mut extra, bindings);
        out.extend(
            extra
                .into_iter()
                .filter(|item| retained(bindings, &call.file, item)),
        );
    }
}

/// Plain parameters and immutable forwarding aliases are known bind values.
fn retained(bindings: &Bindings<'_>, file: &Path, item: &Provenance) -> bool {
    match item {
        Provenance::Reference {
            name,
            scope,
            offset,
            members,
        } => {
            !bindings.plain_parameter(file, *scope, name)
                && !(members.is_empty() && bind_alias(bindings, file, *scope, name, *offset))
        }
        _ => true,
    }
}

fn bind_alias(bindings: &Bindings<'_>, file: &Path, scope: usize, name: &str, at: u32) -> bool {
    let mut seen = FxHashSet::default();
    let (mut scope, mut name, mut at) = (scope, name, at);
    while let Some(binding) = bindings.value(file, scope, name, at) {
        if !binding.immutable || !seen.insert((scope, name, binding.declared_at)) {
            return false;
        }
        let ValueExpression::Named {
            name: next,
            scope: next_scope,
            offset,
            ..
        } = &binding.value
        else {
            return false;
        };
        (scope, name, at) = (next_scope.unwrap_or(0), next.as_str(), *offset);
    }
    bindings.plain_parameter(file, scope, name)
        && bindings.facts.get_ts_facts(file).is_some_and(|ts| {
            ts.value_flow.parameters.iter().any(|parameter| {
                parameter.scope == scope
                    && parameter.name == name
                    && parameter.projection.is_empty()
                    && parameter.invalidated_at.is_none_or(|write| write > at)
            })
        })
}

#[cfg(test)]
mod harness;
#[cfg(test)]
mod tests;
