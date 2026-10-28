use super::{collect_check_facts, config, plan};
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn native_observations_keep_low_heads_and_max_raw_facts_independent() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let path = root.join("native-value-reads-cache.mts");
    let mut low = None;
    let mut raw = None;
    for efforts in [vec!["max"], vec!["low"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, vec![path.clone()], plan(root, &config(&efforts)))
        });
        let file = facts.ts.get(&path).unwrap();
        let embedded = &file.embedded_sql[0].1;
        let work = observer.snapshot().work;
        let count = |key: &str| work.get(key).copied().unwrap_or(0);
        if efforts.contains(&"low") {
            let heads = file.query_annotation_heads.as_ref().unwrap();
            let decisions = embedded
                .call_spans
                .iter()
                .map(|span| heads.accepts(*span))
                .collect::<Vec<_>>();
            assert_eq!(decisions, [true, true, false, false]);
            if let Some(prior) = &low {
                assert_eq!(&decisions, prior);
            }
            low = Some(decisions);
            // Two reads share each positive/negative owner census; imports are checked once.
            assert_eq!(count("postgres.annotation.native_value_owner_censuses"), 2);
            assert_eq!(
                count("postgres.annotation.native_value_constructor_censuses"),
                1
            );
            assert_eq!(count("postgres.annotation.head_syntax_visits"), 1);
            assert_eq!(count("postgres.annotation.head_binding_indexes"), 1);
        } else {
            assert_eq!(count("postgres.annotation.native_value_owner_censuses"), 0);
            assert_eq!(
                count("postgres.annotation.native_value_constructor_censuses"),
                0
            );
        }
        if efforts == ["low"] {
            assert!(file.query_annotation.is_empty());
            assert_eq!(count("postgres.sql.variants_composed"), 0);
        } else {
            let current = format!("{:?}", embedded.calls);
            if let Some(prior) = &raw {
                assert_eq!(&current, prior);
            }
            raw = Some(current);
        }
    }
}
