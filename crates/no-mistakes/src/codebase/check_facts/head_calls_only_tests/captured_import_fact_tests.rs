use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn captured_native_prefix_requires_every_direct_import_effect_without_low_evaluation() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("captured-import-effects");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let paths = [
        "query.mts",
        "effects.mts",
        "gap.mts",
        "mutable.mts",
        "mutable.cjs",
    ]
    .map(|name| root.join("src").join(name));
    let mut prior = None;
    let mut max_calls = None;
    for efforts in [vec!["max"], vec!["low"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, paths.to_vec(), plan(root, &config(&efforts)))
        });
        let file = facts.ts.get(&paths[0]).unwrap();
        if efforts == ["max"] {
            max_calls = Some(format!("{:?}", file.embedded_sql[0].1.calls));
            assert!(file.query_annotation_heads.is_none());
            assert!(!observer
                .snapshot()
                .work
                .contains_key("postgres.annotation.captured_lifetime_censuses"));
            continue;
        }
        if efforts == ["low", "max"] {
            assert_eq!(
                max_calls.as_ref(),
                Some(&format!("{:?}", file.embedded_sql[0].1.calls))
            );
        }
        let heads = file.query_annotation_heads.as_ref().unwrap();
        assert!(file.embedded_sql[0].1.call_spans[..5]
            .iter()
            .all(|span| !heads.accepts_trusted_append(*span)));
        let proofs = file.embedded_sql[0]
            .1
            .call_spans
            .iter()
            .map(|span| heads.calls.get(span).copied())
            .collect::<Vec<_>>();
        assert_eq!(proofs.len(), 18);
        assert_eq!(&proofs[..5], &[Some(HeadProof::Literal); 5]);
        assert_eq!(&proofs[5..], &[Some(HeadProof::Unproven); 13]);
        // Both reads of one owner retain its entire pending list; no effectful
        // alternative is dropped simply because another import resolved.
        assert!(heads
            .conditional_effects
            .iter()
            .any(|condition| condition.required.len() == 2));
        for path in &paths[2..] {
            let file = facts.ts.get(path).unwrap();
            let heads = file.query_annotation_heads.as_ref().unwrap();
            assert_eq!(file.embedded_sql[0].1.call_spans.len(), 1);
            assert!(file.embedded_sql[0]
                .1
                .call_spans
                .iter()
                .all(|span| heads.calls.get(span) == Some(&HeadProof::Unproven)));
        }
        if let Some(prior) = &prior {
            assert_eq!(&proofs, prior);
        } else {
            prior = Some(proofs);
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

#[test]
fn many_executor_reads_share_one_closed_owner_and_pending_callee_census() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("captured-import-effects");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let caller = root.join("src/many.mts");
    let helper = root.join("src/effects.mts");
    let observer = InvocationObserver::new(true);
    let facts = with_observer(Some(observer.clone()), || {
        collect_check_facts(
            root,
            vec![caller.clone(), helper],
            plan(root, &config(&["low"])),
        )
    });
    let file = facts.ts.get(&caller).unwrap();
    let heads = file.query_annotation_heads.as_ref().unwrap();
    let spans = &file.embedded_sql[0].1.call_spans;
    assert_eq!(spans.len(), 64);
    assert!(spans
        .iter()
        .all(|span| heads.calls.get(span) == Some(&HeadProof::Literal)));
    assert_eq!(heads.conditional_effects.len(), 64);
    let module = file.ts.module_bindings.as_ref().unwrap();
    let owner = module
        .bindings
        .iter()
        .find(|binding| binding.name == "q")
        .unwrap();
    let callee = module
        .bindings
        .iter()
        .find(|binding| binding.name == "append")
        .unwrap();
    let work = observer.snapshot().work;
    assert_eq!(
        work.get("postgres.annotation.captured_lifetime_censuses")
            .copied(),
        Some(1)
    );
    assert_eq!(
        work.get("postgres.annotation.captured_lifetime_reference_visits")
            .copied(),
        Some(owner.references.len() as u64)
    );
    assert_eq!(
        work.get("postgres.annotation.captured_pending_callee_censuses")
            .copied(),
        Some(1)
    );
    assert_eq!(
        work.get("postgres.annotation.captured_pending_callee_reference_visits")
            .copied(),
        Some(callee.references.len() as u64)
    );
}

#[test]
fn ordinary_literal_input_needs_no_capture_or_pending_owner_census() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("captured-import-effects");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        collect_check_facts(
            root,
            vec![root.join("src/ordinary.mts")],
            plan(root, &config(&["low"])),
        )
    });
    let work = observer.snapshot().work;
    assert!(!work.contains_key("postgres.annotation.captured_lifetime_censuses"));
    assert!(!work.contains_key("postgres.annotation.captured_pending_callee_censuses"));
}
