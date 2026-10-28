//! Low-only closed lifetimes whose every binding write preserves native identity.
use super::{assignments::Context, bindings::Bindings, syntax, HeadProof, Span};
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};
use crate::fx::{FxHashMap, FxHashSet};
mod collect;
#[derive(Default)]
pub(super) struct Syntax {
    pub roots: FxHashMap<Span, Root>,
    writes: FxHashMap<Span, Write>,
    reads: FxHashMap<Span, Span>,
}
pub(super) struct Root {
    pub tag: Span,
    span: Span,
    context: Context,
    proof: HeadProof,
}
struct Write {
    right: Span,
    span: Span,
    context: Context,
}
#[derive(Default)]
struct Work {
    owners: u64,
    references: u64,
    tags: u64,
    tag_references: u64,
}
pub(super) fn proofs(
    syntax: &syntax::Syntax,
    bindings: &Bindings<'_>,
    module: Option<&TypeScriptModuleFacts>,
    executors: &FxHashSet<Span>,
) -> Vec<(Span, HeadProof)> {
    if syntax.self_appends.roots.is_empty()
        || executors.is_empty()
        || !syntax.dynamic_refs.is_empty()
    {
        return Vec::new();
    }
    let Some(module) = module.filter(|module| module.diagnostics.is_empty()) else {
        return Vec::new();
    };
    let reads = syntax
        .calls
        .iter()
        .filter(|call| executors.contains(&call.span) && call.context.repeated.is_none())
        .filter_map(|call| {
            let reference = *syntax.self_appends.reads.get(&call.span)?;
            let owner = bindings.owner(reference)?;
            syntax
                .self_appends
                .roots
                .contains_key(&(owner.span.start, owner.span.end))
                .then_some((call.span, reference, call.context.function))
        })
        .collect::<Vec<_>>();
    if reads.is_empty() {
        return Vec::new();
    }
    let exported = module
        .exports
        .iter()
        .filter(|item| item.specifier.is_none())
        .map(|item| item.local.as_str())
        .collect::<FxHashSet<_>>();
    let read_refs = reads
        .iter()
        .map(|(_, reference, _)| *reference)
        .collect::<FxHashSet<_>>();
    let mut owners: FxHashMap<usize, Option<HeadProof>> = FxHashMap::default();
    let mut tags = FxHashMap::default();
    let mut work = Work::default();
    let mut result = Vec::new();
    for (call, reference, function) in reads {
        let Some(owner) = bindings.owner(reference) else {
            continue;
        };
        let declaration = (owner.span.start, owner.span.end);
        let Some(root) = syntax.self_appends.roots.get(&declaration) else {
            continue;
        };
        if function != root.context.function || root.span.1 >= call.0 {
            continue;
        }
        let proof = *owners.entry(owner.id).or_insert_with(|| {
            work.owners += 1;
            if owner.imported
                || exported.contains(owner.name.as_str())
                || root.proof == HeadProof::Unproven
                || !bindings.declaration_safe(root.span)
            {
                return None;
            }
            let tag = bindings.owner(root.tag)?;
            let stable = *tags.entry(tag.id).or_insert_with(|| {
                work.tags += 1;
                tag.imported
                    && bindings.trusted_import(tag.id)
                    && tag.references.iter().all(|usage| {
                        work.tag_references += 1;
                        let span = (usage.span.start, usage.span.end);
                        bindings.declaration_safe(span)
                            && !syntax.assignments.writes.contains_key(&span)
                            && !syntax.assignments.unsupported_writes.contains(&span)
                    })
            });
            if !stable {
                return None;
            }
            closed(owner, root, syntax, bindings, &read_refs, &mut work).then_some(root.proof)
        });
        if let Some(proof) = proof {
            result.push((call, proof));
        }
    }
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment(
            "postgres.annotation.self_append_owner_censuses",
            work.owners,
        );
        observer.increment(
            "postgres.annotation.self_append_reference_visits",
            work.references,
        );
        observer.increment("postgres.annotation.self_append_tag_censuses", work.tags);
        observer.increment(
            "postgres.annotation.self_append_tag_reference_visits",
            work.tag_references,
        );
    }
    result
}
fn closed(
    owner: &ModuleBinding,
    root: &Root,
    syntax: &syntax::Syntax,
    bindings: &Bindings<'_>,
    reads: &FxHashSet<Span>,
    work: &mut Work,
) -> bool {
    let mut receivers = FxHashSet::default();
    let mut writes = 0;
    // Index this owner's approved writes before checking RHS reads. No per-read census.
    for usage in &owner.references {
        work.references += 1;
        let span = (usage.span.start, usage.span.end);
        if let Some(write) = syntax.self_appends.writes.get(&span) {
            if write.context.function != root.context.function
                || write.context.repeated.is_some()
                || write.span.0 <= root.span.1
                || !bindings.declaration_safe(write.span)
                || bindings
                    .owner(write.right)
                    .is_none_or(|right| right.id != owner.id)
            {
                return false;
            }
            writes += 1;
            receivers.insert(write.right);
        }
    }
    writes > 0
        && owner.references.iter().all(|usage| {
            work.references += 1;
            let span = (usage.span.start, usage.span.end);
            !syntax.repeated_append_refs.contains(&span)
                && usage.runtime
                && span.0 > root.span.1
                && bindings.owner(span).is_some_and(|item| item.id == owner.id)
                && syntax
                    .contexts
                    .get(&span)
                    .copied()
                    .flatten()
                    .map(|context| context.0)
                    == root.context.function
                && !syntax.assignments.unsupported_writes.contains(&span)
                && (syntax.self_appends.writes.contains_key(&span)
                    || receivers.contains(&span)
                    || reads.contains(&span)
                    || syntax.discarded_appends.contains(&span)
                    || syntax.prefix_discarded_appends.contains(&span))
        })
}
