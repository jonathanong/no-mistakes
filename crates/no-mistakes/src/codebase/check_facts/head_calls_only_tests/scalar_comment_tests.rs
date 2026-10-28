use super::{collect_check_facts, config, plan};
use crate::diagnostics::{with_observer, InvocationObserver};

fn saved() -> tempfile::TempDir {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("scalar-comment-names");
    crate::test_support::materialize_saved_fixture(&path)
}
fn expected(source: &str) -> Vec<bool> {
    source
        .lines()
        .filter_map(|line| line.split_once("// proof: "))
        .map(|(_, value)| value == "Literal")
        .collect()
}
#[test]
fn scalar_append_names_require_complete_literal_callers_and_closed_native_heads() {
    let fixture = saved();
    let root = fixture.path();
    let paths = [
        "query.mts",
        "ctor-alias.mts",
        "ctor-mutation.mts",
        "ctor-namespace.mts",
        "eval.mts",
        "owner-cache.mts",
    ]
    .map(|name| root.join("src").join(name));
    let mut max_calls = None;
    for efforts in [vec!["max"], vec!["low"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, paths.to_vec(), plan(root, &config(&efforts)))
        });
        let calls = paths
            .iter()
            .map(|path| format!("{:?}", facts.ts.get(path).unwrap().embedded_sql[0].1.calls))
            .collect::<Vec<_>>();
        if efforts == ["max"] {
            assert!(!observer
                .snapshot()
                .work
                .contains_key("postgres.annotation.scalar_name_helper_censuses"));
            assert!(!observer
                .snapshot()
                .work
                .contains_key("postgres.annotation.scalar_name_constructor_censuses"));
            max_calls = Some(calls);
            continue;
        }
        for path in &paths {
            let source = std::fs::read_to_string(path).unwrap();
            let file = facts.ts.get(path).unwrap();
            let heads = file.query_annotation_heads.as_ref().unwrap();
            let actual = file.embedded_sql[0]
                .1
                .call_spans
                .iter()
                .map(|span| heads.accepts(*span))
                .collect::<Vec<_>>();
            assert_eq!(actual, expected(&source), "{}", path.display());
            if efforts == ["low"] {
                assert!(file.query_annotation.is_empty());
                assert!(file.embedded_sql[0]
                    .1
                    .calls
                    .iter()
                    .all(|call| call.variants.is_empty()));
            }
        }
        if efforts == ["low"] {
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
            assert_eq!(Some(calls), max_calls);
        }
    }
}
#[test]
fn scalar_name_domains_cache_positive_and_negative_owners_before_many_executor_reads() {
    let fixture = saved();
    let root = fixture.path();
    let path = root.join("src/work.mts");
    let observer = InvocationObserver::new(true);
    let facts = with_observer(Some(observer.clone()), || {
        collect_check_facts(root, vec![path.clone()], plan(root, &config(&["low"])))
    });
    let file = facts.ts.get(&path).unwrap();
    let heads = file.query_annotation_heads.as_ref().unwrap();
    let decisions = file.embedded_sql[0]
        .1
        .call_spans
        .iter()
        .map(|span| heads.accepts(*span))
        .collect::<Vec<_>>();
    assert_eq!(decisions, [true, true, true, false, false]);
    let work = observer.snapshot().work;
    assert_eq!(
        work.get("postgres.annotation.scalar_name_helper_censuses"),
        Some(&2)
    );
    assert_eq!(
        work.get("postgres.annotation.scalar_name_constructor_censuses"),
        Some(&1)
    );
    assert_eq!(work.get("postgres.annotation.head_syntax_visits"), Some(&1));
    assert_eq!(
        work.get("postgres.annotation.head_binding_indexes"),
        Some(&1)
    );
}
