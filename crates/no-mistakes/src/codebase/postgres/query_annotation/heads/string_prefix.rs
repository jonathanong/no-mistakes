//! A local primitive string retains a continuation-safe head through literal +=.
use super::{bindings::Bindings, syntax::Head, syntax::Syntax, HeadProof, Span};
use crate::codebase::ts_source::module_facts::ModuleBinding;
use crate::fx::{FxHashMap, FxHashSet};

pub(super) fn proofs(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    executors: &FxHashSet<Span>,
) -> FxHashMap<Span, HeadProof> {
    let mut calls = FxHashMap::default();
    if executors.is_empty()
        || syntax.string_heads.is_empty()
        || syntax.string_appends.is_empty()
        || !syntax.dynamic_refs.is_empty()
    {
        return calls;
    }
    let reads = syntax
        .calls
        .iter()
        .filter(|call| executors.contains(&call.span))
        .filter_map(|call| match call.head {
            Head::Reference(reference) if !syntax.optional_calls.contains(&call.span) => {
                Some((reference, call))
            }
            _ => None,
        })
        .collect::<FxHashMap<_, _>>();
    // Prefix maxima identify executor reads nested in any assignment RHS without
    // rescanning all writes per call (including reentrant template expressions).
    let mut write_ends = syntax
        .assignments
        .writes
        .values()
        .map(|write| write.span)
        .collect::<Vec<_>>();
    write_ends.sort_unstable();
    let mut end = 0;
    for write in &mut write_ends {
        end = end.max(write.1);
        write.1 = end;
    }
    // Cache both positive and negative ownership censuses, once per declaration.
    let mut owners = FxHashMap::default();
    for (&reference, call) in &reads {
        let Some(owner) = bindings.owner(reference) else {
            continue;
        };
        let declaration = (owner.span.start, owner.span.end);
        let Some(head) = syntax.string_heads.get(&declaration) else {
            continue;
        };
        let valid = *owners
            .entry(declaration)
            .or_insert_with(|| closed(syntax, bindings, &reads, &write_ends, owner, *head));
        if valid {
            calls.insert(call.span, HeadProof::Literal);
        }
    }
    calls
}

fn closed(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    reads: &FxHashMap<Span, &super::syntax::Call>,
    write_ends: &[Span],
    owner: &ModuleBinding,
    head: Head,
) -> bool {
    let declaration = (owner.span.start, owner.span.end);
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment("postgres.annotation.string_prefix_owner_censuses", 1);
    }
    let Some(initial) = syntax.assignments.declarations.get(&declaration) else {
        return false;
    };
    let context = syntax.contexts.get(&declaration).copied().flatten();
    if owner.imported
        || !owner.runtime
        || context.is_none()
        || initial.context.repeated.is_some()
        || !bindings.declaration_safe(initial.span)
        || head.proof(true) != HeadProof::Literal
    {
        return false;
    }
    owner.references.iter().all(|usage| {
        let span = (usage.span.start, usage.span.end);
        if !usage.runtime
            || bindings
                .owner(span)
                .is_none_or(|binding| binding.id != owner.id)
            || syntax.contexts.get(&span).copied().flatten() != context
            || syntax.assignments.unsupported_writes.contains(&span)
        {
            return false;
        }
        if let Some(write) = syntax.assignments.writes.get(&span) {
            syntax.string_appends.contains(&span)
                && write.context.function == initial.context.function
                && write.context.repeated.is_none()
                && initial.span.1 < write.span.0
                && bindings.declaration_safe(write.span)
        } else {
            let index = write_ends.partition_point(|write| write.0 <= span.0);
            (index == 0 || write_ends[index - 1].1 <= span.0)
                && reads.get(&span).is_some_and(|call| {
                    initial.span.1 < call.span.0
                        && call.context.repeated.is_none()
                        && bindings.declaration_safe(call.span)
                })
        }
    })
}
