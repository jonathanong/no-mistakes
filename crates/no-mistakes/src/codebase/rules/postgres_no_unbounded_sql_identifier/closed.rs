use super::super::postgres_sql_cross_file::CompiledOptions;
use super::*;
use crate::codebase::dependencies::extract::{
    unique_closed_collection, GuardKind, RegistryKind, SetMethod, ValueFlowFacts,
};

pub(super) fn proven(
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
        if members.is_empty() && bounded(flow, binding) {
            return true;
        }
        if members.len() == 1 && registry_member(flow, binding, &members[0]) {
            return true;
        }
    }
    if !members.is_empty() {
        return false;
    }
    let Some(context) = flow.calls.iter().find_map(|value_call| {
        ((value_call.offset, value_call.end) == call.span).then_some(value_call.context)
    }) else {
        return false;
    };
    flow.finite_identifier_guards.iter().any(|guard| {
        guard.name == *name
            && guard.context == context
            && call.span.0 >= guard.from
            && call.span.0 < guard.until
            && guard_applies(opts, flow, &guard.kind)
    })
}

fn bounded(
    flow: &ValueFlowFacts,
    binding: &crate::codebase::dependencies::extract::ValueBinding,
) -> bool {
    flow.bounded_bindings.iter().any(|item| {
        item.name == binding.name
            && item.scope == binding.scope
            && item.declared_at == binding.declared_at
    })
}

fn registry_member(
    flow: &ValueFlowFacts,
    binding: &crate::codebase::dependencies::extract::ValueBinding,
    key: &str,
) -> bool {
    flow.closed_string_registries.iter().any(|registry| {
        registry.kind == RegistryKind::Object
            && registry.name == binding.name
            && registry.scope == binding.scope
            && registry.declared_at == binding.declared_at
            && registry.keys.iter().any(|item| item == key)
    })
}

fn guard_applies(opts: &CompiledOptions, flow: &ValueFlowFacts, kind: &GuardKind) -> bool {
    match kind {
        GuardKind::ClosedSet { collection, method } => {
            let kind = match method {
                SetMethod::Has => RegistryKind::Set,
                SetMethod::Includes => RegistryKind::Array,
            };
            unique_closed_collection(flow, collection, kind)
        }
        GuardKind::Assertion { callee, argument } => opts
            .finite_domain_assertions
            .iter()
            .any(|assertion| assertion.name == *callee && assertion.argument == *argument),
    }
}

#[cfg(test)]
#[path = "closed_tests.rs"]
mod tests;
