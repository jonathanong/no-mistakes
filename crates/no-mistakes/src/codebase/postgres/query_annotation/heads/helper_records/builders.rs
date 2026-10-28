//! Provisional helper-returned builders at exact executor endpoints.
use super::{Bindings, ModuleBinding, Summary, Syntax};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::Expression;

pub(in crate::codebase::postgres::query_annotation::heads) fn confirmed_reads(
    syntax: &Syntax,
    executors: &FxHashSet<super::Span>,
) -> FxHashSet<super::Span> {
    syntax
        .calls
        .iter()
        .filter(|call| {
            executors.contains(&call.span)
                && !syntax.optional_calls.contains(&call.span)
                && !syntax.repeated_calls.contains(&call.span)
        })
        .filter_map(|call| match call.head {
            super::super::syntax::Head::Reference(span) => Some(span),
            _ => None,
        })
        .collect()
}

pub(in crate::codebase::postgres::query_annotation::heads) fn append_callee(
    expression: &Expression<'_>,
) -> Option<super::Span> {
    let mut value = unwrap_ts_wrappers(expression);
    let mut appended = false;
    while let Expression::CallExpression(call) = value {
        let Some(receiver) = super::super::append::prefix_receiver(call) else {
            break;
        };
        value = unwrap_ts_wrappers(receiver);
        appended = true;
    }
    appended
        .then(|| super::super::helper_heads::callee(value))
        .flatten()
}

pub(in crate::codebase::postgres::query_annotation::heads) fn builder_candidate<'a>(
    syntax: &Syntax,
    bindings: &'a Bindings<'_>,
    call: &super::super::syntax::Call,
) -> Option<&'a ModuleBinding> {
    if syntax.optional_calls.contains(&call.span) {
        return None;
    }
    if let Some(&reference) = syntax.builder_direct_calls.get(&call.span) {
        if syntax.repeated_calls.contains(&call.span) {
            return None;
        }
        return bindings.owner(reference);
    }
    let super::super::syntax::Head::Reference(use_span) = call.head else {
        return None;
    };
    let result = bindings.owner(use_span).filter(|owner| !owner.imported)?;
    let declaration = (result.span.start, result.span.end);
    let (callee, init_call) = *syntax.record_initializers.get(&declaration)?;
    if !bindings.declaration_safe(declaration)
        || bindings.exported(result)
        || syntax.repeated_calls.contains(&init_call)
        || syntax.repeated_calls.contains(&call.span)
        || declaration.1 >= call.span.0
    {
        return None;
    }
    let context = syntax.contexts.get(&declaration)?;
    let mut executor_use = false;
    for usage in &result.references {
        let span = (usage.span.start, usage.span.end);
        if !usage.runtime
            || syntax.contexts.get(&span) != Some(context)
            || syntax.assignments.writes.contains_key(&span)
            || syntax.assignments.unsupported_writes.contains(&span)
            || span.0 <= init_call.1
        {
            return None;
        }
        if syntax.confirmed_builder_reads.contains(&span) {
            executor_use |= span == use_span;
            continue;
        }
        if syntax.repeated_append_refs.contains(&span)
            || !(syntax.discarded_appends.contains(&span)
                || syntax.prefix_discarded_appends.contains(&span))
        {
            return None;
        }
    }
    executor_use.then(|| bindings.owner(callee)).flatten()
}

pub(in crate::codebase::postgres::query_annotation::heads) fn builder_local_proof(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    call: &super::super::syntax::Call,
) -> Option<Summary> {
    let owner = builder_candidate(syntax, bindings, call)?;
    if owner.imported {
        return None;
    }
    match super::source_proof(syntax, bindings, owner)? {
        summary @ Summary::Builder { .. } => Some(summary),
        Summary::Plain(_) | Summary::PlainLocal { .. } | Summary::ReturnedLocal { .. } => None,
    }
}
