use super::super::{bindings::Bindings, syntax::Syntax, HeadProof, Span};
use super::Conditional;
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;

pub(in crate::codebase::postgres::query_annotation::heads) fn collect(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    executors: &FxHashSet<Span>,
    calls: &BTreeMap<Span, HeadProof>,
) -> Vec<Conditional> {
    let observer = crate::diagnostics::current();
    let mut groups = FxHashMap::<Span, Conditional>::default();
    for call in syntax
        .calls
        .iter()
        .filter(|call| executors.contains(&call.span))
        .filter(|call| !matches!(calls.get(&call.span), Some(HeadProof::Literal)))
    {
        let Some((declaration, required)) = bindings.conditional_prefix(call.head, call.span)
        else {
            continue;
        };
        let group = groups.entry(declaration).or_insert_with(|| {
            let mut required = required.to_vec();
            if let Some(observer) = &observer {
                observer.increment("postgres.annotation.conditional_effect_normalizations", 1);
                observer.increment("postgres.annotation.conditional_effect_groups", 1);
                observer.increment(
                    "postgres.annotation.conditional_effect_requirements_copied",
                    required.len() as u64,
                );
            }
            required.sort_unstable();
            required.dedup();
            Conditional {
                spans: Vec::new(),
                required,
            }
        });
        group.spans.push(call.span);
    }
    let mut groups = groups.into_iter().collect::<Vec<_>>();
    groups.sort_unstable_by_key(|(declaration, _)| *declaration);
    groups
        .into_iter()
        .map(|(_, mut group)| {
            group.spans.sort_unstable();
            group.spans.dedup();
            group
        })
        .collect()
}
