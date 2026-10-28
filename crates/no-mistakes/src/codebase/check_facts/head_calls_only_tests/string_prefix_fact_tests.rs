use super::{collect_check_facts, config, plan};
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn string_prefix_censuses_cache_success_and_failure_in_prepared_low_and_mixed() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let path = root.join("string-prefix-cache.mts");
    let mut expected = None;
    let mut max_calls = None;
    for efforts in [vec!["max"], vec!["low"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, vec![path.clone()], plan(root, &config(&efforts)))
        });
        let file = facts.ts.get(&path).unwrap();
        let embedded = &file.embedded_sql[0].1;
        let work = observer.snapshot().work;
        if efforts == ["max"] {
            assert!(!work.contains_key("postgres.annotation.string_prefix_owner_censuses"));
            max_calls = Some(format!("{:?}", embedded.calls));
            continue;
        }
        let heads = file.query_annotation_heads.as_ref().unwrap();
        let decisions = embedded
            .call_spans
            .iter()
            .map(|span| heads.accepts(*span))
            .collect::<Vec<_>>();
        assert_eq!(decisions, [true, true, true, false, false, false]);
        if let Some(prior) = &expected {
            assert_eq!(&decisions, prior);
        } else {
            expected = Some(decisions);
        }
        assert_eq!(
            work.get("postgres.annotation.string_prefix_owner_censuses"),
            Some(&2)
        );
        assert_eq!(work.get("postgres.annotation.head_syntax_visits"), Some(&1));
        assert_eq!(
            work.get("postgres.annotation.head_binding_indexes"),
            Some(&1)
        );
        if efforts == ["low"] {
            assert_eq!(
                work.get("postgres.sql.variants_composed")
                    .copied()
                    .unwrap_or(0),
                0
            );
            assert!(file.query_annotation.is_empty());
            assert!(embedded.calls.iter().all(|call| call.variants.is_empty()));
        } else {
            assert_eq!(Some(format!("{:?}", embedded.calls)), max_calls);
            assert!(embedded
                .call_spans
                .iter()
                .all(|span| !heads.accepts_trusted_append(*span)));
        }
    }
}

#[test]
fn real_prepared_branch_and_interpolation_proofs_match_the_saved_oracles() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let path = root.join("string-prefix.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let expected = source
        .lines()
        .filter_map(|line| line.split_once("// low: "))
        .map(|(_, expected)| expected.parse::<bool>().unwrap())
        .collect::<Vec<_>>();
    for efforts in [vec!["low"], vec!["low", "max"]] {
        let facts = collect_check_facts(root, vec![path.clone()], plan(root, &config(&efforts)));
        let file = facts.ts.get(&path).unwrap();
        let heads = file.query_annotation_heads.as_ref().unwrap();
        let decisions = file.embedded_sql[0]
            .1
            .call_spans
            .iter()
            .map(|span| heads.accepts(*span))
            .collect::<Vec<_>>();
        assert_eq!(decisions, expected);
    }
}
