//! Closed scalar append heads reuse prepared ownership and the finite name check.
use super::{scalar_domain::Index, Span};
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings,
    syntax::{Head, Syntax},
    HeadProof,
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;

pub(in super::super) fn extend_wanted(
    syntax: &Syntax,
    module: &TypeScriptModuleFacts,
    wanted: &mut FxHashSet<Span>,
) {
    if !syntax.finite_roots.iter().any(|root| root.scalar.is_some()) {
        return;
    }
    let native = module
        .imports
        .iter()
        .filter(|import| import.specifier == "sql-template-strings" && !import.type_only)
        .flat_map(|import| &import.bindings)
        .filter(|binding| !binding.type_only)
        .map(|binding| binding.binding_id)
        .collect::<FxHashSet<_>>();
    for binding in &module.bindings {
        if binding.imported && native.contains(&binding.id) {
            wanted.extend(
                binding
                    .references
                    .iter()
                    .filter(|usage| usage.runtime)
                    .map(|usage| (usage.span.start, usage.span.end)),
            );
        }
    }
}
pub(super) fn proofs(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    module: &TypeScriptModuleFacts,
    executors: &FxHashSet<Span>,
) -> BTreeMap<Span, HeadProof> {
    let mut out = BTreeMap::new();
    if !syntax.finite_roots.iter().any(|root| root.scalar.is_some()) {
        return out;
    }
    let mut reads: FxHashMap<Span, Vec<(Span, Span)>> = FxHashMap::default();
    let mut read_references = FxHashSet::default();
    for call in syntax.calls.iter().filter(|call| {
        executors.contains(&call.span) && !syntax.optional_calls.contains(&call.span)
    }) {
        if let Head::Reference(reference) = call.head {
            if let Some(owner) = bindings.owner(reference) {
                read_references.insert(reference);
                reads
                    .entry((owner.span.start, owner.span.end))
                    .or_default()
                    .push((reference, call.span));
            }
        }
    }
    if reads.is_empty() {
        return out;
    }
    let mut index = Index::new(syntax, bindings, module);
    for root in syntax
        .finite_roots
        .iter()
        .filter(|root| root.scalar.is_some())
    {
        let Some(calls) = reads.get(&root.declaration) else {
            continue;
        };
        let Some(values) = index.values(syntax, bindings, root) else {
            continue;
        };
        if !super::proof::all_named(&root.before, &root.after, values) {
            continue;
        }
        let Some(tag) = index.constructor(syntax, bindings, root.tag) else {
            continue;
        };
        if index.constructor(syntax, bindings, root.scalar.unwrap()) != Some(tag) {
            continue;
        }
        let Some(owner) = index.declarations.get(&root.declaration).copied().flatten() else {
            continue;
        };
        if owner.imported
            || index.exported(owner)
            || !super::scalar_ownership::stable(syntax, bindings, owner)
            || owner
                .references
                .iter()
                .filter(|usage| usage.runtime)
                .any(|usage| {
                    let span = (usage.span.start, usage.span.end);
                    syntax.contexts.get(&span).copied().flatten() != Some(root.function)
                        || !(syntax.discarded_appends.contains(&span)
                            || syntax.prefix_discarded_appends.contains(&span)
                            || read_references.contains(&span))
                })
        {
            continue;
        }
        for &(reference, call) in calls {
            if bindings.finite_root_accepts(syntax, root.declaration, reference) {
                out.insert(call, HeadProof::Literal);
            }
        }
    }
    out
}
