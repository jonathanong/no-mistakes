//! One immutable primitive helper result read by a confirmed executor.
use super::{
    bindings::Bindings,
    finite_returns,
    syntax::{Head, Syntax},
    HeadProof, Span,
};
use crate::fx::FxHashMap;

pub(super) fn proof(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    call: &super::syntax::Call,
    memo: &mut FxHashMap<usize, Option<HeadProof>>,
) -> Option<HeadProof> {
    let Head::Reference(reference) = call.head else {
        return None;
    };
    if !syntax.confirmed_builder_reads.contains(&reference) || !syntax.dynamic_refs.is_empty() {
        return None;
    }
    let result = bindings.owner(reference).filter(|owner| !owner.imported)?;
    let declaration = (result.span.start, result.span.end);
    let &(callee, initialized) = syntax.record_initializers.get(&declaration)?;
    if !syntax.primitive_initializers.contains(&declaration)
        || !bindings.declaration_safe(declaration)
        || bindings.exported(result)
        || syntax.repeated_calls.contains(&initialized)
        || syntax.repeated_calls.contains(&call.span)
        || initialized.1 >= reference.0
        || !single_read(syntax, bindings, result, reference)
    {
        return None;
    }
    let owner = bindings.owner(callee).filter(|owner| !owner.imported)?;
    *memo.entry(owner.id).or_insert_with(|| {
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("postgres.annotation.primitive_result_censuses", 1);
        }
        if syntax
            .primitive_functions
            .contains(&(owner.span.start, owner.span.end))
        {
            finite_returns::primitive_head_proof(syntax, bindings, owner)
        } else {
            None
        }
    })
}

fn single_read(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    result: &crate::codebase::ts_source::module_facts::ModuleBinding,
    reference: Span,
) -> bool {
    let context = syntax.contexts.get(&reference);
    syntax.contexts.get(&(result.span.start, result.span.end)) == context
        && result
            .references
            .iter()
            .filter(|usage| usage.runtime)
            .count()
            == 1
        && result
            .references
            .iter()
            .filter(|usage| usage.runtime)
            .all(|usage| {
                let span = (usage.span.start, usage.span.end);
                span == reference
                    && bindings.declaration_safe(span)
                    && bindings
                        .owner(span)
                        .is_some_and(|owner| owner.id == result.id)
                    && !syntax.assignments.writes.contains_key(&span)
                    && !syntax.assignments.unsupported_writes.contains(&span)
            })
}
