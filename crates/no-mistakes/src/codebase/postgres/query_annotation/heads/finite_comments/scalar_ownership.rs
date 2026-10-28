//! Exact prepared ownership for scalar labels, helpers, and native constructors.
use super::Span;
use crate::codebase::postgres::query_annotation::heads::{bindings::Bindings, syntax::Syntax};
use crate::codebase::ts_source::module_facts::ModuleBinding;

pub(super) fn only(binding: &ModuleBinding, expected: &[Span]) -> bool {
    let mut references = binding.references.iter().filter(|usage| usage.runtime);
    references.clone().count() == expected.len()
        && references.all(|usage| expected.contains(&(usage.span.start, usage.span.end)))
}
pub(super) fn stable(syntax: &Syntax, bindings: &Bindings<'_>, owner: &ModuleBinding) -> bool {
    bindings.declaration_safe((owner.span.start, owner.span.end))
        && owner
            .references
            .iter()
            .filter(|usage| usage.runtime)
            .all(|usage| {
                let span = (usage.span.start, usage.span.end);
                bindings
                    .owner(span)
                    .is_some_and(|actual| actual.id == owner.id)
                    && !syntax.assignments.writes.contains_key(&span)
                    && !syntax.assignments.unsupported_writes.contains(&span)
            })
}

pub(super) fn native_safe(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    owner: &ModuleBinding,
    exported: bool,
) -> bool {
    owner.imported
        && !exported
        && bindings.trusted_import(owner.id)
        && stable(syntax, bindings, owner)
        && owner
            .references
            .iter()
            .filter(|usage| usage.runtime)
            .all(|usage| {
                syntax
                    .finite_tags
                    .contains(&(usage.span.start, usage.span.end))
            })
}

pub(super) fn parameter(bindings: &Bindings<'_>, reference: Span, declaration: Span) -> Option<()> {
    let owner = bindings.owner(reference)?;
    ((owner.span.start, owner.span.end) == declaration).then_some(())
}
