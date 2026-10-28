use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn async_and_local_source_callbacks_keep_low_heads_under_mixed_demand() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("callback-more");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let caller = root.join("src/query.mts");
    let reassigned = root.join("src/reassigned.mts");
    let unresolved = ["duplicate.mts", "eval-shadow.mts", "shadowed.mts"]
        .map(|name| root.join("src").join(name));
    let paths = vec![
        caller.clone(),
        root.join("src/helpers.mts"),
        reassigned.clone(),
        unresolved[0].clone(),
        unresolved[1].clone(),
        unresolved[2].clone(),
    ];
    let mut prior = None;
    for efforts in [vec!["low"], vec!["low", "max"]] {
        let mut config = config(&efforts);
        for (rule, effort) in config.rules.iter_mut().zip(&efforts) {
            rule.options = serde_yaml::from_str(&format!(
                "effort: {effort}\nimportSpecifier: '@bench/db'\nexecutorNames: [read, write]"
            ))
            .unwrap();
        }
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, paths.clone(), plan(root, &config))
        });
        let file = facts.ts.get(&caller).unwrap();
        let heads = file.query_annotation_heads.as_ref().unwrap();
        let proofs = file.embedded_sql[0]
            .1
            .call_spans
            .iter()
            .map(|span| heads.calls.get(span).copied())
            .collect::<Vec<_>>();
        assert_eq!(proofs.len(), 12);
        assert_eq!(&proofs[..2], &[Some(HeadProof::Literal); 2]);
        assert_eq!(&proofs[2..], &[Some(HeadProof::Unproven); 10]);
        let unstable = facts.ts.get(&reassigned).unwrap();
        let span = unstable.embedded_sql[0].1.call_spans[0];
        assert_eq!(
            unstable
                .query_annotation_heads
                .as_ref()
                .unwrap()
                .calls
                .get(&span),
            Some(&HeadProof::Unproven)
        );
        for path in &unresolved {
            let file = facts.ts.get(path).unwrap();
            let span = file.embedded_sql[0].1.call_spans[0];
            assert_eq!(
                file.query_annotation_heads
                    .as_ref()
                    .unwrap()
                    .calls
                    .get(&span),
                Some(&HeadProof::Unproven),
                "{}",
                path.display()
            );
        }
        if let Some(prior) = &prior {
            assert_eq!(&proofs, prior);
        } else {
            prior = Some(proofs);
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
            let work = &observer.snapshot().work;
            for key in [
                "postgres.annotation.callback_append_helper_indexes",
                "postgres.annotation.callback_local_payload_indexes",
            ] {
                assert!((2..=paths.len() as u64).contains(&work[key]), "{key}");
            }
            assert!(work["postgres.annotation.callback_append_helper_binding_visits"] > 0);
            assert!(work["postgres.annotation.callback_local_payload_binding_visits"] > 0);
        }
    }
}

#[test]
fn ordinary_executor_read_skips_callback_payload_indexes() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("callback-more");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let path = root.join("src/ordinary.mts");
    let observer = InvocationObserver::new(true);
    let facts = with_observer(Some(observer.clone()), || {
        collect_check_facts(root, vec![path.clone()], plan(root, &config(&["low"])))
    });
    assert!(facts
        .ts
        .get(&path)
        .unwrap()
        .query_annotation_heads
        .is_some());
    let work = observer.snapshot().work;
    for key in [
        "postgres.annotation.callback_append_helper_indexes",
        "postgres.annotation.callback_local_payload_indexes",
        "postgres.annotation.callback_append_helper_binding_visits",
        "postgres.annotation.callback_local_payload_binding_visits",
    ] {
        assert!(!work.contains_key(key), "{key}");
    }
}
