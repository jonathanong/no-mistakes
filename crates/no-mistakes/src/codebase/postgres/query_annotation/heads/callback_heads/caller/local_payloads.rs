//! One exceptional use of a named trusted local builder may supply a callback payload.
use super::Span;
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings,
    syntax::{Head, Syntax},
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashSet;

pub(super) fn references(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    module: &TypeScriptModuleFacts,
    append_helpers: &FxHashSet<Span>,
) -> FxHashSet<Span> {
    let observer = crate::diagnostics::current();
    if let Some(observer) = &observer {
        observer.increment("postgres.annotation.callback_local_payload_indexes", 1);
    }
    let mut allowed = FxHashSet::default();
    let append_uses = syntax
        .effect_calls
        .iter()
        .filter(|call| append_helpers.contains(&call.callee))
        .map(|call| call.builder)
        .collect::<FxHashSet<_>>();
    let mut binding_visits = 0;
    for binding in module
        .bindings
        .iter()
        .filter(|binding| binding.runtime && !binding.imported)
    {
        binding_visits += 1;
        let declaration = (binding.span.start, binding.span.end);
        if !bindings.trusted_root(declaration)
            || !matches!(
                syntax.constants.get(&declaration),
                Some(Head::Tag | Head::AppendTag)
            )
            || !bindings.declaration_safe(declaration)
        {
            continue;
        }
        let mut exceptional = None;
        let mut sound = true;
        for usage in binding.references.iter().filter(|usage| usage.runtime) {
            let span = (usage.span.start, usage.span.end);
            if !bindings.declaration_safe(span)
                || syntax.contexts.get(&span) != syntax.contexts.get(&declaration)
                || syntax.assignments.writes.contains_key(&span)
                || syntax.assignments.unsupported_writes.contains(&span)
            {
                sound = false;
                break;
            }
            if syntax.discarded_appends.contains(&span)
                || syntax.prefix_discarded_appends.contains(&span)
                || append_uses.contains(&span)
            {
                continue;
            }
            if exceptional.replace(span).is_some() {
                sound = false;
                break;
            }
        }
        if sound {
            if let Some(span) = exceptional {
                allowed.insert(span);
            }
        }
    }
    if let Some(observer) = observer {
        observer.increment(
            "postgres.annotation.callback_local_payload_binding_visits",
            binding_visits,
        );
    }
    allowed
}
