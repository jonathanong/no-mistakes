use super::super::{bindings::Bindings, syntax::Syntax, HeadProof, Span};
use super::Conditional;
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::codebase::postgres::query_annotation::heads) fn merge(
    conditions: &mut Vec<Conditional>,
    deferred: Vec<Conditional>,
) {
    if deferred.is_empty() {
        return;
    }
    let mut per_call = BTreeMap::<Span, BTreeSet<Span>>::new();
    for condition in conditions.drain(..).chain(deferred) {
        for call in condition.spans {
            per_call
                .entry(call)
                .or_default()
                .extend(condition.required.iter().copied());
        }
    }
    let mut groups = BTreeMap::<Vec<Span>, Vec<Span>>::new();
    for (call, required) in per_call {
        groups
            .entry(required.into_iter().collect())
            .or_default()
            .push(call);
    }
    *conditions = groups
        .into_iter()
        .map(|(required, spans)| Conditional { spans, required })
        .collect();
    conditions.sort_unstable_by_key(|condition| condition.spans[0]);
}

#[cfg(test)]
mod tests {
    use super::{merge, Conditional};

    #[test]
    fn overlapping_groups_require_every_effect_only_at_the_shared_call() {
        let mut conditions = vec![Conditional {
            spans: vec![(1, 2), (3, 4)],
            required: vec![(11, 12)],
        }];
        merge(
            &mut conditions,
            vec![Conditional {
                spans: vec![(3, 4), (5, 6)],
                required: vec![(13, 14)],
            }],
        );
        assert_eq!(conditions.len(), 3);
        assert_eq!(conditions[0].spans, vec![(1, 2)]);
        assert_eq!(conditions[0].required, vec![(11, 12)]);
        assert_eq!(conditions[1].spans, vec![(3, 4)]);
        assert_eq!(conditions[1].required, vec![(11, 12), (13, 14)]);
        assert_eq!(conditions[2].spans, vec![(5, 6)]);
        assert_eq!(conditions[2].required, vec![(13, 14)]);
    }
}

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
