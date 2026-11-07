//! A fresh imported helper result remains private through paired field reads.
use super::super::{
    bindings::Bindings,
    syntax::{Call, Head, Syntax},
    Span,
};

pub(super) fn declaration(syntax: &Syntax, bindings: &Bindings<'_>, call: &Call) -> Option<Span> {
    if !syntax.record_result_calls.contains(&call.span)
        || syntax.repeated_calls.contains(&call.span)
        || !syntax.dynamic_refs.is_empty()
        || !bindings.declaration_safe(call.span)
    {
        return None;
    }
    let Head::FieldReference(reference) = call.head else {
        return None;
    };
    let owner = bindings.owner(reference).filter(|owner| !owner.imported)?;
    let declaration = (owner.span.start, owner.span.end);
    let initial = syntax.returned_param_roots.get(&declaration)?.initializer;
    if bindings.exported(owner)
        || !bindings.declaration_safe(declaration)
        || !bindings.declaration_safe(initial)
        || declaration.1 >= call.span.0
    {
        return None;
    }
    let context = syntax.contexts.get(&declaration)?;
    let mut text = false;
    let mut values = false;
    for usage in owner.references.iter().filter(|usage| usage.runtime) {
        let span = (usage.span.start, usage.span.end);
        if syntax.contexts.get(&span) != Some(context)
            || !bindings.declaration_safe(span)
            || span.0 <= initial.1
            || syntax.assignments.writes.contains_key(&span)
            || syntax.assignments.unsupported_writes.contains(&span)
        {
            return None;
        }
        if span == reference {
            text = true;
        } else if syntax.value_reads.get(&span) == Some(&call.span) {
            values = true;
        } else if !syntax.discarded_appends.contains(&span) || span.1 >= call.span.0 {
            return None;
        }
    }
    (text && values).then_some(declaration)
}
