use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn exact_imported_callback_proofs_stay_in_low_prepared_calls() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("callback-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let caller = root.join("src/query.mts");
    let target = root.join("src/helpers.mts");
    for efforts in [vec!["low"], vec!["low", "max"], vec!["max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(
                root,
                vec![caller.clone(), target.clone()],
                plan(root, &config(&efforts)),
            )
        });
        let file = facts.ts.get(&caller).unwrap();
        if efforts == ["max"] {
            assert!(file
                .query_annotation_heads
                .as_ref()
                .is_none_or(|heads| heads.pending_callbacks.is_empty()));
            continue;
        }
        let heads = file.query_annotation_heads.as_ref().unwrap();
        let proofs = file.embedded_sql[0]
            .1
            .call_spans
            .iter()
            .map(|span| heads.calls.get(span).copied())
            .collect::<Vec<_>>();
        assert_eq!(proofs.len(), 11);
        assert_eq!(&proofs[..2], &[Some(HeadProof::Literal); 2]);
        assert_eq!(&proofs[2..10], &[Some(HeadProof::Unproven); 8]);
        assert_eq!(proofs[10], Some(HeadProof::Missing));
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
