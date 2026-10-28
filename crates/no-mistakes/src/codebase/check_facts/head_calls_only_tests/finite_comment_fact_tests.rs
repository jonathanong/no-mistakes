use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn finite_comment_heads_share_prepared_facts_without_sql_composition() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let query = root.join("finite-comment-names.mts");
    let mut low_proofs = None;
    for efforts in [vec!["low"], vec!["low", "max"], vec!["max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, vec![query.clone()], plan(root, &config(&efforts)))
        });
        let file = facts.ts.get(&query).expect("prepared query");
        if efforts == ["max"] {
            continue;
        }
        let heads = file.query_annotation_heads.as_ref().expect("low heads");
        let proofs = file.embedded_sql[0]
            .1
            .call_spans
            .iter()
            .map(|span| heads.calls.get(span).copied())
            .collect::<Vec<_>>();
        assert_eq!(
            proofs,
            vec![
                Some(HeadProof::Literal),
                Some(HeadProof::Literal),
                Some(HeadProof::Unproven),
                Some(HeadProof::Unproven),
            ]
        );
        if let Some(prior) = &low_proofs {
            assert_eq!(&proofs, prior);
        }
        low_proofs = Some(proofs);
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

#[test]
fn indexed_finite_domains_preserve_prepared_low_heads_and_max_raw_facts() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let query = root.join("finite-comment-index-work.mts");
    let mut max_calls = None;
    let mut low = None;
    for efforts in [vec!["max"], vec!["low"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, vec![query.clone()], plan(root, &config(&efforts)))
        });
        let file = facts.ts.get(&query).unwrap();
        let embedded = &file.embedded_sql[0].1;
        let work = observer.snapshot().work;
        if efforts == ["max"] {
            max_calls = Some(format!("{:?}", embedded.calls));
            assert_eq!(
                work.get("postgres.annotation.finite_index_builds")
                    .copied()
                    .unwrap_or(0),
                0
            );
            continue;
        }
        assert_eq!(embedded.call_spans.len(), 64);
        let heads = file.query_annotation_heads.as_ref().unwrap();
        let decisions = embedded
            .call_spans
            .iter()
            .map(|span| heads.calls[span])
            .collect::<Vec<_>>();
        assert!(decisions[..32]
            .iter()
            .all(|proof| *proof == HeadProof::Literal));
        assert!(decisions[32..]
            .iter()
            .all(|proof| *proof == HeadProof::Unproven));
        assert_eq!(
            work.get("postgres.annotation.finite_index_builds").copied(),
            Some(1)
        );
        assert_eq!(
            work.get("postgres.annotation.finite_helper_domain_censuses")
                .copied(),
            Some(2)
        );
        if efforts == ["low"] {
            assert!(file.query_annotation.is_empty());
            assert_eq!(
                work.get("postgres.sql.variants_composed")
                    .copied()
                    .unwrap_or(0),
                0
            );
            low = Some(decisions);
        } else {
            assert_eq!(Some(decisions), low);
            assert_eq!(Some(format!("{:?}", embedded.calls)), max_calls);
        }
    }
}
