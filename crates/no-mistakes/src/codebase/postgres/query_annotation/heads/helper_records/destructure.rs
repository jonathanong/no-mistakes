//! Fresh plain record destructured into one exact executor text/values pair.
use super::{Bindings, ModuleBinding, Summary, Syntax};

pub(in crate::codebase::postgres::query_annotation::heads) fn candidate<'a>(
    syntax: &Syntax,
    bindings: &'a Bindings<'_>,
    call: &super::super::syntax::Call,
) -> Option<&'a ModuleBinding> {
    let (text_ref, values_ref) = *syntax.destructure_calls.get(&call.span)?;
    let text = bindings.owner(text_ref).filter(|owner| !owner.imported)?;
    let values = bindings.owner(values_ref).filter(|owner| !owner.imported)?;
    let text_decl = (text.span.start, text.span.end);
    let values_decl = (values.span.start, values.span.end);
    let source = syntax.record_destructures.get(&text_decl)?;
    if source.text != text_decl
        || source.values != values_decl
        || source.init.1 >= call.span.0
        || !bindings.declaration_safe(text_decl)
        || !bindings.declaration_safe(values_decl)
        || bindings.exported(text)
        || bindings.exported(values)
        || source.context.function != call.context.function
        || source.context.repeated != call.context.repeated
        || !single_read(syntax, text, text_ref)
        || !single_read(syntax, values, values_ref)
    {
        return None;
    }
    bindings.owner(source.callee)
}

fn single_read(syntax: &Syntax, owner: &ModuleBinding, wanted: super::super::Span) -> bool {
    let [usage] = owner.references.as_slice() else {
        return false;
    };
    let span = (usage.span.start, usage.span.end);
    usage.runtime
        && span == wanted
        && syntax.contexts.get(&span) == syntax.contexts.get(&(owner.span.start, owner.span.end))
        && !syntax.assignments.writes.contains_key(&span)
        && !syntax.assignments.unsupported_writes.contains(&span)
}

pub(in crate::codebase::postgres::query_annotation::heads) fn local_proof(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    call: &super::super::syntax::Call,
) -> Option<Summary> {
    let owner = candidate(syntax, bindings, call)?;
    if owner.imported {
        return None;
    }
    match super::source_proof(syntax, bindings, owner)? {
        summary @ Summary::Plain(_) => Some(summary),
        Summary::PlainLocal { .. } | Summary::Builder { .. } | Summary::ReturnedLocal { .. } => {
            None
        }
    }
}
