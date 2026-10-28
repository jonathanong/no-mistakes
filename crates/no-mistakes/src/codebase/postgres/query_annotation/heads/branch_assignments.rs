//! Exhaustive adjacent assignments of fresh native builders, without value evaluation.
use super::{bindings::Bindings, syntax::Syntax, HeadProof, Span};
use crate::fx::FxHashSet;
use oxc_ast::ast::FunctionBody;
mod collect;

#[derive(Clone, Copy)]
pub(super) struct Arm {
    reference: Span,
    tag: Span,
    proof: HeadProof,
}
pub(super) struct Candidate {
    body: Span,
    declaration: Span,
    arms: [Arm; 2],
    call: Span,
    reference: Span,
}
impl Candidate {
    pub fn references(&self) -> impl Iterator<Item = Span> + '_ {
        self.arms.iter().flat_map(|arm| [arm.reference, arm.tag])
    }
}
pub(super) fn collect(syntax: &mut Syntax, body: &FunctionBody<'_>) {
    if syntax.collect_helper_heads && syntax.assignments.context.function.is_some() {
        syntax.branch_assignments.extend(collect::candidates(body));
    }
}
pub(super) fn proofs(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    executors: &FxHashSet<Span>,
) -> Vec<(Span, HeadProof)> {
    if syntax.branch_assignments.is_empty() {
        return Vec::new();
    }
    // Sort once: checking a function for eval/arguments must not rescan every
    // dynamic reference for each of its adjacent assignment candidates.
    let mut dynamic = syntax.dynamic_refs.clone();
    dynamic.sort_unstable();
    syntax
        .branch_assignments
        .iter()
        .filter_map(|candidate| {
            let start = dynamic.partition_point(|span| span.0 < candidate.body.0);
            if !executors.contains(&candidate.call)
                || !bindings.declaration_safe(candidate.body)
                || dynamic
                    .get(start)
                    .is_some_and(|span| span.0 < candidate.body.1)
            {
                return None;
            }
            let owner = bindings.owner(candidate.reference)?;
            if owner.imported || (owner.span.start, owner.span.end) != candidate.declaration {
                return None;
            }
            let expected = [
                candidate.arms[0].reference,
                candidate.arms[1].reference,
                candidate.reference,
            ];
            let references = owner
                .references
                .iter()
                .filter(|usage| usage.runtime)
                .take(4)
                .collect::<Vec<_>>();
            if references.len() != 3
                || references
                    .iter()
                    .any(|usage| !expected.contains(&(usage.span.start, usage.span.end)))
            {
                return None;
            }
            for arm in &candidate.arms {
                let tag = bindings.owner(arm.tag)?;
                if !tag.imported || !bindings.trusted_import(tag.id) {
                    return None;
                }
            }
            let proof = candidate.arms[0].proof;
            (proof == candidate.arms[1].proof && proof != HeadProof::Unproven)
                .then_some((candidate.call, proof))
        })
        .collect()
}
