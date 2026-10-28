use super::{collect_check_facts, config, plan};
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn callback_payloads_reject_runtime_exports_but_keep_private_shadowed_bindings() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("callback-export-payload");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let paths = ["query.mts", "helpers.mts", "mutator.mts"].map(|name| root.join("src").join(name));
    let mut low = None;
    for efforts in [vec!["low"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, paths.to_vec(), plan(root, &config(&efforts)))
        });
        let file = facts.ts.get(&paths[0]).unwrap();
        let heads = file.query_annotation_heads.as_ref().unwrap();
        let decisions = file.embedded_sql[0]
            .1
            .call_spans
            .iter()
            .map(|span| heads.accepts(*span))
            .collect::<Vec<_>>();
        assert_eq!(decisions, [false, false, false, true, true]);
        if let Some(low) = &low {
            assert_eq!(&decisions, low);
        } else {
            low = Some(decisions);
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
