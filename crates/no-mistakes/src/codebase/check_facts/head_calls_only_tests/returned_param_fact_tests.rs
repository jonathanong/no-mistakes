use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn returned_first_builder_projects_one_prepared_hop_without_low_evaluation() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("returned-params");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let caller = root.join("src/query.mts");
    let helper = root.join("src/helpers.mts");
    let reexport = root.join("src/reexport.mts");
    let mut low_proofs = None;
    let mut max_calls = None;
    for efforts in [vec!["low"], vec!["max"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(
                root,
                vec![caller.clone(), helper.clone(), reexport.clone()],
                plan(root, &config(&efforts)),
            )
        });
        let source = facts.ts.get(&helper).expect("prepared helper-only module");
        let file = facts.ts.get(&caller).expect("prepared caller");
        let embedded = &file.embedded_sql[0].1;
        if efforts == ["max"] {
            assert!(file.query_annotation_heads.is_none());
            max_calls = Some(format!("{:?}", embedded.calls));
            continue;
        }
        let producer = source
            .query_annotation_heads
            .as_ref()
            .expect("producer heads");
        assert!(producer
            .exported_returned_params
            .contains("selectWithOwner"));
        assert!(!producer.exported_returned_params.contains("wrongReturn"));
        assert!(!producer.exported_returned_params.contains("aliasEscape"));
        assert!(!producer.exported_returned_params.contains("unsafeDefault"));
        let heads = file.query_annotation_heads.as_ref().expect("caller heads");
        assert!(heads.has_pending_imports());
        let proofs = embedded
            .call_spans
            .iter()
            .map(|span| heads.calls.get(span).copied())
            .collect::<Vec<_>>();
        assert_eq!(proofs.len(), 13);
        assert_eq!(proofs[0], Some(HeadProof::Literal));
        assert!(proofs[1..6]
            .iter()
            .all(|proof| *proof == Some(HeadProof::Unproven)));
        assert_eq!(proofs[6], Some(HeadProof::Missing));
        assert_eq!(&proofs[7..10], &[Some(HeadProof::Unproven); 3]);
        assert_eq!(proofs[10], Some(HeadProof::Missing));
        assert_eq!(proofs[11], Some(HeadProof::Literal));
        assert_eq!(proofs[12], Some(HeadProof::Missing));
        if let Some(prior) = &low_proofs {
            assert_eq!(&proofs, prior);
            assert_eq!(format!("{:?}", embedded.calls), max_calls.clone().unwrap());
        } else {
            low_proofs = Some(proofs);
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
