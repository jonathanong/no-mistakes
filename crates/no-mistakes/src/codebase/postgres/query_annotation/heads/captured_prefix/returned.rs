//! A captured builder returned by its owner keeps a fixed head if every use is closed.
use super::{Bindings, Head, Span, Syntax, TypeScriptModuleFacts};
use crate::fx::FxHashSet;

pub(in crate::codebase::postgres::query_annotation::heads) fn proofs(
    syntax: &Syntax,
    module: &TypeScriptModuleFacts,
    bindings: &Bindings<'_>,
) -> FxHashSet<Span> {
    // Dynamic lexical access could observe or change a captured builder without
    // a canonical binding reference. An unrelated gap also prevents this proof.
    if !bindings.has_trusted_roots()
        || !syntax.dynamic_refs.is_empty()
        || !module.diagnostics.is_empty()
    {
        return FxHashSet::default();
    }
    let roots = module
        .scopes
        .iter()
        .filter(|scope| scope.parent_id.is_none())
        .map(|scope| scope.id)
        .collect::<FxHashSet<_>>();
    let exports = module
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
                || roots.contains(&binding.scope_id)
                || exports.contains(binding.name.as_str())
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
            let context = syntax.contexts.get(&declaration).copied().flatten()?;
            let mut captured = false;
            let mut returned = false;
            for usage in &binding.references {
                let span = (usage.span.start, usage.span.end);
                let use_context = syntax.contexts.get(&span).copied().flatten()?;
                if bindings
                    .owner(span)
                    .is_none_or(|owner| owner.id != binding.id)
                    || syntax.assignments.writes.contains_key(&span)
                    || syntax.assignments.unsupported_writes.contains(&span)
                {
                    return None;
                }
                if syntax.return_sinks.contains(&span) && use_context == context {
                    returned = true;
                    continue;
                }
                if !(syntax.discarded_appends.contains(&span)
                    || syntax.prefix_discarded_appends.contains(&span)
                    || bindings.allowed_effect(span))
                {
                    return None;
                }
                captured |= use_context != context;
            }
            (captured && returned).then_some(declaration)
        })
        .collect()
}
