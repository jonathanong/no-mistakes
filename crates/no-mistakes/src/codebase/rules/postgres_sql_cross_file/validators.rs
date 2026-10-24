use super::{unique_export, RegistryRoutes};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{CallableId, ValueExpression};
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::Path;

pub(crate) fn is_finite_validator(
    graph: &DepGraph,
    facts: &CheckFactMap,
    routes: &RegistryRoutes<'_>,
    file: &Path,
    callee: &ValueExpression,
) -> bool {
    let ValueExpression::Named { name, .. } = callee else {
        return false;
    };
    callable_returns_literals(facts, file, name)
        || unique_export(graph, facts, routes, file, name).is_some_and(|unique| {
            unique.producer != *file
                && callable_returns_literals(facts, &unique.producer, &unique.exported)
        })
}

fn callable_returns_literals(facts: &CheckFactMap, file: &Path, name: &str) -> bool {
    let Some(ts) = facts.get_ts_facts(file) else {
        return false;
    };
    if ts.parse_error.is_some() || ts.operational_error.is_some() {
        return false;
    }
    let ids: Vec<CallableId> = ts
        .callable_scope_ids
        .iter()
        .filter(|(_, scope)| scope.as_ref() == name || scope.ends_with(&format!(".{name}")))
        .map(|(id, _)| *id)
        .collect();
    if ids.iter().any(|id| returns_literals(ts, *id)) {
        return true;
    }
    ts.value_flow.bindings.iter().any(|binding| {
        binding.owner.is_none()
            && binding.name == name
            && matches!(&binding.value, ValueExpression::Callable(id) if returns_literals(ts, *id))
    })
}

fn returns_literals(ts: &TsFileFacts, id: CallableId) -> bool {
    let mut any = false;
    ts.value_flow
        .returns
        .iter()
        .filter(|(owner, _)| *owner == id)
        .all(|(_, value)| {
            any = true;
            matches!(
                value,
                ValueExpression::Unknown | ValueExpression::Undefined | ValueExpression::Null
            )
        })
        && any
}

#[cfg(test)]
mod tests;
