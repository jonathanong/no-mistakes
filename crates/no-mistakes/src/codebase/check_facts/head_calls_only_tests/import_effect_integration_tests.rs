use super::{collect_check_facts, config, plan};
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn async_and_captured_deferred_effects_keep_the_plain_and_closed_export_gate() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("import-effect-integration");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let paths =
        ["query.mts", "factories.mts", "targets.mts"].map(|name| root.join("src").join(name));
    let mut max_raw = None;
    for efforts in [vec!["max"], vec!["low"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, paths.to_vec(), plan(root, &config(&efforts)))
        });
        let file = facts.ts.get(&paths[0]).unwrap();
        let embedded = &file.embedded_sql[0].1;
        if efforts == ["max"] {
            max_raw = Some(format!("{:?}", embedded.calls));
            continue;
        }
        let heads = file.query_annotation_heads.as_ref().unwrap();
        let decisions = embedded
            .call_spans
            .iter()
            .map(|span| heads.accepts(*span))
            .collect::<Vec<_>>();
        assert_eq!(
            decisions,
            [true, false, false, false, true, false, false, false]
        );
        let factories = facts
            .ts
            .get(&paths[1])
            .unwrap()
            .query_annotation_heads
            .as_ref()
            .unwrap();
        let targets = facts
            .ts
            .get(&paths[2])
            .unwrap()
            .query_annotation_heads
            .as_ref()
            .unwrap();
        assert_eq!(factories.exported_async_returns.len(), 4);
        for span in &embedded.call_spans[4..] {
            assert!(heads
                .conditional_effects
                .iter()
                .any(|condition| condition.span == *span && condition.required.len() == 1));
        }
        let deferred = factories
            .pending_effects
            .iter()
            .filter(|effect| effect.require_plain)
            .collect::<Vec<_>>();
        assert_eq!(deferred.len(), 4);
        for effect in deferred {
            assert_eq!(effect.proven_by(targets), effect.imported == "plain");
            // Legacy direct effects keep accepts(shape); only the new dependency is stricter.
            let mut direct = effect.clone();
            direct.require_plain = false;
            assert!(direct.proven_by(targets), "{}", direct.imported);
        }
        if efforts == ["low"] {
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
        } else {
            assert_eq!(Some(format!("{:?}", embedded.calls)), max_raw);
        }
    }
}
