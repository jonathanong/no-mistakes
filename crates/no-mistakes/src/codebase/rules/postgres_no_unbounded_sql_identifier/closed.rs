use super::super::postgres_sql_cross_file::CompiledOptions;
use super::*;
use crate::codebase::dependencies::extract::{
    unique_closed_collection, FiniteIdentifierGuard, GuardKind, GuardSubject, RegistryKind,
    SetMethod, ValueFlowFacts,
};

mod collection_imports;
mod collections;
mod generated;
mod metadata;
mod subjects;
mod validators;
use subjects::{guard_subject_holds, written_since};
pub(super) mod configured;

pub(super) fn proven(
    graph: &DepGraph,
    opts: &CompiledOptions,
    facts: &CheckFactMap,
    bindings: &Bindings<'_>,
    call: &SqlCall<'_>,
    provenance: &Provenance,
) -> bool {
    if configured::proven(bindings, call, provenance) {
        return true;
    }
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
        if members.is_empty() && generated::proves(graph, facts, bindings, &call.file, binding) {
            return true;
        }
        if metadata::proves(graph, facts, bindings, &call.file, flow, binding, members) {
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
            && guard_applies(graph, opts, facts, bindings, &call.file, guard)
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
    facts: &CheckFactMap,
    bindings: &Bindings<'_>,
    file: &Path,
    guard: &FiniteIdentifierGuard,
) -> bool {
    match &guard.kind {
        GuardKind::ClosedSet { collection, method } => {
            let kind = match method {
                SetMethod::Has => RegistryKind::Set,
                SetMethod::Includes => RegistryKind::Array,
            };
            collections::proven(graph, facts, bindings, file, collection, kind)
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
