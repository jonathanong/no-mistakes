use super::super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn returned_builder_third_options_use_prepared_low_facts_only() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let path = root.join("text-field-third.mts");
    let mut low = None;
    for efforts in [vec!["low"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, vec![path.clone()], plan(root, &config(&efforts)))
        });
        let file = facts.ts.get(&path).expect("prepared file");
        let embedded = &file.embedded_sql[0].1;
        let heads = file.query_annotation_heads.as_ref().expect("low heads");
        let proofs = embedded
            .call_spans
            .iter()
            .map(|span| heads.calls.get(span).copied())
            .collect::<Vec<_>>();
        assert_eq!(proofs.len(), 10);
        assert_eq!(proofs[0], Some(HeadProof::Literal));
        assert_eq!(proofs[1], Some(HeadProof::Missing));
        assert!(proofs[2..]
            .iter()
            .all(|proof| *proof == Some(HeadProof::Unproven)));
        if let Some((previous, appends)) = &low {
            assert_eq!(&proofs, previous);
            assert_eq!(&heads.trusted_append_calls, appends);
        } else {
            low = Some((proofs, heads.trusted_append_calls.clone()));
            assert!(file.query_annotation.is_empty());
            assert!(embedded.calls.iter().all(|call| call.variants.is_empty()));
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
fn captured_text_reads_keep_low_proofs_under_mixed_demand() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let path = root.join("captured-text-field.mts");
    let mut low = None;
    let mut max_variants = None;
    for efforts in [vec!["low"], vec!["max"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, vec![path.clone()], plan(root, &config(&efforts)))
        });
        let file = facts.ts.get(&path).expect("prepared file");
        let embedded = &file.embedded_sql[0].1;
        if efforts == ["max"] {
            assert!(file.query_annotation_heads.is_none());
            max_variants = Some(
                embedded
                    .calls
                    .iter()
                    .map(|call| call.variants.len())
                    .collect::<Vec<_>>(),
            );
            continue;
        }
        let heads = file.query_annotation_heads.as_ref().expect("low heads");
        let proofs = embedded
            .call_spans
            .iter()
            .map(|span| heads.calls.get(span).copied())
            .collect::<Vec<_>>();
        assert_eq!(proofs.len(), 8);
        assert_eq!(&proofs[..2], &[Some(HeadProof::Literal); 2]);
        assert!(proofs[2..]
            .iter()
            .all(|proof| *proof == Some(HeadProof::Unproven)));
        if let Some(prior) = &low {
            assert_eq!(&proofs, prior);
            assert_eq!(
                embedded
                    .calls
                    .iter()
                    .map(|call| call.variants.len())
                    .collect::<Vec<_>>(),
                max_variants.clone().expect("max control"),
            );
        } else {
            low = Some(proofs);
            assert!(file.query_annotation.is_empty());
            assert!(embedded.calls.iter().all(|call| call.variants.is_empty()));
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
