use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn inert_later_defaults_keep_prepared_low_heads_without_evaluator_work() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("effect-defaults");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let query = root.join("src/query.mts");
    let helper = root.join("src/helpers.mts");
    let mut prior = None;
    for efforts in [vec!["low"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(
                root,
                vec![query.clone(), helper.clone()],
                plan(root, &config(&efforts)),
            )
        });
        let file = facts.ts.get(&query).expect("prepared caller");
        let heads = file.query_annotation_heads.as_ref().expect("low heads");
        let proofs = file.embedded_sql[0]
            .1
            .call_spans
            .iter()
            .map(|span| heads.calls.get(span).copied())
            .collect::<Vec<_>>();
        assert_eq!(proofs.len(), 24);
        assert_eq!(&proofs[..7], &[Some(HeadProof::Literal); 7]);
        assert!(proofs[7..]
            .iter()
            .all(|proof| *proof == Some(HeadProof::Unproven)));
        if let Some(prior) = &prior {
            assert_eq!(&proofs, prior);
        } else {
            prior = Some(proofs);
        }
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
