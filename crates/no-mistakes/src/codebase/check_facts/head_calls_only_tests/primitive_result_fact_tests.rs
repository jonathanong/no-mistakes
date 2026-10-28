use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn primitive_result_source_proofs_are_memoized_without_low_evaluation() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let query = root.join("primitive-result-work.mts");
    let mut prior = None;
    let mut max_calls = None;
    for efforts in [vec!["max"], vec!["low"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, vec![query.clone()], plan(root, &config(&efforts)))
        });
        let file = facts.ts.get(&query).expect("prepared query");
        let embedded = &file.embedded_sql[0].1;
        let calls = format!("{:?}", embedded.calls);
        if efforts == ["max"] {
            max_calls = Some(calls);
            assert!(file.query_annotation_heads.is_none());
            continue;
        }
        assert_eq!(max_calls.as_ref(), Some(&calls));
        let heads = file.query_annotation_heads.as_ref().expect("low heads");
        let proofs = embedded
            .call_spans
            .iter()
            .map(|span| heads.calls.get(span).copied())
            .collect::<Vec<_>>();
        assert_eq!(&proofs[..4], &[Some(HeadProof::Literal); 4]);
        assert_eq!(&proofs[4..], &[Some(HeadProof::Unproven); 4]);
        assert_eq!(
            observer
                .snapshot()
                .work
                .get("postgres.annotation.primitive_result_censuses")
                .copied(),
            Some(2)
        );
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
