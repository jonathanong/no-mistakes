use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn prepared_imported_record_heads_keep_low_proofs_under_mixed_effort() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("imported-helper-records");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let caller = root.join("src/query.mts");
    let paths = vec![
        caller.clone(),
        root.join("src/helpers.mts"),
        root.join("src/plain-only.mts"),
        root.join("src/reexport.mts"),
    ];
    let mut low_proofs = None;
    for efforts in [vec!["low"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, paths.clone(), plan(root, &config(&efforts)))
        });
        let file = facts.ts.get(&caller).expect("prepared caller");
        let embedded = &file.embedded_sql[0].1;
        let heads = file.query_annotation_heads.as_ref().expect("low facts");
        let proofs = embedded
            .call_spans
            .iter()
            .map(|span| heads.calls.get(span).copied())
            .collect::<Vec<_>>();
        assert_eq!(
            proofs,
            vec![
                Some(HeadProof::Literal),
                Some(HeadProof::Missing),
                Some(HeadProof::Literal),
                Some(HeadProof::Unproven),
                Some(HeadProof::Unproven),
                Some(HeadProof::Unproven),
                Some(HeadProof::Unproven),
                Some(HeadProof::Unproven),
                Some(HeadProof::Literal),
                Some(HeadProof::Literal),
                Some(HeadProof::Literal),
                Some(HeadProof::Unproven),
            ]
        );
        if let Some(low_proofs) = &low_proofs {
            assert_eq!(&proofs, low_proofs);
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
