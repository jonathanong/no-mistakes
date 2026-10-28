use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn composed_effects_keep_low_heads_without_variant_work_or_max_widening() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let query = root.join("append-effect-composition.mts");
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
        let spans = &file.embedded_sql[0].1.call_spans;
        let proofs = spans
            .iter()
            .map(|span| heads.calls.get(span).copied())
            .collect::<Vec<_>>();
        assert_eq!(proofs.len(), 12);
        assert_eq!(proofs[0], Some(HeadProof::Literal));
        assert!(proofs[1..]
            .iter()
            .all(|proof| *proof == Some(HeadProof::Unproven)));
        assert!(!heads.accepts_trusted_append(spans[0]));
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
