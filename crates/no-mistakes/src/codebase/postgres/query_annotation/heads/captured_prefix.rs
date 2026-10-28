//! A trusted builder's fixed head survives any ordering of closed uses.
use super::{
    bindings::Bindings,
    syntax::{Head, Syntax},
    Span,
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::{FxHashMap, FxHashSet};
mod pending;
mod returned;
pub(super) use returned::proofs as return_proofs;

#[derive(Default)]
pub(super) struct Proofs {
    pub unconditional: FxHashSet<Span>,
    pub required: FxHashMap<Span, Vec<Span>>,
}

pub(super) fn proofs(
    syntax: &Syntax,
    module: &TypeScriptModuleFacts,
    bindings: &Bindings<'_>,
    executors: &FxHashSet<Span>,
) -> Proofs {
    // Direct eval and the implicit argument array can expose a captured local
    // without an ordinary reference to its binding. Stay conservative here.
    if !bindings.has_trusted_roots()
        || !syntax.dynamic_refs.is_empty()
        || !module.diagnostics.is_empty()
    {
        return Proofs::default();
    }
    let mut reads = FxHashSet::default();
    let mut paired = FxHashSet::default();
    for call in syntax
        .calls
        .iter()
        .filter(|call| executors.contains(&call.span))
    {
        match call.head {
            Head::Reference(span) => {
                reads.insert(span);
            }
            Head::FieldReference(span) if syntax.record_result_calls.contains(&call.span) => {
                reads.insert(span);
                paired.insert(call.span);
            }
            _ => {}
        }
    }
    reads.extend(
        syntax
            .value_reads
            .iter()
            .filter_map(|(reference, call)| paired.contains(call).then_some(*reference)),
    );
    let module_scopes = module
        .scopes
        .iter()
        .filter(|scope| scope.parent_id.is_none())
        .map(|scope| scope.id)
        .collect::<FxHashSet<_>>();
    let exported = module
        .exports
        .iter()
        .filter(|item| item.specifier.is_none())
        .map(|item| item.local.as_str())
        .collect::<FxHashSet<_>>();
    let mut pending = pending::PendingUses::new(syntax, bindings);
    let pending_references = std::mem::take(&mut pending.references);
    let mut result = Proofs::default();
    let mut owners = 0;
    let mut visits = 0;
    let closed = module.bindings.iter().filter_map(|binding| {
        let declaration = (binding.span.start, binding.span.end);
        if !binding.runtime
            || binding.imported
            || module_scopes.contains(&binding.scope_id)
            || exported.contains(binding.name.as_str())
            || !bindings.trusted_root(declaration)
            || !matches!(
                syntax.constants.get(&declaration),
                Some(Head::Tag | Head::AppendTag | Head::AppendPrefixTag)
            )
            || !bindings.declaration_safe(declaration)
        {
            return None;
        }
        let context = syntax.contexts.get(&declaration)?;
        owners += 1;
        let mut required = FxHashSet::default();
        let mut captured = false;
        let mut read = false;
        for usage in &binding.references {
            visits += 1;
            if !usage.runtime {
                return None;
            }
            let span = (usage.span.start, usage.span.end);
            if bindings
                .owner(span)
                .is_none_or(|owner| owner.id != binding.id)
            {
                return None;
            }
            let usage_context = syntax.contexts.get(&span)?;
            captured |= usage_context != context;
            read |= reads.contains(&span);
            if let Some(effect) = pending_references.get(&span) {
                required.insert(*effect);
                if required.len() > 8 {
                    return None;
                }
                continue;
            }
            if !(syntax.discarded_appends.contains(&span)
                || syntax.prefix_discarded_appends.contains(&span)
                || reads.contains(&span)
                || bindings.allowed_effect(span))
            {
                return None;
            }
        }
        (captured && read).then_some((declaration, required))
    });
    for (declaration, required) in closed {
        if required.is_empty() {
            result.unconditional.insert(declaration);
        } else {
            if !required
                .iter()
                .all(|span| pending.accepts(*span, syntax, bindings))
            {
                continue;
            }
            let mut required = required.into_iter().collect::<Vec<_>>();
            required.sort_unstable();
            result.required.insert(declaration, required);
        }
    }
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment("postgres.annotation.captured_lifetime_censuses", owners);
        observer.increment(
            "postgres.annotation.captured_lifetime_reference_visits",
            visits,
        );
        pending.report(&observer);
    }
    result
}
