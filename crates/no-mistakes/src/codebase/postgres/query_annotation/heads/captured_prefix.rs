//! A trusted builder's fixed head survives any ordering of closed uses.
use super::{
    bindings::Bindings,
    syntax::{Head, Syntax},
    Span,
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashSet;
mod returned;
pub(super) use returned::proofs as return_proofs;

pub(super) fn proofs(
    syntax: &Syntax,
    module: &TypeScriptModuleFacts,
    bindings: &Bindings<'_>,
    executors: &FxHashSet<Span>,
) -> FxHashSet<Span> {
    // Direct eval and the implicit argument array can expose a captured local
    // without an ordinary reference to its binding. Stay conservative here.
    if !bindings.has_trusted_roots()
        || !syntax.dynamic_refs.is_empty()
        || !module.diagnostics.is_empty()
    {
        return FxHashSet::default();
    }
    let reads = syntax
        .calls
        .iter()
        .filter(|call| executors.contains(&call.span))
        .filter_map(|call| match call.head {
            Head::Reference(span) => Some(span),
            _ => None,
        })
        .collect::<FxHashSet<_>>();
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
    module
        .bindings
        .iter()
        .filter_map(|binding| {
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
                || binding.references.iter().any(|usage| !usage.runtime)
            {
                return None;
            }
            let context = syntax.contexts.get(&declaration)?;
            let mut captured = false;
            let mut read = false;
            for usage in &binding.references {
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
                if !(syntax.discarded_appends.contains(&span)
                    || syntax.prefix_discarded_appends.contains(&span)
                    || reads.contains(&span)
                    || bindings.allowed_effect(span))
                {
                    return None;
                }
            }
            (captured && read).then_some(declaration)
        })
        .collect()
}
