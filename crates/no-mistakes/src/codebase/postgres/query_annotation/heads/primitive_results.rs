//! One immutable primitive helper result read by a confirmed executor.
use super::{
    bindings::{Bindings, ExportIndex},
    finite_returns,
    syntax::{Head, Syntax},
    HeadProof, Span,
};
use crate::fx::FxHashMap;

#[derive(Default)]
pub(super) struct Memo {
    owners: FxHashMap<usize, Option<Span>>,
    helpers: FxHashMap<usize, Option<HeadProof>>,
    exports: Option<ExportIndex>,
}

pub(super) fn proof(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    call: &super::syntax::Call,
    memo: &mut Memo,
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
        || syntax.repeated_calls.contains(&call.span)
    {
        return None;
    }
    let selected = *memo.owners.entry(result.id).or_insert_with(|| {
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("postgres.annotation.primitive_owner_censuses", 1);
        }
        if !bindings.declaration_safe(declaration)
            || memo
                .exports
                .get_or_insert_with(|| bindings.export_index())
                .contains(result)
            || syntax.repeated_calls.contains(&initialized)
        {
            return None;
        }
        single_read(syntax, bindings, result)
    });
    if selected != Some(reference) || initialized.1 >= reference.0 {
        return None;
    }
    let owner = bindings.owner(callee).filter(|owner| !owner.imported)?;
    *memo.helpers.entry(owner.id).or_insert_with(|| {
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
) -> Option<Span> {
    let mut selected = None;
    for usage in result.references.iter().filter(|usage| usage.runtime) {
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("postgres.annotation.primitive_owner_refs_visited", 1);
        }
        let span = (usage.span.start, usage.span.end);
        if selected.is_some()
            || syntax.contexts.get(&(result.span.start, result.span.end))
                != syntax.contexts.get(&span)
            || !syntax.confirmed_builder_reads.contains(&span)
            || !bindings.declaration_safe(span)
            || !bindings
                .owner(span)
                .is_some_and(|owner| owner.id == result.id)
            || syntax.assignments.writes.contains_key(&span)
            || syntax.assignments.unsupported_writes.contains(&span)
        {
            return None;
        }
        selected = Some(span);
    }
    selected
}
