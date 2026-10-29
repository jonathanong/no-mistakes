use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn second_argument_local_effects_keep_prepared_heads_without_evaluation() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let query = root.join("second-argument-effects.mts");
    let mut prior = None;
    let mut max_calls = None;
    for efforts in [vec!["max"], vec!["low"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, vec![query.clone()], plan(root, &config(&efforts)))
        });
        let file = facts.ts.get(&query).expect("prepared query");
        let calls = format!("{:?}", file.embedded_sql[0].1.calls);
        if efforts == ["max"] {
            max_calls = Some(calls);
            continue;
        }
        // Head-only extraction intentionally omits max-only call metadata.
        // Adding low demand must preserve max's existing embedded-call facts.
        if efforts == ["low", "max"] {
            assert_eq!(max_calls.as_ref(), Some(&calls));
        }
        let heads = file.query_annotation_heads.as_ref().expect("low heads");
        let spans = &file.embedded_sql[0].1.call_spans;
        let proofs = spans
            .iter()
            .map(|span| heads.calls.get(span).copied())
            .collect::<Vec<_>>();
        assert_eq!(proofs.len(), 15);
        assert_eq!(proofs[0], Some(HeadProof::Literal));
        assert_eq!(proofs[1], Some(HeadProof::Unproven));
        assert_eq!(proofs[2], Some(HeadProof::Literal));
        assert_eq!(proofs[4], Some(HeadProof::Missing));
        assert_eq!(proofs[3], Some(HeadProof::Unproven));
        assert!(proofs[5..]
            .iter()
            .all(|proof| *proof == Some(HeadProof::Unproven)));
        if let Some(prior) = &prior {
            assert_eq!(&proofs, prior);
        }
        prior = Some(proofs);
        if efforts == ["low"] {
            assert!(file.query_annotation.is_empty());
            assert_eq!(
                observer
                    .snapshot()
                    .work
                    .get("postgres.sql.variants_composed")
                    .copied()
                    .unwrap_or(0),
                0
            );
        }
    }
}
