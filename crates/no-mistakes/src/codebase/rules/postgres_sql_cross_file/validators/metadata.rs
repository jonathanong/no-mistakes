use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{
    CallableId, GuardKind, GuardSubject, InvocationKind, ValueExpression, ValueFlowFacts,
};
use crate::codebase::dependencies::graph::{DepGraph, NodeId, TsFactLookup};
use std::path::Path;

pub(crate) fn helper_call(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    value: &ValueExpression,
) -> bool {
    let offset = match value {
        ValueExpression::Named { offset, .. } | ValueExpression::Call(offset, _) => *offset,
        _ => return false,
    };
    let mut sites = graph
        .call_sites_in_file(file)
        .iter()
        .filter(|site| site.offset == offset && site.invocation == InvocationKind::Call);
    let Some(site) = sites.next() else {
        return false;
    };
    if sites.next().is_some() || graph.failed_call_sources(site).next().is_some() {
        return false;
    }
    let Some(NodeId::Symbol {
        file,
        callable_id: Some(owner),
        ..
    }) = &site.target_node
    else {
        return false;
    };
    let Some(ts) = facts.get_ts_facts(file.as_ref()) else {
        return false;
    };
    if ts.parse_error.is_some() || ts.operational_error.is_some() {
        return false;
    }
    let flow = &ts.value_flow;
    if flow.incomplete_reason.is_some()
        || flow.async_callables.contains(owner)
        || flow.generator_callables.contains(owner)
    {
        return false;
    }
    let mut returned = flow.returns.iter().filter(|(id, _)| id == owner).peekable();
    returned.peek().is_some() && returned.all(|(_, value)| closed_return(flow, *owner, value))
}

fn closed_return(flow: &ValueFlowFacts, owner: CallableId, value: &ValueExpression) -> bool {
    let ValueExpression::Member(base, member) = value else {
        return false;
    };
    let ValueExpression::Named {
        name,
        scope: Some(scope),
        offset,
        ..
    } = base.as_ref()
    else {
        return false;
    };
    let mut bindings = flow.bindings.iter().filter(|binding| {
        binding.name == *name
            && binding.scope == *scope
            && binding.owner == Some(owner)
            && binding.declared_at <= *offset
            && binding.invalidated_at.is_none_or(|end| *offset < end)
    });
    let Some(binding) = bindings.next() else {
        return false;
    };
    if bindings.next().is_some() || mutated(flow, name, *scope) {
        return false;
    }
    flow.bounded_bindings.iter().any(|bounded| {
        bounded.requires_guard
            && bounded.name == *name
            && bounded.scope == *scope
            && bounded.declared_at == binding.declared_at
            && bounded.members.len() == 1
            && bounded.members[0] == *member
            && bounded
                .source
                .as_ref()
                .is_some_and(|(name, scope)| !mutated(flow, name, *scope))
            && flow.finite_identifier_guards.iter().any(|guard| {
                guard.context == Some(owner)
                    && guard.name == *name
                    && matches!(guard.kind, GuardKind::Lookup)
                    && guard.from <= *offset
                    && *offset < guard.until
                    && guard.subject
                        == Some(GuardSubject::Binding {
                            scope: *scope,
                            declared_at: binding.declared_at,
                        })
                    && !flow.identifier_writes.iter().any(|write| {
                        write.scope == *scope
                            && write.name == *name
                            && guard.from < write.at
                            && write.at < *offset
                    })
            })
    })
}

fn mutated(flow: &ValueFlowFacts, name: &str, scope: usize) -> bool {
    flow.registry_mutations
        .iter()
        .any(|mutation| mutation.name == name && mutation.scope == scope)
}

#[cfg(test)]
#[path = "metadata_tests.rs"]
mod tests;
