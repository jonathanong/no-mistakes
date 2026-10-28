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

#[test]
fn string_wanted_expansion_is_lazy_and_once_per_repeated_owner() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    for (name, calls, accepted, owners, references) in [
        ("string-prefix-unused-work.mts", 1, true, 0, 0),
        ("string-prefix-owner-work.mts", 64, true, 1, 65),
        ("string-prefix-negative-owner-work.mts", 64, false, 1, 66),
    ] {
        let path = root.join(name);
        let mut max_calls = None;
        let mut low = None;
        for efforts in [vec!["max"], vec!["low"], vec!["low", "max"]] {
            let observer = InvocationObserver::new(true);
            let facts = with_observer(Some(observer.clone()), || {
                collect_check_facts(root, vec![path.clone()], plan(root, &config(&efforts)))
            });
            let file = facts.ts.get(&path).unwrap();
            let embedded = &file.embedded_sql[0].1;
            let work = observer.snapshot().work;
            if efforts == ["max"] {
                max_calls = Some(format!("{:?}", embedded.calls));
                for metric in [
                    "postgres.annotation.string_wanted_owner_visits",
                    "postgres.annotation.string_wanted_reference_visits",
                    "postgres.annotation.string_wanted_lookup_visits",
                ] {
                    assert_eq!(work.get(metric).copied().unwrap_or(0), 0);
                }
                continue;
            }
            assert_eq!(embedded.call_spans.len(), calls);
            let heads = file.query_annotation_heads.as_ref().unwrap();
            let decisions = embedded
                .call_spans
                .iter()
                .map(|span| heads.accepts(*span))
                .collect::<Vec<_>>();
            assert!(decisions.iter().all(|value| *value == accepted));
            assert_eq!(
                work.get("postgres.annotation.string_wanted_owner_visits")
                    .copied()
                    .unwrap_or(0),
                owners
            );
            assert_eq!(
                work.get("postgres.annotation.string_wanted_reference_visits")
                    .copied()
                    .unwrap_or(0),
                references
            );
            if owners == 0 {
                assert_eq!(
                    work.get("postgres.annotation.string_wanted_lookup_visits")
                        .copied()
                        .unwrap_or(0),
                    0
                );
            } else {
                assert_eq!(
                    work.get("postgres.annotation.string_wanted_eligibility_censuses")
                        .copied(),
                    Some(1)
                );
                assert_eq!(
                    work.get("postgres.annotation.string_prefix_owner_censuses")
                        .copied(),
                    Some(1)
                );
            }
            if efforts == ["low"] {
                assert!(file.query_annotation.is_empty());
                assert_eq!(
                    work.get("postgres.sql.variants_composed")
                        .copied()
                        .unwrap_or(0),
                    0
                );
                low = Some(decisions);
            } else {
                assert_eq!(Some(decisions), low);
                assert_eq!(Some(format!("{:?}", embedded.calls)), max_calls);
            }
        }
    }
}
