use super::super::postgres_sql_cross_file::{metadata_helper_call, CompiledOptions};
use super::*;
use crate::codebase::dependencies::extract::{
    unique_closed_collection, FiniteIdentifierGuard, GuardKind, GuardSubject, RegistryKind,
    SetMethod, ValueFlowFacts,
};

mod validators;

pub(super) fn proven(
    graph: &DepGraph,
    opts: &CompiledOptions,
    facts: &CheckFactMap,
    bindings: &Bindings<'_>,
    call: &SqlCall<'_>,
    provenance: &Provenance,
) -> bool {
    let Provenance::Reference {
        name,
        scope,
        offset,
        members,
    } = provenance
    else {
        return false;
    };
    let Some(ts) = facts.get_ts_facts(&call.file) else {
        return false;
    };
    let flow = &ts.value_flow;
    if let Some(binding) = bindings.value(&call.file, *scope, name, *offset) {
        if bounded(flow, binding, members, false)
            || members.is_empty() && metadata_helper_call(graph, facts, &call.file, &binding.value)
        {
            return true;
        }
        if members.len() == 1 && registry_member(flow, binding, &members[0]) {
            return true;
        }
    }
    let Some(context) = flow.calls.iter().find_map(|value_call| {
        ((value_call.offset, value_call.end) == call.span).then_some(value_call.context)
    }) else {
        return false;
    };
    flow.finite_identifier_guards.iter().any(|guard| {
        guard.name == *name
            && (if matches!(guard.kind, GuardKind::Lookup) {
                bindings
                    .value(&call.file, *scope, name, *offset)
                    .is_some_and(|binding| bounded(flow, binding, members, true))
            } else {
                guard.members == *members
            })
            && guard.context == context
            && *offset >= guard.from
            && *offset < guard.until
            && guard_subject_holds(flow, bindings, &call.file, name, *scope, *offset, guard)
            && !written_since(flow, guard, *offset)
            && guard_applies(graph, opts, flow, &call.file, guard)
    })
}

fn guard_subject_holds(
    flow: &ValueFlowFacts,
    bindings: &Bindings<'_>,
    file: &std::path::Path,
    name: &str,
    scope: usize,
    offset: u32,
    guard: &FiniteIdentifierGuard,
) -> bool {
    match guard.subject {
        Some(GuardSubject::Binding {
            scope: guard_scope,
            declared_at,
        }) => bindings
            .value(file, scope, name, offset)
            .is_some_and(|binding| {
                binding.scope == guard_scope && binding.declared_at == declared_at
            }),
        Some(GuardSubject::Parameter {
            scope: guard_scope,
            owner,
            index,
        }) => {
            scope == guard_scope
                && bindings.value(file, scope, name, offset).is_none()
                && flow.parameters.iter().any(|parameter| {
                    parameter.owner == owner
                        && parameter.index == index
                        && parameter.scope == guard_scope
                        && parameter.name == name
                })
        }
        None => false,
    }
}

fn written_since(flow: &ValueFlowFacts, guard: &FiniteIdentifierGuard, use_at: u32) -> bool {
    let scope = match guard.subject {
        Some(GuardSubject::Binding { scope, .. } | GuardSubject::Parameter { scope, .. }) => scope,
        None => return false,
    };
    if (!guard.members.is_empty() || matches!(guard.kind, GuardKind::Lookup))
        && flow
            .registry_mutations
            .iter()
            .any(|mutation| mutation.scope == scope && mutation.name == guard.name)
    {
        return true;
    }
    flow.identifier_writes.iter().any(|write| {
        write.scope == scope
            && write.name == guard.name
            && write.at > guard.from
            && write.at < use_at
            && write
                .members
                .iter()
                .zip(&guard.members)
                .all(|(written, guarded)| written.as_ref().is_none_or(|written| written == guarded))
    })
}

fn bounded(
    flow: &ValueFlowFacts,
    binding: &crate::codebase::dependencies::extract::ValueBinding,
    members: &[String],
    requires_guard: bool,
) -> bool {
    flow.bounded_bindings.iter().any(|item| {
        item.name == binding.name
            && item.scope == binding.scope
            && item.declared_at == binding.declared_at
            && item.members == members
            && item.requires_guard == requires_guard
            && item.source.as_ref().is_none_or(|(name, scope)| {
                flow.registry_mutations
                    .iter()
                    .all(|mutation| &mutation.name != name || mutation.scope != *scope)
            })
    })
}

fn registry_member(
    flow: &ValueFlowFacts,
    binding: &crate::codebase::dependencies::extract::ValueBinding,
    key: &str,
) -> bool {
    if flow
        .registry_mutations
        .iter()
        .any(|mutation| mutation.name == binding.name && mutation.scope == binding.scope)
    {
        return false;
    }
    flow.closed_string_registries.iter().any(|registry| {
        registry.kind == RegistryKind::Object
            && registry.name == binding.name
            && registry.scope == binding.scope
            && registry.declared_at == binding.declared_at
            && registry.keys.iter().any(|item| item == key)
    })
}

fn guard_applies(
    graph: &DepGraph,
    opts: &CompiledOptions,
    flow: &ValueFlowFacts,
    file: &Path,
    guard: &FiniteIdentifierGuard,
) -> bool {
    match &guard.kind {
        GuardKind::ClosedSet { collection, method } => {
            let kind = match method {
                SetMethod::Has => RegistryKind::Set,
                SetMethod::Includes => RegistryKind::Array,
            };
            unique_closed_collection(flow, collection, kind)
        }
        GuardKind::Assertion {
            callee,
            argument,
            call,
        } => {
            opts.finite_domain_assertions
                .iter()
                .any(|assertion| assertion.name == *callee && assertion.argument == *argument)
                || validators::matches(graph, opts, file, callee, *argument, *call)
        }
        GuardKind::Lookup => true,
    }
}

#[cfg(test)]
#[path = "closed_tests.rs"]
mod tests;
