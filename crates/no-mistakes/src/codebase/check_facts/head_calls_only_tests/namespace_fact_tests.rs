use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::codebase::ts_source::discover_visible_paths;
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn namespace_helper_heads_use_only_prepared_exact_export_bindings() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("namespace-helper-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let query = root.join("src/query.mts");
    let mut low_proofs = None;
    for efforts in [vec!["low"], vec!["low", "max"], vec!["max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(
                root,
                discover_visible_paths(root),
                plan(root, &config(&efforts)),
            )
        });
        let file = facts.ts.get(&query).expect("prepared query");
        let embedded = &file.embedded_sql[0].1;
        if efforts == ["max"] {
            assert!(file
                .query_annotation_heads
                .as_ref()
                .is_none_or(|heads| heads.pending_namespaces.is_empty()));
            continue;
        }
        let heads = file.query_annotation_heads.as_ref().expect("low heads");
        let proofs = embedded
            .call_spans
            .iter()
            .map(|span| heads.calls.get(span).copied())
            .collect::<Vec<_>>();
        assert_eq!(proofs.len(), 21);
        assert_eq!(
            &proofs[..4],
            &[
                Some(HeadProof::Literal),
                Some(HeadProof::Literal),
                Some(HeadProof::Missing),
                Some(HeadProof::Literal)
            ]
        );
        assert!(proofs[4..11]
            .iter()
            .all(|proof| *proof == Some(HeadProof::Unproven)));
        assert_eq!(proofs[11], Some(HeadProof::Literal));
        assert!(proofs[12..]
            .iter()
            .all(|proof| *proof == Some(HeadProof::Unproven)));
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
