use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn async_native_origins_require_all_prepared_effects_and_exact_await_consumption() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("async-return-origin");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let paths = [
        "query.mts",
        "helpers.mts",
        "effects.mts",
        "gap.mts",
        "gap-query.mts",
        "dynamic.mts",
        "dynamic-query.mts",
        "mutated.mts",
        "mutated-query.mts",
        "exported-query.mts",
    ]
    .map(|name| root.join("src").join(name));
    let mut low = None;
    let mut max_calls = None;
    for efforts in [vec!["low"], vec!["max"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, paths.to_vec(), plan(root, &config(&efforts)))
        });
        let file = facts.ts.get(&paths[0]).unwrap();
        if efforts == ["max"] {
            max_calls = Some(format!("{:?}", file.embedded_sql[0].1.calls));
            assert!(file.query_annotation_heads.is_none());
            assert!(facts
                .ts
                .get(&paths[1])
                .unwrap()
                .query_annotation_heads
                .is_none());
            assert!(!observer
                .snapshot()
                .work
                .contains_key("postgres.annotation.async_return_censuses"));
            assert!(!observer
                .snapshot()
                .work
                .contains_key("postgres.annotation.async_import_censuses"));
            continue;
        }
        if efforts == ["low", "max"] {
            assert_eq!(
                max_calls.as_ref(),
                Some(&format!("{:?}", file.embedded_sql[0].1.calls))
            );
        }
        let heads = file.query_annotation_heads.as_ref().unwrap();
        let proofs = file.embedded_sql[0]
            .1
            .call_spans
            .iter()
            .map(|span| heads.calls.get(span).copied())
            .collect::<Vec<_>>();
        assert_eq!(proofs.len(), 33);
        assert_eq!(
            &proofs[..3],
            &[
                Some(HeadProof::Literal),
                Some(HeadProof::Literal),
                Some(HeadProof::Missing)
            ]
        );
        assert!(proofs[3..]
            .iter()
            .all(|proof| *proof == Some(HeadProof::Unproven)));
        for index in [4, 6, 8, 9] {
            let gap = facts.ts.get(&paths[index]).unwrap();
            let spans = &gap.embedded_sql[0].1.call_spans;
            assert_eq!(spans.len(), 1, "{}", paths[index].display());
            let heads = gap.query_annotation_heads.as_ref().unwrap();
            assert!(
                spans
                    .iter()
                    .all(|span| heads.calls.get(span) == Some(&HeadProof::Unproven)),
                "{}",
                paths[index].display()
            );
        }
        if let Some(prior) = &low {
            assert_eq!(prior, &proofs);
        } else {
            low = Some(proofs);
        }
        if efforts == ["low"] {
            assert!(file.query_annotation.is_empty());
            assert!(file.embedded_sql[0]
                .1
                .calls
                .iter()
                .all(|call| call.variants.is_empty()));
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

#[path = "async_return_work_fact_tests.rs"]
mod work;
